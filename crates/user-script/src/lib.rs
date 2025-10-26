use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunAt { DocumentStart, DocumentEnd }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum World { Main, Isolated }

#[derive(Debug, Clone)]
pub struct UserScript {
    pub id: u64,
    pub pattern: String,  // naive: url prefix or host substring
    pub run_at: RunAt,
    pub world: World,
    pub code: String,
}

#[derive(Default, Clone)]
pub struct UserScriptRegistry {
    inner: Arc<Mutex<Inner>>, 
}

#[derive(Default)]
struct Inner {
    next_id: u64,
    scripts: HashMap<u64, UserScript>,
}

impl UserScriptRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register(&self, pattern: &str, run_at: RunAt, world: World, code: &str) -> u64 {
        let mut g = self.inner.lock().unwrap();
        g.next_id = g.next_id.wrapping_add(1);
        let id = if g.next_id == 0 { 1 } else { g.next_id };
        let us = UserScript { id, pattern: pattern.to_string(), run_at, world, code: code.to_string() };
        g.scripts.insert(id, us);
        id
    }

    pub fn remove(&self, id: u64) -> bool {
        let mut g = self.inner.lock().unwrap();
        g.scripts.remove(&id).is_some()
    }

    pub fn scripts_for_url(&self, url: &str, run_at: RunAt) -> Vec<UserScript> {
        let g = self.inner.lock().unwrap();
        let host = extract_host(url);
        let mut v: Vec<UserScript> = g.scripts.values()
            .filter(|s| s.run_at == run_at && (url.starts_with(&s.pattern) || (host.len() > 0 && host.contains(&s.pattern))))
            .cloned()
            .collect();
        // deterministic order: by id asc
        v.sort_by_key(|s| s.id);
        v
    }

    pub fn len(&self) -> usize { self.inner.lock().unwrap().scripts.len() }
}

fn extract_host(url: &str) -> String {
    // very naive URL host extractor: scheme://host[:port]/...
    if let Some(pos) = url.find("://") {
        let rest = &url[pos+3..];
        let host_end = rest.find('/').unwrap_or(rest.len());
        rest[..host_end].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_query_by_run_at() {
        let reg = UserScriptRegistry::new();
        let id1 = reg.register("https://example.com/", RunAt::DocumentStart, World::Isolated, "console.log('a')");
        let id2 = reg.register("example.com", RunAt::DocumentEnd, World::Main, "console.log('b')");
        assert_eq!(reg.len(), 2);

        let url = "https://example.com/page";
        let start = reg.scripts_for_url(url, RunAt::DocumentStart);
        assert_eq!(start.len(), 1);
        assert_eq!(start[0].id, id1);

        let end = reg.scripts_for_url(url, RunAt::DocumentEnd);
        assert_eq!(end.len(), 1);
        assert_eq!(end[0].id, id2);
    }

    #[test]
    fn remove_works() {
        let reg = UserScriptRegistry::new();
        let id = reg.register("rust-lang.org", RunAt::DocumentStart, World::Isolated, "1");
        assert_eq!(reg.len(), 1);
        assert!(reg.remove(id));
        assert_eq!(reg.len(), 0);
        assert!(!reg.remove(id));
    }
}

