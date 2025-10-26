use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DownloadId(u64);

#[derive(Debug, Clone)]
pub enum DownloadState {
    Queued,
    Running { downloaded: u64, total: Option<u64> },
    Paused { downloaded: u64, total: Option<u64> },
    Completed,
    Failed(String),
    Canceled,
}

struct Entry {
    url: String,
    dst_path: PathBuf,
    downloaded: u64,
    total: Option<u64>,
    paused: Arc<AtomicBool>,
    canceled: Arc<AtomicBool>,
    state: DownloadState,
    handle: Option<JoinHandle<()>>,
}

struct Manager {
    next_id: u64,
    entries: HashMap<DownloadId, Arc<Mutex<Entry>>>,
}

impl Manager {
    fn new() -> Self { Self { next_id: 1, entries: HashMap::new() } }
}

fn mgr() -> &'static Mutex<Manager> {
    static M: OnceLock<Mutex<Manager>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(Manager::new()))
}

fn default_download_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MONAZITE_DOWNLOAD_DIR") { return PathBuf::from(dir); }
    if let Some(ud) = directories::UserDirs::new() {
        if let Some(d) = ud.download_dir() { return d.to_path_buf(); }
    }
    // Fallback
    PathBuf::from("Downloads")
}

fn ensure_sandboxed(dst: &Path) -> Result<PathBuf> {
    let base = default_download_dir();
    let base = fs::canonicalize(&base).unwrap_or(base);
    let target_parent = dst.parent().unwrap_or(&base);
    let target_parent = if target_parent.as_os_str().is_empty() { &base } else { target_parent };
    fs::create_dir_all(target_parent).ok();
    let canon_parent = fs::canonicalize(target_parent).unwrap_or(target_parent.to_path_buf());
    let full = canon_parent.join(dst.file_name().ok_or_else(|| anyhow!("invalid dst"))?);
    if !canon_parent.starts_with(&base) {
        return Err(anyhow!("dst outside download sandbox"));
    }
    Ok(full)
}

fn parse_scheme(url: &str) -> Option<&str> {
    url.split("://").next()
}

fn file_path_from_url(url: &str) -> Result<PathBuf> {
    // Expect file:///absolute/path
    if !url.starts_with("file://") {
        return Err(anyhow!("unsupported scheme (only file:// for now)"));
    }
    let path = &url["file://".len()..];
    if path.is_empty() { return Err(anyhow!("invalid file:// url")); }
    Ok(PathBuf::from(path))
}

pub fn download(url: &str, dst: &Path) -> Result<DownloadId> {
    let scheme = parse_scheme(url).unwrap_or("");
    if scheme != "file" {
        return Err(anyhow!("unsupported scheme: {} (only file:// in S10 minimal)", scheme));
    }
    let src = file_path_from_url(url)?;
    let dst_full = ensure_sandboxed(dst)?;

    let paused = Arc::new(AtomicBool::new(false));
    let canceled = Arc::new(AtomicBool::new(false));

    let mut m = mgr().lock().unwrap();
    let id = DownloadId(m.next_id);
    m.next_id += 1;

    let entry = Arc::new(Mutex::new(Entry {
        url: url.to_string(),
        dst_path: dst_full.clone(),
        downloaded: 0,
        total: None,
        paused: paused.clone(),
        canceled: canceled.clone(),
        state: DownloadState::Queued,
        handle: None,
    }));

    let entry_cloned = entry.clone();
    let handle = thread::spawn(move || {
        let mut e = entry_cloned.lock().unwrap();
        e.state = DownloadState::Running { downloaded: e.downloaded, total: None };
        drop(e);

        // Determine sizes
        let total_size = fs::metadata(&src).ok().map(|m| m.len());
        let mut out = OpenOptions::new().create(true).write(true).append(true).open(&dst_full);
        if let Err(err) = &out { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Failed(err.to_string()); return; }
        let mut out = out.unwrap();
        let mut done = out.metadata().ok().map(|m| m.len()).unwrap_or(0);

        // Open src and seek to current done
        let mut inp = File::open(&src);
        if let Err(err) = &inp { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Failed(err.to_string()); return; }
        let mut inp = inp.unwrap();
        if let Err(err) = inp.seek(SeekFrom::Start(done)) { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Failed(err.to_string()); return; }

        let mut buf = vec![0u8; 64 * 1024];
        loop {
            if canceled.load(Ordering::SeqCst) {
                let mut e = entry_cloned.lock().unwrap();
                e.downloaded = done; e.state = DownloadState::Canceled; return;
            }
            while paused.load(Ordering::SeqCst) {
                {
                    let mut e = entry_cloned.lock().unwrap();
                    e.downloaded = done; e.total = total_size; e.state = DownloadState::Paused { downloaded: done, total: total_size };
                }
                thread::sleep(Duration::from_millis(20));
                if canceled.load(Ordering::SeqCst) { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Canceled; return; }
            }
            let readn = match inp.read(&mut buf) { Ok(0) => 0, Ok(n) => n as u64, Err(_) => { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Failed("read error".to_string()); return; } };
            if readn == 0 { break; }
            if let Err(_) = out.write_all(&buf[..readn as usize]) { let mut e = entry_cloned.lock().unwrap(); e.state = DownloadState::Failed("write error".to_string()); return; }
            done += readn;
            {
                let mut e = entry_cloned.lock().unwrap();
                e.downloaded = done; e.total = total_size; e.state = DownloadState::Running { downloaded: done, total: total_size };
            }
            // Slow down a bit to make pause/resume observable in tests
            thread::sleep(Duration::from_millis(5));
        }
        let mut e = entry_cloned.lock().unwrap();
        e.downloaded = done; e.total = total_size; e.state = DownloadState::Completed;
    });

    entry.lock().unwrap().handle = Some(handle);
    m.entries.insert(id, entry);
    Ok(id)
}

pub fn pause(id: DownloadId) -> Result<()> {
    let m = mgr().lock().unwrap();
    let Some(e) = m.entries.get(&id) else { return Err(anyhow!("unknown id")); };
    e.lock().unwrap().paused.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn resume(id: DownloadId) -> Result<()> {
    let m = mgr().lock().unwrap();
    let Some(e) = m.entries.get(&id) else { return Err(anyhow!("unknown id")); };
    e.lock().unwrap().paused.store(false, Ordering::SeqCst);
    Ok(())
}

pub fn cancel(id: DownloadId) -> Result<()> {
    let m = mgr().lock().unwrap();
    let Some(e) = m.entries.get(&id) else { return Err(anyhow!("unknown id")); };
    e.lock().unwrap().canceled.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn status(id: DownloadId) -> Option<DownloadState> {
    let m = mgr().lock().unwrap();
    m.entries.get(&id).map(|e| e.lock().unwrap().state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_source_file(dir: &Path, name: &str, bytes: usize) -> PathBuf {
        let src = dir.join(name);
        let mut f = File::create(&src).unwrap();
        let pattern: Vec<u8> = (0..=255u32).map(|n| (n as u8)).collect();
        let mut written = 0;
        while written < bytes {
            let chunk = std::cmp::min(bytes - written, pattern.len());
            f.write_all(&pattern[..chunk]).unwrap();
            written += chunk;
        }
        src
    }

    #[test]
    fn file_download_pause_resume_complete() {
        let base = std::env::temp_dir().join(format!(
            "dm_test_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()
        ));
        let src_dir = base.join("src");
        let dst_dir = base.join("Downloads");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&dst_dir).unwrap();
        std::env::set_var("MONAZITE_DOWNLOAD_DIR", &dst_dir);

        let src = write_source_file(&src_dir, "big.bin", 2 * 1024 * 1024); // 2MB
        let url = format!("file://{}", src.to_str().unwrap());
        let dst = PathBuf::from("big.bin");

        let id = download(&url, &dst).unwrap();
        // wait a bit
        thread::sleep(Duration::from_millis(30));
        pause(id).unwrap();
        let s1 = {
            let deadline = std::time::Instant::now() + Duration::from_millis(300);
            loop {
                if let Some(s) = status(id) {
                    if let DownloadState::Paused { .. } = s { break s; }
                }
                if std::time::Instant::now() > deadline { panic!("expected paused"); }
                thread::sleep(Duration::from_millis(5));
            }
        };
        // ensure paused holds
        let paused_bytes = if let DownloadState::Paused { downloaded, .. } = s1 { downloaded } else { 0 };
        thread::sleep(Duration::from_millis(30));
        let s2 = status(id).unwrap();
        if let DownloadState::Paused { downloaded, .. } = s2 { assert_eq!(downloaded, paused_bytes); } else { panic!("still paused") }

        resume(id).unwrap();
        // wait for completion
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(DownloadState::Completed) = status(id) { break; }
            if std::time::Instant::now() > deadline { panic!("timeout"); }
            thread::sleep(Duration::from_millis(10));
        }
        // compare files
        let dst_full = super::ensure_sandboxed(&PathBuf::from("big.bin")).unwrap();
        assert_eq!(fs::metadata(&dst_full).unwrap().len(), fs::metadata(&src).unwrap().len());
        let mut a = Vec::new(); let mut b = Vec::new();
        File::open(&dst_full).unwrap().read_to_end(&mut a).unwrap();
        File::open(&src).unwrap().read_to_end(&mut b).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn sandbox_rejects_outside_paths() {
        let base = std::env::temp_dir().join(format!(
            "dm_test_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()
        ));
        let src_dir = base.join("src");
        let dst_dir = base.join("Downloads");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&dst_dir).unwrap();
        std::env::set_var("MONAZITE_DOWNLOAD_DIR", &dst_dir);

        let src = write_source_file(&src_dir, "a.bin", 128 * 1024);
        let url = format!("file://{}", src.to_str().unwrap());
        let outside = base.join(".." ).join("escape.bin");
        let err = download(&url, &outside).unwrap_err();
        assert!(format!("{}", err).contains("outside download sandbox"));
    }

    #[test]
    fn unsupported_http_scheme() {
        let dst = PathBuf::from("x.bin");
        let err = download("http://example.com/a.bin", &dst).unwrap_err();
        assert!(format!("{}", err).contains("unsupported scheme"));
    }
}

