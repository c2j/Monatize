pub type TabId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TabState {
    pub url: String,
    pub title: String,
    pub frozen: bool,
}

#[derive(Default)]
pub struct TabManager {
    next_id: TabId,
    tabs: std::collections::BTreeMap<TabId, TabState>,
}

impl TabManager {
    pub fn new() -> Self {
        Self { next_id: 1, tabs: Default::default() }
    }

    pub fn len(&self) -> usize { self.tabs.len() }

    pub fn new_tab(&mut self, url: &str) -> TabId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.tabs.insert(id, TabState { url: url.to_string(), title: String::new(), frozen: false });
        id
    }

    pub fn close_tab(&mut self, id: TabId) -> anyhow::Result<()> {
        if self.tabs.remove(&id).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("tab not found: {}", id))
        }
    }

    pub fn freeze_tab(&mut self, id: TabId) -> anyhow::Result<TabState> {
        let st = self.tabs.get_mut(&id).ok_or_else(|| anyhow::anyhow!("tab not found: {}", id))?;
        st.frozen = true;
        Ok(st.clone())
    }

    pub fn get(&self, id: TabId) -> Option<&TabState> { self.tabs.get(&id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lifecycle() {
        let mut tm = TabManager::new();
        let a = tm.new_tab("https://a.com");
        let b = tm.new_tab("https://b.com");
        assert_ne!(a, b);
        assert_eq!(tm.len(), 2);
        assert_eq!(tm.get(a).unwrap().frozen, false);
        let s = tm.freeze_tab(a).unwrap();
        assert!(s.frozen);
        tm.close_tab(a).unwrap();
        assert_eq!(tm.len(), 1);
        assert!(tm.close_tab(a).is_err());
    }

    #[test]
    fn id_wraps_but_not_zero() {
        let mut tm = TabManager { next_id: u64::MAX, tabs: Default::default() };
        let id1 = tm.new_tab("x");
        let id2 = tm.new_tab("y");
        assert_eq!(id1, u64::MAX);
        assert_eq!(id2, 1);
    }
}

