//! S13 crash-reporter (stub)

#[derive(Debug, Clone, Copy)]
pub struct InitError;

impl core::fmt::Display for InitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "init failed") }
}
impl std::error::Error for InitError {}

pub fn init(upload: Option<&str>) -> Result<(), InitError> {
    // Stub: just log a marker; a real impl would set signal/handler and write minidumps
    println!("P2_S13_CRASH_INIT upload={}", upload.unwrap_or("none"));
    Ok(())
}

