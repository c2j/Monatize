//! S12 update-service (stub)
//! Minimal API to satisfy Phase-2 spec.

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateError {
    Unsupported,
}
impl core::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self { UpdateError::Unsupported => write!(f, "update not supported in stub"), }
    }
}
impl std::error::Error for UpdateError {}

pub fn check_update() -> Result<UpdateInfo, UpdateError> {
    Ok(UpdateInfo { version: "0.0.0-stub".to_string() })
}

pub fn apply_patch(_patch: &[u8]) -> Result<(), UpdateError> {
    // Stub: pretend to apply successfully
    Ok(())
}

