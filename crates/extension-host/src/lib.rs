use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};
use std::fs;
use wasmi::{Caller, Engine, Instance, Linker, Module, Store};
use extension_api::{Event, HostMessenger};

pub type ExtensionId = u64;

#[derive(Debug, Clone)]
pub struct InstalledExt {
    pub id: ExtensionId,
    pub dir: PathBuf,
    pub name: String,
}

struct HostState { logs: Vec<String>, bus: Option<Arc<dyn HostMessenger>> }
struct Runtime { store: Store<HostState>, instance: Instance }

#[derive(Default)]
struct Inner {
    next_id: ExtensionId,
    exts: HashMap<ExtensionId, InstalledExt>,
    runtime: HashMap<ExtensionId, Runtime>,
    bus: Option<Arc<dyn HostMessenger>>, // Optional event bus
}

#[derive(Default, Clone)]
pub struct ExtensionHost {
    inner: Arc<Mutex<Inner>>,
}

impl ExtensionHost {
    /// Set an optional event bus used to emit extension events.
    pub fn set_bus(&self, bus: Arc<dyn HostMessenger>) {
        let mut g = self.inner.lock().unwrap();
        g.bus = Some(bus);
    }

    /// Create a host pre-wired with the given event bus.
    pub fn with_bus(bus: Arc<dyn HostMessenger>) -> Self {
        let this = Self::new();
        this.set_bus(bus);
        this
    }

    pub fn new() -> Self { Self::default() }

    /// Load an extension from a directory containing a manifest.json.
    /// If background.wat exists, instantiate a WASM runtime and call exported `hello` or `init`.
    pub fn load(&self, manifest_dir: &Path) -> Result<ExtensionId, String> {
        if !manifest_dir.is_dir() { return Err("not a directory".into()); }
        let manifest = manifest_dir.join("manifest.json");
        if !manifest.exists() { return Err("manifest.json not found".into()); }
        let name = manifest_dir.file_name().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string();
        let mut g = self.inner.lock().unwrap();
        g.next_id = g.next_id.wrapping_add(1);
        let id = if g.next_id == 0 { 1 } else { g.next_id };
        let inst = InstalledExt { id, dir: manifest_dir.to_path_buf(), name };
        g.exts.insert(id, inst);
        drop(g);
        let bus = { self.inner.lock().unwrap().bus.clone() };
        if let Ok(Some(mut rt)) = Self::init_runtime(manifest_dir, bus) {
            let mut g = self.inner.lock().unwrap();
            Self::call_export(&mut rt, "hello");
            Self::call_export(&mut rt, "init");
            g.runtime.insert(id, rt);
        }
        Ok(id)
    }

    pub fn unload(&self, id: ExtensionId) -> Result<(), String> {
        let mut g = self.inner.lock().unwrap();
        let existed = g.exts.remove(&id).is_some();
        g.runtime.remove(&id);
        if existed { Ok(()) } else { Err("unknown extension id".into()) }
    }

    pub fn is_loaded(&self, id: ExtensionId) -> bool {
        let g = self.inner.lock().unwrap();
        g.exts.contains_key(&id)
    }
    pub fn list_ids(&self) -> Vec<ExtensionId> {
        let g = self.inner.lock().unwrap();
        g.exts.keys().copied().collect()
    }

    pub fn logs(&self, id: ExtensionId) -> Vec<String> {
        let g = self.inner.lock().unwrap();
        if let Some(rt) = g.runtime.get(&id) {
            rt.store.data().logs.clone()
        } else { Vec::new() }
    }

    fn init_runtime(manifest_dir: &Path, bus: Option<Arc<dyn HostMessenger>>) -> Result<Option<Runtime>, String> {
        let wat_path = manifest_dir.join("background.wat");
        if !wat_path.exists() { return Ok(None); }
        let wat = fs::read_to_string(&wat_path).map_err(|e| e.to_string())?;
        let engine = Engine::default();
        let module = Module::new(&engine, &wat).map_err(|e| e.to_string())?;
        let mut store = Store::new(&engine, HostState { logs: Vec::new(), bus });
        let mut linker: Linker<HostState> = Linker::new(&engine);
        // host.hello(i32): demo callback + event
        linker.func_wrap("host", "hello", |mut caller: Caller<'_, HostState>, n: i32| {
            let payload = format!("hello({n})");
            let bus = caller.data().bus.clone();
            caller.data_mut().logs.push(payload.clone());
            if let Some(b) = bus {
                let _ = b.post(&Event::new("runtime", "hello", &payload));
            }
        }).map_err(|e| e.to_string())?;
        // host.log(i32): logs + runtime.log event
        linker.func_wrap("host", "log", |mut caller: Caller<'_, HostState>, n: i32| {
            let payload = format!("log({n})");
            let bus = caller.data().bus.clone();
            caller.data_mut().logs.push(payload.clone());
            if let Some(b) = bus {
                let _ = b.post(&Event::new("runtime", "log", &payload));
            }
        }).map_err(|e| e.to_string())?;
        // host.emit_event(i32): generic runtime.emit event with code payload
        linker.func_wrap("host", "emit_event", |caller: Caller<'_, HostState>, code: i32| {
            if let Some(b) = caller.data().bus.clone() {
                let _ = b.post(&Event::new("runtime", "emit", &format!("code({})", code)));
            }
        }).map_err(|e| e.to_string())?;
        // host.post_message(i32): runtime.message event
        linker.func_wrap("host", "post_message", |caller: Caller<'_, HostState>, msg: i32| {
            if let Some(b) = caller.data().bus.clone() {
                let _ = b.post(&Event::new("runtime", "message", &format!("msg({})", msg)));
            }
        }).map_err(|e| e.to_string())?;
        // host.set_timeout(i32 delay_ms, i32 id): spawn timer thread and emit runtime.timeout(id)
        linker.func_wrap("host", "set_timeout", |caller: Caller<'_, HostState>, delay_ms: i32, id: i32| {
            if let Some(b) = caller.data().bus.clone() {
                let delay = if delay_ms < 0 { 0 } else { delay_ms as u64 };
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                    let _ = b.post(&Event::new("runtime", "timeout", &format!("id({})", id)));
                });
            }
        }).map_err(|e| e.to_string())?;
        // host.tabs_create(): minimal tabs.create event
        linker.func_wrap("host", "tabs_create", |caller: Caller<'_, HostState>| {
            if let Some(b) = caller.data().bus.clone() {
                let _ = b.post(&Event::new("tabs", "create", "about:blank"));
            }
        }).map_err(|e| e.to_string())?;
        let instance = linker.instantiate_and_start(&mut store, &module)
            .map_err(|e| e.to_string())?;
        Ok(Some(Runtime { store, instance }))
    }

    fn call_export(rt: &mut Runtime, name: &str) {
        if let Ok(func) = rt.instance.get_typed_func::<(), ()>(&rt.store, name) {
            let _ = func.call(&mut rt.store, ());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mktemp_dir(prefix: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("{}_{}", prefix, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn load_unload_basic() {
        let tmp = mktemp_dir("ext_host");
        fs::write(tmp.join("manifest.json"), b"{\n  \"manifest_version\":3, \"name\":\"Demo\"\n}\n").unwrap();
        let host = ExtensionHost::new();
        let id = host.load(&tmp).expect("load ok");
        assert!(host.is_loaded(id));
        assert!(host.list_ids().contains(&id));
        host.unload(id).expect("unload ok");
        assert!(!host.is_loaded(id));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_without_manifest_fails() {
        let tmp = mktemp_dir("ext_host_nomani");
        let host = ExtensionHost::new();
        let err = host.load(&tmp).unwrap_err();
        assert!(err.contains("manifest.json"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn wasm_hello_calls_host() {
        let tmp = mktemp_dir("ext_host_wasm_ok");
        fs::write(tmp.join("manifest.json"), "{\"manifest_version\":3,\"name\":\"Demo\"}\n").unwrap();
        let wat = r#"(module (import "host" "hello" (func $h (param i32))) (func (export "hello") (call $h (i32.const 7))))"#;
        fs::write(tmp.join("background.wat"), wat).unwrap();
        let host = ExtensionHost::new();
        let id = host.load(&tmp).unwrap();
        let logs = host.logs(id);
        assert!(logs.iter().any(|s| s.contains("hello(7)")), "logs={:?}", logs);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn wasm_tabs_create_posts_event() {
        let tmp = mktemp_dir("ext_host_tabs_create");
        fs::write(tmp.join("manifest.json"), "{\"manifest_version\":3,\"name\":\"Demo\"}\n").unwrap();
        let wat = r#"(module (import "host" "tabs_create" (func $c)) (func (export "hello") (call $c)))"#;
        fs::write(tmp.join("background.wat"), wat).unwrap();
        let host = ExtensionHost::new();
        let bus = extension_api::InMemoryBus::new();
        host.set_bus(Arc::new(bus.clone()));
        let _ = host.load(&tmp).unwrap();
        let events = bus.events();
        assert!(events.iter().any(|e| e.ns == "tabs" && e.name == "create"), "events={:?}", events);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn wasm_post_message_posts_event() {
        let tmp = mktemp_dir("ext_host_post_msg");
        fs::write(tmp.join("manifest.json"), "{\"manifest_version\":3,\"name\":\"Demo\"}\n").unwrap();
        let wat = r#"(module (import "host" "post_message" (func $pm (param i32))) (func (export "hello") (call $pm (i32.const 123))))"#;
        fs::write(tmp.join("background.wat"), wat).unwrap();
        let host = ExtensionHost::new();
        let bus = extension_api::InMemoryBus::new();
        host.set_bus(Arc::new(bus.clone()));
        let _ = host.load(&tmp).unwrap();
        let events = bus.events();
        assert!(events.iter().any(|e| e.ns == "runtime" && e.name == "message" && e.payload.contains("msg(123)")), "events={:?}", events);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn wasm_set_timeout_posts_event() {
        let tmp = mktemp_dir("ext_host_timeout");
        fs::write(tmp.join("manifest.json"), "{\"manifest_version\":3,\"name\":\"Demo\"}\n").unwrap();
        let wat = r#"(module (import "host" "set_timeout" (func $st (param i32 i32))) (func (export "hello") (call $st (i32.const 10) (i32.const 7))))"#;
        fs::write(tmp.join("background.wat"), wat).unwrap();
        let host = ExtensionHost::new();
        let bus = extension_api::InMemoryBus::new();
        host.set_bus(Arc::new(bus.clone()));
        let _ = host.load(&tmp).unwrap();
        // Wait for timer to fire
        std::thread::sleep(std::time::Duration::from_millis(100));
        let events = bus.events();
        assert!(events.iter().any(|e| e.ns == "runtime" && e.name == "timeout" && e.payload.contains("id(7)")), "events={:?}", events);
        let _ = fs::remove_dir_all(&tmp);
    }



    #[test]
    fn wasm_trap_no_crash() {
        let tmp = mktemp_dir("ext_host_wasm_trap");
        fs::write(tmp.join("manifest.json"), "{\"manifest_version\":3,\"name\":\"Demo\"}\n").unwrap();
        let wat = r#"(module (func (export "hello") (i32.const 1) (i32.const 0) (i32.div_s) drop))"#;
        fs::write(tmp.join("background.wat"), wat).unwrap();
        let host = ExtensionHost::new();
        let id = host.load(&tmp).unwrap();
        assert!(host.is_loaded(id));
        let _ = fs::remove_dir_all(&tmp);
    }
}
