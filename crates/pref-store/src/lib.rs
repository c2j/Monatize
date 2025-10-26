use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_BYTES: usize = 100 * 1024 * 1024; // 100 MB
const FILE_NAME: &str = "prefs.json";

fn default_config_dir() -> PathBuf {
    // Allow override for tests or custom deployments
    if let Ok(dir) = std::env::var("MONAZITE_PREFS_DIR") {
        return PathBuf::from(dir);
    }
    // Fallback to platform config dir: ~/.config/Monazite
    if let Some(proj) = directories::ProjectDirs::from("dev", "Monazite", "Monazite") {
        return proj.config_dir().to_path_buf();
    }
    // Last resort: current directory
    PathBuf::from(".")
}

fn prefs_path() -> PathBuf { default_config_dir().join(FILE_NAME) }

fn load_map(path: &Path) -> Result<HashMap<String, Value>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data = fs::read(path)?;
    if data.is_empty() {
        return Ok(HashMap::new());
    }
    let map: HashMap<String, Value> = serde_json::from_slice(&data)
        .map_err(|e| anyhow!("prefs.json corrupted: {}", e))?;
    Ok(map)
}

fn save_map(path: &Path, map: &HashMap<String, Value>) -> Result<()> {
    let dir = path.parent().ok_or_else(|| anyhow!("invalid prefs path"))?;
    fs::create_dir_all(dir)?;
    let buf = serde_json::to_vec_pretty(map)?;
    if buf.len() > MAX_BYTES { return Err(anyhow!("pref-store size exceeds 100MB")); }

    // Write to temp file then atomic rename
    let tmp_path = dir.join(format!("{}.tmp", FILE_NAME));
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(&buf)?;
        f.sync_all()?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Ensure readable by user only by default (600)
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp_path, path)?; // Atomic on POSIX when same mount point
    Ok(())
}

pub fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    let path = prefs_path();
    let Ok(map) = load_map(&path) else { return None; };
    let Some(val) = map.get(key) else { return None; };
    serde_json::from_value::<T>(val.clone()).ok()
}

pub fn set<T: Serialize>(key: &str, value: &T) -> Result<()> {
    let path = prefs_path();
    let mut map = load_map(&path).unwrap_or_default();
    map.insert(key.to_string(), serde_json::to_value(value)?);
    save_map(&path, &map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn get_in(dir: &Path, key: &str) -> Option<Value> {
        let path = dir.join(FILE_NAME);
        let map = load_map(&path).ok()?;
        map.get(key).cloned()
    }

    fn set_in(dir: &Path, key: &str, value: &Value) -> Result<()> {
        let path = dir.join(FILE_NAME);
        let mut map = load_map(&path).unwrap_or_default();
        map.insert(key.to_string(), value.clone());
        save_map(&path, &map)
    }

    #[test]
    fn write_and_read_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        set_in(dir, "ui.theme", &Value::String("dark".into())).unwrap();
        let v = get_in(dir, "ui.theme").unwrap();
        assert_eq!(v, Value::String("dark".into()));
        // And via typed API using env override
        std::env::set_var("MONAZITE_PREFS_DIR", dir);
        super::set("ui.zoom", &1.25f64).unwrap();
        let z: Option<f64> = super::get("ui.zoom");
        assert_eq!(z, Some(1.25));
    }

    #[test]
    fn concurrent_writes_do_not_corrupt_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let threads = 8;
        let iters = 50;
        let dir_arc = Arc::new(dir);
        let mut hs = vec![];
        for t in 0..threads {
            let d = dir_arc.clone();
            hs.push(thread::spawn(move || {
                for i in 0..iters {
                    let key = format!("k{}.{}", t, i);
                    let val = Value::String(format!("v{}", i));
                    let _ = set_in(&d, &key, &val);
                }
            }));
        }
        for h in hs { let _ = h.join(); }
        // File should be valid JSON map
        let path = dir_arc.join(FILE_NAME);
        let bytes = fs::read(path).unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v.is_object());
    }

    #[test]
    fn readonly_dir_gracefully_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        // Make directory read-only (0555)
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(dir, fs::Permissions::from_mode(0o555)).unwrap();
        }
        let path = dir.join(FILE_NAME);
        let res = save_map(&path, &HashMap::new());
        assert!(res.is_err());
    }

    #[test]
    fn limit_100mb_enforced() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let huge = "x".repeat(MAX_BYTES + 1);
        let mut map = HashMap::new();
        map.insert("big".to_string(), Value::String(huge));
        let res = save_map(&dir.join(FILE_NAME), &map);
        assert!(res.is_err());
    }
}

