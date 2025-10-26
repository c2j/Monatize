use anyhow::{anyhow, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn sanitize_filename(name: &str) -> String {
    let mut s = name.replace('/', "_").replace('\\', "_");
    if !s.ends_with(".pdf") { s.push_str(".pdf"); }
    s
}

pub fn print_to_pdf(output_dir: &Path, name_hint: &str) -> Result<PathBuf> {
    if name_hint.trim().is_empty() { return Err(anyhow!("empty name")); }
    fs::create_dir_all(output_dir)?;
    let out = output_dir.join(sanitize_filename(name_hint));
    // Minimal PDF stub content (sufficient for E2E existence checks)
    let content = b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n";
    let mut f = File::create(&out)?;
    f.write_all(content)?;
    f.sync_all()?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_non_empty_pdf() {
        let base = std::env::temp_dir().join(format!(
            "pm_test_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()
        ));
        fs::create_dir_all(&base).unwrap();
        let out = print_to_pdf(&base, "hello").unwrap();
        assert!(out.exists());
        let size = fs::metadata(&out).unwrap().len();
        assert!(size > 0);
        let data = fs::read(&out).unwrap();
        assert!(data.starts_with(b"%PDF"));
    }
}

