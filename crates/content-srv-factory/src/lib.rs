use anyhow::Result;
use std::process::{Child, Command, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("no compatible content-srv binary found")] 
    NotFound,
    #[error("spawn failed: {0}")] 
    Io(#[from] std::io::Error),
}

fn resolve_content_srv() -> Option<String> {
    let candidates = [
        "target/debug/content-srv",
        "target/release/content-srv",
        "content-srv",
    ];
    for bin in candidates {
        if bin == "content-srv" {
            // Probe for --help so we don't accidentally spawn an incompatible binary on PATH
            if let Ok(out) = Command::new(bin).arg("--help").stdout(Stdio::null()).stderr(Stdio::null()).output() {
                let s_ok = out.status.success();
                if s_ok { return Some(bin.to_string()); }
            }
        } else if std::path::Path::new(bin).exists() {
            return Some(bin.to_string());
        }
    }
    None
}

/// Spawn a real-rendering content process for a given URL.
/// Show window only when explicitly enabled via env MONAZITE_SHOW_CONTENT=1/true/yes/on
/// and when a GUI display is present. Default is headless/no-window.
pub fn spawn(url: &str) -> std::result::Result<Child, SpawnError> {
    let bin = resolve_content_srv().ok_or(SpawnError::NotFound)?;
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    let show_content = std::env::var("MONAZITE_SHOW_CONTENT")
        .ok()
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);

    let mut cmd = Command::new(bin);
    cmd.args(["--url", url]);
    if has_display {
        if show_content {
            cmd.arg("--show");
        } else {
            cmd.arg("--headless-keepalive");
        }
    }
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let child = cmd.spawn()?;
    Ok(child)
}

/// Kill a spawned content process.
pub fn kill(child: &mut Child) -> Result<()> {
    // Try terminate gracefully (future: SIGTERM) then kill
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests are ignored by default due to GUI/headless environment constraints
    #[test]
    #[ignore]
    fn spawn_and_kill_works() {
        let mut ch = spawn("https://example.com").expect("spawn content-srv");
        kill(&mut ch).unwrap();
    }
}

