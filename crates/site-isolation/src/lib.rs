use std::collections::HashMap;

pub type ProcessId = u32;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct SiteKey {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl SiteKey {
    pub fn parse(url: &str) -> Option<Self> {
        // Extremely small parser to avoid external deps: scheme://host[:port]/...
        let lower = url.trim();
        let mut scheme = "";
        let mut rest = lower;
        if let Some(i) = lower.find("://") {
            scheme = &lower[..i];
            rest = &lower[i+3..];
        }
        // host[:port][/...]
        let host_port = rest.split('/').next().unwrap_or("");
        let mut host = host_port;
        let mut port: u16 = match scheme {
            "http" => 80,
            "https" => 443,
            _ => 0,
        };
        if let Some(j) = host_port.rfind(':') {
            if host_port[j+1..].chars().all(|c| c.is_ascii_digit()) {
                host = &host_port[..j];
                if let Ok(p) = host_port[j+1..].parse() { port = p; }
            }
        }
        if scheme.is_empty() || host.is_empty() { return None; }
        Some(SiteKey { scheme: scheme.to_ascii_lowercase(), host: host.to_ascii_lowercase(), port })
    }
}

#[derive(Default)]
pub struct ProcessMap {
    // Map site -> pid
    map: HashMap<SiteKey, ProcessId>,
    // Simple load counter per pid for least-busy reuse
    load: HashMap<ProcessId, u32>,
    max_processes: usize,
    next_pid: ProcessId,
}

impl ProcessMap {
    pub fn new(max_processes: usize) -> Self {
        Self { map: HashMap::new(), load: HashMap::new(), max_processes, next_pid: 1 }
    }

    pub fn allocate_process(&mut self, url: &str) -> ProcessId {
        let site = SiteKey::parse(url).unwrap_or(SiteKey { scheme: "http".into(), host: url.to_string(), port: 0 });
        if let Some(pid) = self.map.get(&site).copied() {
            *self.load.entry(pid).or_insert(0) += 1;
            return pid;
        }
        // Need a new or reused process
        let pid = if self.load.len() < self.max_processes {
            let pid = self.next_pid;
            self.next_pid = self.next_pid.wrapping_add(1).max(1);
            pid
        } else {
            // Find least-loaded pid
            let (pid, _) = self.load.iter().min_by_key(|(_p, l)| **l).map(|(p, l)| (*p, *l)).unwrap();
            pid
        };
        self.map.insert(site, pid);
        *self.load.entry(pid).or_insert(0) += 1;
        pid
    }

    /// Decrease load counter for a pid after a tab is closed.
    /// Returns the remaining load for that pid (clamped at 0).
    pub fn release_pid(&mut self, pid: ProcessId) -> u32 {
        if let Some(entry) = self.load.get_mut(&pid) {
            if *entry > 0 { *entry -= 1; }
            *entry
        } else {
            0
        }
    }
}

/// Spec-compatible helper: allocate per-site process from a `url::Url`.
/// This keeps the public API stable for Phase-2 while retaining the internal map.
pub fn allocate_process(url: &url::Url, map: &mut ProcessMap) -> ProcessId {
    let scheme = url.scheme().to_ascii_lowercase();
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    let port = url.port_or_known_default().unwrap_or(0);
    let site = SiteKey { scheme, host, port };

    if let Some(&pid) = map.map.get(&site) {
        *map.load.entry(pid).or_insert(0) += 1;
        return pid;
    }

    let pid = if map.load.len() < map.max_processes {
        let pid = map.next_pid;
        map.next_pid = map.next_pid.wrapping_add(1).max(1);
        pid
    } else {
        let (pid, _) = map
            .load
            .iter()
            .min_by_key(|(_p, l)| **l)
            .map(|(p, l)| (*p, *l))
            .unwrap();
        pid
    };
    map.map.insert(site, pid);
    *map.load.entry(pid).or_insert(0) += 1;
    pid
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_sites_get_different_pids_until_cap() {
        let mut pm = ProcessMap::new(2);
        let p1 = pm.allocate_process("https://a.com/");
        let p2 = pm.allocate_process("https://b.com/");
        assert_ne!(p1, p2);
        // Third unique site should reuse least loaded (either p1 or p2)
        let p3 = pm.allocate_process("https://c.com/");
        assert!(p3 == p1 || p3 == p2);
    }

    #[test]
    fn same_site_reuses() {
        let mut pm = ProcessMap::new(4);
        let p1 = pm.allocate_process("http://example.com:8080/page");
        let p2 = pm.allocate_process("http://example.com:8080/other");
        assert_eq!(p1, p2);
    }
}

