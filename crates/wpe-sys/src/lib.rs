#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Skeleton for WebKit Port FFI bindings (Phase 0)

/// Returns the selected port by cargo feature.
pub fn selected_port() -> &'static str {
    #[cfg(feature = "port_wpe")]
    { "wpe" }
    #[cfg(all(not(feature = "port_wpe"), feature = "port_gtk"))]
    { "gtk" }
    #[cfg(all(not(feature = "port_wpe"), not(feature = "port_gtk")))]
    { "unknown" }
}

#[cfg(test)]
mod tests {
    #[test]
    fn has_default_feature() {
        let s = super::selected_port();
        assert!(!s.is_empty());
    }
}

