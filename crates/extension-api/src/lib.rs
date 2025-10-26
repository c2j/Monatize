use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub ns: String,     // e.g., "tabs", "runtime"
    pub name: String,   // e.g., "onCreated"
    pub payload: String // JSON string or freeform for skeleton
}

impl Event {
    pub fn new(ns: &str, name: &str, payload: &str) -> Self {
        Self { ns: ns.to_string(), name: name.to_string(), payload: payload.to_string() }
    }
}

pub trait HostMessenger: Send + Sync {
    fn post(&self, evt: &Event) -> Result<(), String>;
}

#[derive(Default, Clone)]
pub struct InMemoryBus {
    inner: Arc<Mutex<Vec<Event>>>,
}

impl InMemoryBus {
    pub fn new() -> Self { Self::default() }
    pub fn len(&self) -> usize { self.inner.lock().unwrap().len() }
    pub fn events(&self) -> Vec<Event> { self.inner.lock().unwrap().clone() }
}

impl HostMessenger for InMemoryBus {
    fn post(&self, evt: &Event) -> Result<(), String> {
        self.inner.lock().unwrap().push(evt.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_many_events() {
        let bus = InMemoryBus::new();
        let n = 50;
        for i in 0..n {
            let e = Event::new("runtime", "onMessage", &format!("{i}"));
            bus.post(&e).unwrap();
        }
        assert_eq!(bus.len(), n as usize);
        let evts = bus.events();
        assert_eq!(evts.first().unwrap().payload, "0");
        assert_eq!(evts.last().unwrap().payload, "49");
    }
}

