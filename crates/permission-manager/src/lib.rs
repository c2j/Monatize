use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PermissionKind {
    Notifications,
    Geolocation,
    Camera,
    Microphone,
    ClipboardRead,
    ClipboardWrite,
}

impl PermissionKind {
    fn as_str(&self) -> &'static str {
        match self {
            PermissionKind::Notifications => "Notifications",
            PermissionKind::Geolocation => "Geolocation",
            PermissionKind::Camera => "Camera",
            PermissionKind::Microphone => "Microphone",
            PermissionKind::ClipboardRead => "ClipboardRead",
            PermissionKind::ClipboardWrite => "ClipboardWrite",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Decision {
    Allow,
    Deny,
    Prompt,
}

impl Default for Decision {
    fn default() -> Self { Decision::Prompt }
}

fn pref_key(origin: &str, kind: PermissionKind) -> String {
    format!("perms::{}::{}", origin, kind.as_str())
}

#[derive(Default)]
pub struct PermissionManager {
    // key: origin (scheme://host:port), kind -> decision
    store: HashMap<(String, PermissionKind), Decision>,
    default_decision: Decision,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self { store: HashMap::new(), default_decision: Decision::Prompt }
    }

    pub fn set_default(&mut self, decision: Decision) { self.default_decision = decision; }

    pub fn check(&self, origin: &str, kind: PermissionKind) -> Decision {
        if let Some(d) = self.store.get(&(origin.to_string(), kind)).copied() { return d; }
        // Try persisted value (skip during unit tests to avoid touching real FS)
        #[cfg(not(test))]
        {
            if let Some(d) = pref_store::get::<Decision>(&pref_key(origin, kind)) { return d; }
        }
        self.default_decision
    }

    pub fn grant(&mut self, origin: &str, kind: PermissionKind) {
        self.store.insert((origin.to_string(), kind), Decision::Allow);
        #[cfg(not(test))]
        { let _ = pref_store::set(&pref_key(origin, kind), &Decision::Allow); }
    }
    pub fn deny(&mut self, origin: &str, kind: PermissionKind) {
        self.store.insert((origin.to_string(), kind), Decision::Deny);
        #[cfg(not(test))]
        { let _ = pref_store::set(&pref_key(origin, kind), &Decision::Deny); }
    }
    pub fn revoke(&mut self, origin: &str, kind: PermissionKind) -> Option<Decision> {
        let prev = self.store.remove(&(origin.to_string(), kind));
        // Persist as Prompt to represent revoked/unknown state
        #[cfg(not(test))]
        { let _ = pref_store::set(&pref_key(origin, kind), &Decision::Prompt); }
        prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_prompt() {
        let pm = PermissionManager::new();
        assert_eq!(pm.check("https://example.com:443", PermissionKind::Notifications), Decision::Prompt);
    }

    #[test]
    fn grant_then_check_is_allow() {
        let mut pm = PermissionManager::new();
        pm.grant("https://example.com:443", PermissionKind::Notifications);
        assert_eq!(pm.check("https://example.com:443", PermissionKind::Notifications), Decision::Allow);
    }

    #[test]
    fn deny_then_check_is_deny() {
        let mut pm = PermissionManager::new();
        pm.deny("https://example.com:443", PermissionKind::Geolocation);
        assert_eq!(pm.check("https://example.com:443", PermissionKind::Geolocation), Decision::Deny);
    }

    #[test]
    fn revoke_restores_default() {
        let mut pm = PermissionManager::new();
        pm.grant("https://example.com:443", PermissionKind::ClipboardRead);
        pm.revoke("https://example.com:443", PermissionKind::ClipboardRead);
        assert_eq!(pm.check("https://example.com:443", PermissionKind::ClipboardRead), Decision::Prompt);
    }
}

