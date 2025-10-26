use anyhow::{anyhow, Result};
use clap::Parser;
use clap::ValueEnum;

use sha2::{Digest, Sha256};
use message_defs::{HttpRequest, DisplayList, DrawCmd};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use gtk::prelude::*;
use webkit2gtk::WebViewExt;
use gtk::gdk::keys::constants as keys;
use gtk::gdk::ModifierType;
use extension_host::ExtensionHost;
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum PopupPolicy {
    Allow,
    Block,
    SameOrigin,
}

use extension_api::{InMemoryBus, HostMessenger, Event};


#[derive(Parser, Debug)]
#[command(name = "browser")]
struct Args {
    /// URL to open
    url: String,
    /// Save a CPU-rasterized DisplayList preview to this PPM file (Phase-1 demo)
    #[arg(long)]
    screenshot: Option<PathBuf>,
    /// Save a real WebKit render to this PNG (spawns content-srv)
    #[arg(long)]
    real_screenshot: Option<PathBuf>,
    /// Disable GUI even if display is available
    #[arg(long)]
    no_gui: bool,

    /// Show a real WebKit window (spawns content-srv; non-blocking)
    #[arg(long)]
    show: bool,

    /// Extra tabs to open at startup (repeatable)
    #[arg(long, value_name = "URL")]
    extra_tab: Vec<String>,

    /// P2 S11: stub print to PDF into given directory; exits after printing
    #[arg(long, value_name = "DIR")]
    print_stub: Option<PathBuf>,


        /// Popup/new-window policy: allow | block | same-origin
        #[arg(long, value_enum, default_value_t = PopupPolicy::Allow)]
        popup_policy: PopupPolicy,

    /// P2 S6: load extension from directory (repeatable; requires manifest.json; optional background.wat)
    #[arg(long, value_name = "DIR")]
    ext_load: Vec<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Collect extension-bus events (S7) for later processing
    let mut ext_bus_events: Vec<Event> = Vec::new();

    let args = Args::parse();

    // P2 S11: stub print to PDF and exit early when requested
    if let Some(dir) = &args.print_stub {
        let out = print_manager::print_to_pdf(dir, "smoke-2")?;
        println!("P2_S11_PRINT_OK path={}", out.display());
        return Ok(());
    }
    // P2 S6/S7: optional extension loading; log host callbacks and emit events via InMemoryBus
    if !args.ext_load.is_empty() {
        let host = ExtensionHost::new();
        let bus = InMemoryBus::new();
        let bus_arc: Arc<dyn HostMessenger> = Arc::new(bus.clone());
        host.set_bus(bus_arc);
        for dir in &args.ext_load {
            match host.load(dir) {
                Ok(id) => {
                    println!("P2_S6_LOADED id={} dir={}", id, dir.display());
                    let logs = host.logs(id);
                    for msg in logs {
                        println!("P2_S6_LOG id={} msg={}", id, msg);
                    }
                    // Also print events captured on the bus (S7)
                    for evt in bus.events() {
                        println!("P2_S7_EVT ns={} name={} payload={}", evt.ns, evt.name, evt.payload);
                    }
                }
                Err(e) => {
                    eprintln!("P2_S6_LOAD_ERR dir={} err={}", dir.display(), e);
                }
            }
        }
        // Snapshot events for later processing (e.g., tabs.create)
        ext_bus_events = bus.events();
    }


    // P2: Initialize TabManager and SiteIsolation map (skeleton integration)
    let mut tabs = tab_manager::TabManager::new();
    // P2 S5: permission check (skeleton)
    let origin = if let Some(sk) = site_isolation::SiteKey::parse(&args.url) {
        format!("{}://{}:{}", sk.scheme, sk.host, sk.port)
    } else {
        args.url.clone()
    };
    let perm_mgr = permission_manager::PermissionManager::default();
    let decision = perm_mgr.check(&origin, permission_manager::PermissionKind::Notifications);
    println!("P2_S5_CHECK origin={} kind=Notifications decision={:?}", origin, decision);

    let tab_id = tabs.new_tab(&args.url);
    let mut proc_map = site_isolation::ProcessMap::new(4);
    let pid = proc_map.allocate_process(&args.url);
    println!("P2_INIT_TAB id={} url={} pid={}", tab_id, args.url, pid);

    // P2 S7: process extension bus events (e.g., tabs.create)
    for ev in &ext_bus_events {
        if ev.ns == "tabs" && ev.name == "create" {
            let url = if ev.payload.is_empty() { "about:blank".to_string() } else { ev.payload.clone() };
            let id2 = tabs.new_tab(&url);
            println!("P2_S7_TABS_CREATED id={} url={}", id2, url);
        }
    }

    // Network fetch
    let net = network_srv::Network::new();
    let resp = net.fetch(HttpRequest { url: args.url.clone() }).await?;
    let body = String::from_utf8_lossy(&resp.body);
    println!("HTTP {} ({} bytes)", resp.status, resp.body.len());

    // AI summarize (mock)
    let summary = ai_runtime::summarize_text(&body, 16);
    println!("SUMMARY: {}", summary);

    // Servo-lite DL (Phase-1 minimal display list)
    let viewport = (800, 600);
    let dl = servo_lite::html_to_display_list(&body, viewport).unwrap_or_else(|_| {
        DisplayList { items: vec![] }
    });

    // Optional: show a real WebKit window via content-srv (non-blocking)
    if args.show {
        if let Err(e) = show_render_window_via_content_srv(&args.url) {
            eprintln!("REAL_WINDOW_ERROR: {}", e);
        } else {
            println!("REAL_WINDOW_STARTED");
        }
    }

    // Optional: real WebKit render via content-srv
    if let Some(path) = &args.real_screenshot {
        if let Err(e) = real_render_via_content_srv(&args.url, path) {
            eprintln!("REAL_SCREENSHOT_ERROR: {}", e);
        } else {
            println!("REAL_SCREENSHOT_SAVED: {}", path.display());
        }
    }

    // Default GUI: if a display is available and not explicitly disabled, show embedded WebView
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if has_display && !args.no_gui && !args.show {
        if let Err(e) = show_gui_webview(&args.url, &args.extra_tab, 1024, 768, args.popup_policy) {
            eprintln!("GUI_ERROR: {}", e);
        }
        return Ok(());
    }

    // Optional: CPU rasterize DL to PPM for preview
    if let Some(path) = &args.screenshot {
        let img = rasterize_dl_rgba8(viewport.0, viewport.1, &dl);
        write_ppm(path, viewport.0, viewport.1, &img)?;
        println!("Saved screenshot to {} (PPM)", path.display());
    }

    // GPU solid screenshot & hash (placeholder for Phase-1)
    let pixels = gpu_srv::render_solid_rgba8(640, 360, [0.2, 0.6, 0.9, 1.0])?;
    let mut hasher = Sha256::new();
    hasher.update(&pixels);
    let hex = format!("{:x}", hasher.finalize());
    println!("SCREENSHOT_SHA256: {}", hex);

    Ok(())
}

fn rasterize_dl_rgba8(width: u32, height: u32, dl: &DisplayList) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    for cmd in &dl.items {
        let DrawCmd::Rect { x, y, w, h, rgba } = *cmd;
        let (a, r, g, b) = unpack_argb_u32(rgba);
        for yy in y..y.saturating_add(h).min(height) {
            for xx in x..x.saturating_add(w).min(width) {
                let idx = ((yy * width + xx) * 4) as usize;
                let sa = a as u32;
                let da = 255u32 - sa;
                buf[idx + 0] = ((r as u32 * sa + buf[idx + 0] as u32 * da) / 255) as u8;
                buf[idx + 1] = ((g as u32 * sa + buf[idx + 1] as u32 * da) / 255) as u8;
                buf[idx + 2] = ((b as u32 * sa + buf[idx + 2] as u32 * da) / 255) as u8;
                buf[idx + 3] = 255u8;
            }
        }
    }
    buf
}

#[inline]
fn unpack_argb_u32(v: u32) -> (u8, u8, u8, u8) {
    // Phase-1 contract: 0xAARRGGBB
    let a = ((v >> 24) & 0xFF) as u8;
    let r = ((v >> 16) & 0xFF) as u8;
    let g = ((v >> 8) & 0xFF) as u8;
    let b = (v & 0xFF) as u8;
    (a, r, g, b)
}


fn real_render_via_content_srv(url: &str, out: &PathBuf) -> Result<()> {
    let out_str = out.to_string_lossy().to_string();
    // Prefer local target builds first
    let candidates = [
        "target/debug/content-srv".to_string(),
        "target/release/content-srv".to_string(),
        "content-srv".to_string(),
    ];
    for bin in &candidates {
        let status = Command::new(bin)
            .args(["--url", url, "--screenshot", &out_str])
            .status();
        match status {
            Ok(st) if st.success() => return Ok(()),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    Err(anyhow!("failed to invoke content-srv; tried {:?}", candidates))
}


fn write_ppm(path: &PathBuf, width: u32, height: u32, rgba: &[u8]) -> Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    // PPM (P6): header + raw RGB bytes; ignore alpha
    writeln!(f, "P6\n{} {}\n255", width, height)?;
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            f.write_all(&[rgba[idx], rgba[idx + 1], rgba[idx + 2]])?;
        }
    }
    Ok(())
}

fn show_render_window_via_content_srv(url: &str) -> Result<()> {
    use std::path::Path;
    // Prefer local target builds first
    let candidates = [
        "target/debug/content-srv",
        "target/release/content-srv",
        "content-srv",
    ];
    for bin in &candidates {
        if *bin != "content-srv" && !Path::new(bin).exists() {
            continue;
        }
        // Probe for --show support to avoid spawning an old binary on PATH
        let help = Command::new(bin).arg("--help").output();
        let supports_show = match help {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout);
                let e = String::from_utf8_lossy(&out.stderr);
                s.contains("--show") || e.contains("--show")
            }
            Err(_) => false,
        };
        if !supports_show {
            continue;
        }
        // Non-blocking spawn; content-srv owns the GTK window lifecycle
        Command::new(bin).args(["--url", url, "--show"]).spawn()?;
        return Ok(());
    }
    Err(anyhow!("failed to spawn content-srv with --show (no compatible binary found)"))
}

fn show_gui_webview(url: &str, extra_tabs: &[String], width: i32, height: i32, popup_policy: PopupPolicy) -> Result<()> {
    // Init GTK on main thread
    gtk::init()?;

    // Root window and vertical container
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_title("Monazite - Browser");
    win.set_default_size(width, height);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    win.add(&vbox);

    // HeaderBar: Back | Forward | Reload | [ Address bar | Go | Spinner ] | +Tab | Close Tab
    let hb = gtk::HeaderBar::new();
    hb.set_show_close_button(true);
    hb.set_title(Some("Monazite"));

    let btn_back = gtk::Button::with_label("← Back");
    let btn_fwd = gtk::Button::with_label("→ Forward");
    let btn_reload = gtk::Button::with_label("⟳ Reload");

    let entry_url = gtk::Entry::new();
    entry_url.set_placeholder_text(Some("Enter URL"));
    entry_url.set_hexpand(true);
    let btn_go = gtk::Button::with_label("Go");
    let spinner = gtk::Spinner::new();
    spinner.set_size_request(16, 16);

    let entry_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    entry_box.pack_start(&entry_url, true, true, 0);
    entry_box.pack_start(&btn_go, false, false, 0);
    entry_box.pack_start(&spinner, false, false, 0);

    let btn_new = gtk::Button::with_label("＋ Tab");
    let btn_close = gtk::Button::with_label("× Close");
    btn_back.set_sensitive(false);
    btn_fwd.set_sensitive(false);

    hb.pack_start(&btn_back);
    hb.pack_start(&btn_fwd);
    hb.pack_start(&btn_reload);
    hb.set_custom_title(Some(&entry_box));
    hb.pack_end(&btn_close);
    hb.pack_end(&btn_new);

    // Lightweight error bar (hidden by default)
    let infobar = gtk::InfoBar::new();
    infobar.set_message_type(gtk::MessageType::Error);
    let info_lbl = gtk::Label::new(None);
    infobar.content_area().add(&info_lbl);
    infobar.hide();
    vbox.pack_start(&infobar, false, false, 0);

    win.set_titlebar(Some(&hb));

    // TabStrip: StackSwitcher + Stack
    let switcher = gtk::StackSwitcher::new();
    let stack = gtk::Stack::new();
    switcher.set_stack(Some(&stack));
    switcher.set_margin_start(6);
    switcher.set_margin_end(6);
    vbox.pack_start(&switcher, false, false, 0);
    vbox.pack_start(&stack, true, true, 0);

    // P2 S3: per-site process allocation + spawn content-srv on first use
    let proc_map = std::rc::Rc::new(std::cell::RefCell::new(site_isolation::ProcessMap::new(4)));
    let children: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<site_isolation::ProcessId, std::process::Child>>> =
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
    let tab_to_pid: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, site_isolation::ProcessId>>> =
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));

    // P2 S4: minimal compositor wiring (dummy surfaces per tab)
    let compositor = std::rc::Rc::new(std::cell::RefCell::new(gpu_compositor::GpuCompositor::new()));
    let tab_to_surface_id: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, u64>>> =
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));


    // Track tabs that were opened by scripts (window.open), for window.close policy
    let script_tabs: std::rc::Rc<std::cell::RefCell<std::collections::HashSet<String>>> =
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashSet::new()));

    let win_for_add = win.clone();

    // Helper: update nav buttons for current tab + reflect URL and window title
    let update_nav = {
        let btn_back = btn_back.clone();
        let btn_fwd = btn_fwd.clone();
        let stack = stack.clone();
        let entry_for_nav = entry_url.clone();
        let win_for_nav = win.clone();
        move || {
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() {
                    btn_back.set_sensitive(wv.can_go_back());
                    btn_fwd.set_sensitive(wv.can_go_forward());
                    // Reflect current URL in the address bar
                    let current = wv.uri().map(|u| u.to_string()).unwrap_or_default();
                    entry_for_nav.set_text(&current);
                    // Update window title with page title or URL
                    let title = wv.title().map(|t| t.to_string()).unwrap_or_else(|| current.clone());
                    if title.is_empty() {
                        win_for_nav.set_title("Monazite - Browser");
                    } else {
                        win_for_nav.set_title(&format!("Monazite — {}", title));
                    }
                }
            }
        }
    };

    // Helper: add a new tab into the stack
    let tab_seq = std::rc::Rc::new(std::cell::Cell::new(0u32));
    let add_tab = {
        let stack = stack.clone();
        let update_nav_cl = update_nav.clone();
        let tab_seq = tab_seq.clone();
        let proc_map_for_add = proc_map.clone();
        let children_for_add = children.clone();
        let tab_to_pid_for_add = tab_to_pid.clone();
        let compositor_for_add = compositor.clone();
        let tab_to_surface_for_add = tab_to_surface_id.clone();
        let entry_for_add = entry_url.clone();
        move |uri: &str| -> webkit2gtk::WebView {
            let wv = webkit2gtk::WebView::new();
            // Inject badge on load finished and refresh nav sensitivity + spinner + title
            {
                let update_nav_in = update_nav_cl.clone();
                let spinner_in = spinner.clone();
                let win_for_title = win_for_add.clone();
                let stack_for_title = stack.clone();
                wv.connect_load_changed(move |wv, ev| {
                    use webkit2gtk::LoadEvent;
                    match ev {
                        LoadEvent::Started => {
                            spinner_in.start();
                            win_for_title.set_title("Monazite
 Loading...");
                        }
                        LoadEvent::Finished => {
                            // Inject a small badge once loaded
                            let js = r#"
                            (function(){
                              try{
                                var id='monazite-badge';
                                if(!document.getElementById(id)){
                                  var s=document.createElement('style');
                                  s.id='monazite-badge-style';
                                  s.textContent='#monazite-badge{position:fixed;top:12px;right:12px;z-index:2147483647;background:#111;color:#fff;font:12px/1.8 -apple-system,BlinkMacSystemFont,\"Segoe UI\",Roboto,Helvetica,Arial,sans-serif;padding:4px 8px;border-radius:12px;opacity:.9;box-shadow:0 2px 6px rgba(0,0,0,.3)}';
                                  (document.head||document.documentElement).appendChild(s);
                                  var d=document.createElement('div');
                                  d.id=id; d.textContent='Monazite';
                                  (document.body||document.documentElement).appendChild(d);
                                }
                                document.documentElement.setAttribute('data-monazite','1');
                              }catch(e){}
                            })();
                            "#;
                            let _ = wv.run_javascript(js, None::<&gio::Cancellable>, |_| {});
                            spinner_in.stop();
                            // Update title from page
                            let title = wv.title().map(|t| t.to_string()).or_else(|| wv.uri().map(|u| u.to_string())).unwrap_or_else(|| "".to_string());
                            if !title.is_empty() { win_for_title.set_title(&format!("Monazite
 {}", title)); }
                            // Update tab switcher label
                            use glib::{ToValue, Cast};
                            let w: gtk::Widget = wv.clone().upcast();
                            let _ = stack_for_title.child_set_property(&w, "title", &title.to_value());
                        }
                        _ => {}
                    }
                    // Update toolbar/nav/address bar
                    update_nav_in();
                });
            }
            // Lightweight error: show infobar on load failure
            {
                let ib = infobar.clone();
                let ib_lbl = info_lbl.clone();
                let spinner_err = spinner.clone();
                wv.connect_load_failed(move |_wv, _ev, uri, err| {
                    spinner_err.stop();
                    ib_lbl.set_text(&format!("Failed to load {}: {}", uri, err));
                    ib.show_all();
                    let ib2 = ib.clone();
                    glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                    false
                });
            }
            // Support window.open/new-window via policy decision: open in a new tab
            {
                use webkit2gtk::{PolicyDecisionExt, PolicyDecisionType, NavigationPolicyDecision, NavigationPolicyDecisionExt, URIRequestExt};
                let stack_c = stack.clone();
                let update_nav_c = update_nav_cl.clone();
                let tab_seq_c = tab_seq.clone();
                let proc_map_c = proc_map_for_add.clone();
                let children_c = children_for_add.clone();
                let tab_to_pid_c = tab_to_pid_for_add.clone();
                let compositor_c = compositor_for_add.clone();
                let tab_to_surface_c = tab_to_surface_for_add.clone();
                let entry_c = entry_for_add.clone();
                let script_tabs_c = script_tabs.clone();

                let spinner_c = spinner.clone();
                let win_c = win_for_add.clone();
                let infobar_c = infobar.clone();
                let info_lbl_c = info_lbl.clone();
                let policy_c = popup_policy;

                wv.connect_decide_policy(move |this, decision, decision_type| {
                    if decision_type == PolicyDecisionType::NewWindowAction {
                        // Try to extract target URI
                        let mut uri: Option<String> = None;
                        if let Ok(nav) = decision.clone().downcast::<NavigationPolicyDecision>() {
                            // Apply popup policy

	                            if let Some(req) = nav.request() { uri = req.uri().map(|u| u.to_string()); }

                            match policy_c {
                                PopupPolicy::Block => {
                                    let msg = if let Some(u) = uri.as_deref() { format!("Popup blocked: {}", u) } else { "Popup blocked".to_string() };
                                    info_lbl_c.set_text(&msg);
                                    infobar_c.show_all();
                                    let ib2 = infobar_c.clone();
                                    glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                                    decision.ignore();
                                    return true;
                                }
                                PopupPolicy::SameOrigin => {
                                    let allow = {
                                        let cur = this.uri().map(|u| u.to_string());
                                        match (cur.as_deref(), uri.as_deref()) {
                                            (_, None) => true,
                                            (Some(cu), Some(tu)) => {
                                                if let (Some(a), Some(b)) = (site_isolation::SiteKey::parse(cu), site_isolation::SiteKey::parse(tu)) { a == b } else { false }
                                            }
                                            _ => false
                                        }
                                    };
                                    if !allow {
                                        let msg = if let Some(u) = uri.as_deref() { format!("Popup blocked (cross-origin): {}", u) } else { "Popup blocked (cross-origin)".to_string() };
                                        info_lbl_c.set_text(&msg);
                                        infobar_c.show_all();
                                        let ib2 = infobar_c.clone();
                                        glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                                        decision.ignore();
                                        return true;
                                    }
                                }
                                PopupPolicy::Allow => {}
                            }

                        }
                        // Create new tab and navigate to uri (if any)
                        let wv2 = webkit2gtk::WebView::new();
                        let id2 = tab_seq_c.get() + 1; tab_seq_c.set(id2);
                        let name2 = format!("tab-{}", id2);
                        script_tabs_c.borrow_mut().insert(name2.clone());

                        stack_c.add_titled(&wv2, &name2, uri.as_deref().unwrap_or(""));
                        tab_to_surface_c.borrow_mut().insert(name2.clone(), id2 as u64);
                        {
                            let mut comp = compositor_c.borrow_mut();
                            comp.add_surface(id2 as u64, gpu_compositor::SurfaceHandle::new(id2 as u64, 800, 600));
                            if let Ok(n) = comp.render_frame() { let seq = comp.frames_rendered(); println!("P2_S4_FRAME seq={} surfaces={}", seq, n); }
                        }
                        // Wire load_changed for spinner/title/tab label & S3 allocation
                        {
                            let update_nav_in = update_nav_c.clone();
                            let spinner_in = spinner_c.clone();
                            let win_for_title = win_c.clone();
                            let stack_for_title = stack_c.clone();
                            let name_for_pid = name2.clone();
                            let proc_map_in = proc_map_c.clone();
                            let tab_to_pid_in = tab_to_pid_c.clone();
                            let children_in = children_c.clone();
                            wv2.connect_load_changed(move |wv, ev| {
                                use webkit2gtk::LoadEvent;
                                match ev {
                                    LoadEvent::Started => {
                                        spinner_in.start();
                                        win_for_title.set_title("Monazite — Loading...");
                                        if let Some(u) = wv.uri() {
                                            let uri = u.to_string();
                                            let pid = proc_map_in.borrow_mut().allocate_process(&uri);
                                            tab_to_pid_in.borrow_mut().insert(name_for_pid.clone(), pid);
                                            let mut ch = children_in.borrow_mut();
                                            if !ch.contains_key(&pid) {
                                                match content_srv_factory::spawn(&uri) {
                                                    Ok(child) => { println!("P2_S3_SPAWN pid={} url={}", pid, uri); ch.insert(pid, child); }
                                                    Err(e) => { eprintln!("P2_S3_SPAWN_ERR pid={} url={} err={:?}", pid, uri, e); }
                                                }
                                            } else {
                                                println!("P2_S3_REUSE pid={} url={}", pid, uri);
                                            }
                                        }
                                    }
                                    LoadEvent::Finished => {
                                        spinner_in.stop();
                                        let title = wv.title().map(|t| t.to_string()).or_else(|| wv.uri().map(|u| u.to_string())).unwrap_or_else(|| "".to_string());
                                        if !title.is_empty() { win_for_title.set_title(&format!("Monazite — {}", title)); }
                                        use glib::{ToValue, Cast};
                                        let w: gtk::Widget = wv.clone().upcast();
                                        let _ = stack_for_title.child_set_property(&w, "title", &title.to_value());
                                    }
                                    _ => {}
                                }
                                update_nav_in();
                            });
                        }
                        // Wire load_failed: show error infobar
                        {
                            let ib = infobar_c.clone();
                            let ib_lbl = info_lbl_c.clone();
                            let spinner_err = spinner_c.clone();
                            wv2.connect_load_failed(move |_wv, _ev, uri, err| {
                                spinner_err.stop();
                                ib_lbl.set_text(&format!("Failed to load {}: {}", uri, err));
                                ib.show_all();
                                let ib2 = ib.clone();
                                glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                                false
                            });
                        }
                        // Support window.close() for this new tab
                        {
                            let stack_close = stack_c.clone();
                            let proc_map_close = proc_map_c.clone();
                            let children_close = children_c.clone();
                            let tab_to_pid_close = tab_to_pid_c.clone();
                            let compositor_close = compositor_c.clone();
                            let tab_to_surface_close = tab_to_surface_c.clone();
                            let name_close = name2.clone();
                            let wv2_for_close = wv2.clone();
                            let infobar_close = infobar_c.clone();
                            let info_lbl_close = info_lbl_c.clone();
                            let script_tabs_close = script_tabs_c.clone();
                            wv2.connect_close(move |_wv| {
                                if !script_tabs_close.borrow().contains(&name_close) {
                                    info_lbl_close.set_text("Blocked window.close() (not script-opened)");
                                    infobar_close.show_all();
                                    let ib2 = infobar_close.clone();
                                    glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                                    return;
                                }
                                script_tabs_close.borrow_mut().remove(&name_close);
                                if let Some(pid) = tab_to_pid_close.borrow_mut().remove(&name_close) {
                                    let remaining = proc_map_close.borrow_mut().release_pid(pid);
                                    if remaining == 0 {
                                        if let Some(mut child) = children_close.borrow_mut().remove(&pid) {
                                            let _ = content_srv_factory::kill(&mut child);
                                            println!("P2_S3_KILL pid={} name={}", pid, name_close);
                                        }
                                    } else {
                                        println!("P2_S3_DEC pid={} remaining={}", pid, remaining);
                                    }
                                }
                                if let Some(sid) = tab_to_surface_close.borrow_mut().remove(&name_close) {
                                    let mut comp = compositor_close.borrow_mut();
                                    let _ = comp.remove_surface(sid);
                                    if let Ok(n) = comp.render_frame() {
                                        let seq = comp.frames_rendered();
                                        println!("P2_S4_FRAME seq={} surfaces={}", seq, n);
                                    }
                                }
                                let w: gtk::Widget = wv2_for_close.clone().upcast();
                                stack_close.remove(&w);
                            });
                        }

                        stack_c.set_visible_child(&wv2);
                        if let Some(u) = uri.as_deref() { entry_c.set_text(u); wv2.load_uri(u); } else { entry_c.set_text(""); }
                        update_nav_c();
                        decision.ignore(); // we will handle by our own navigation
                        return true;
                    }
                    false
                });
            }

            let id = tab_seq.get() + 1; tab_seq.set(id);
            let name = format!("tab-{}", id);
            stack.add_titled(&wv, &name, uri);

            // Support window.close() for this tab (blocked unless script-opened)
            {
                let stack_close = stack.clone();
                let proc_map_close = proc_map_for_add.clone();
                let children_close = children_for_add.clone();
                let tab_to_pid_close = tab_to_pid_for_add.clone();
                let compositor_close = compositor_for_add.clone();
                let tab_to_surface_close = tab_to_surface_for_add.clone();
                let name_close = name.clone();
                let wv_for_close = wv.clone();
                let infobar_close = infobar.clone();
                let info_lbl_close = info_lbl.clone();
                let script_tabs_close = script_tabs.clone();
                wv.connect_close(move |_wv| {
                    if !script_tabs_close.borrow().contains(&name_close) {
                        info_lbl_close.set_text("Blocked window.close() (not script-opened)");
                        infobar_close.show_all();
                        let ib2 = infobar_close.clone();
                        glib::timeout_add_seconds_local(3, move || { ib2.hide(); glib::Continue(false) });
                        return;
                    }
                    script_tabs_close.borrow_mut().remove(&name_close);
                    if let Some(pid) = tab_to_pid_close.borrow_mut().remove(&name_close) {
                        let remaining = proc_map_close.borrow_mut().release_pid(pid);
                        if remaining == 0 {
                            if let Some(mut child) = children_close.borrow_mut().remove(&pid) {
                                let _ = content_srv_factory::kill(&mut child);
                                println!("P2_S3_KILL pid={} name={}", pid, name_close);
                            }
                        } else {
                            println!("P2_S3_DEC pid={} remaining={}", pid, remaining);
                        }
                    }
                    if let Some(sid) = tab_to_surface_close.borrow_mut().remove(&name_close) {
                        let mut comp = compositor_close.borrow_mut();
                        let _ = comp.remove_surface(sid);
                        if let Ok(n) = comp.render_frame() {
                            let seq = comp.frames_rendered();
                            println!("P2_S4_FRAME seq={} surfaces={}", seq, n);
                        }
                    }
                    let w: gtk::Widget = wv_for_close.clone().upcast();
                    stack_close.remove(&w);
                });
            }


            // S4 wiring: add a dummy surface per tab and render a frame
            tab_to_surface_for_add.borrow_mut().insert(name.clone(), id as u64);
            {
                let mut comp = compositor_for_add.borrow_mut();
                comp.add_surface(id as u64, gpu_compositor::SurfaceHandle::new(id as u64, 800, 600));
                if let Ok(n) = comp.render_frame() {
                    let seq = comp.frames_rendered();
                    println!("P2_S4_FRAME seq={} surfaces={}", seq, n);
                }
            }

            // S3 wiring: allocate per-site process and spawn if first time
            {
                let pid = proc_map_for_add.borrow_mut().allocate_process(uri);
                tab_to_pid_for_add.borrow_mut().insert(name.clone(), pid);
                let mut ch = children_for_add.borrow_mut();
                if !ch.contains_key(&pid) {
                    match content_srv_factory::spawn(uri) {
                        Ok(child) => {
                            println!("P2_S3_SPAWN pid={} url={}", pid, uri);
                            ch.insert(pid, child);
                        }
                        Err(e) => {
                            eprintln!("P2_S3_SPAWN_ERR pid={} url={} err={:?}", pid, uri, e);
                        }
                    }
                } else {
                    println!("P2_S3_REUSE pid={} url={}", pid, uri);
                }
            }

            // Set address bar and navigate
            entry_for_add.set_text(uri);
            wv.load_uri(uri);
            stack.set_visible_child(&wv);
            wv.clone()

        }
    };

    // Wire toolbar to act on current tab
    {
        let stack = stack.clone();
        btn_back.connect_clicked(move |_| {
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() { wv.go_back(); }
            }
        });
    }
    {
        let stack = stack.clone();
        btn_fwd.connect_clicked(move |_| {
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() { wv.go_forward(); }
            }
        });
    }
    {
        let stack = stack.clone();
        btn_reload.connect_clicked(move |_| {
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() { wv.reload(); }
            }
        });


    }
    // Address bar: Go button and Enter key to navigate current tab
    {
        let stack = stack.clone();
        let entry = entry_url.clone();
        btn_go.connect_clicked(move |_| {
            let mut uri = entry.text().to_string();
            if !(uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("about:") || uri.starts_with("file://")) {
                uri = format!("https://{}", uri);
            }
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() { wv.load_uri(&uri); }
            }
        });
    }
    {
        let stack = stack.clone();
        let entry = entry_url.clone();
        entry_url.connect_activate(move |_| {
            let mut uri = entry.text().to_string();
            if !(uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("about:") || uri.starts_with("file://")) {
                uri = format!("https://{}", uri);
            }
            if let Some(child) = stack.visible_child() {
                if let Ok(wv) = child.downcast::<webkit2gtk::WebView>() { wv.load_uri(&uri); }
            }
        });
    }


    // Add initial tab before wiring +Tab button
    add_tab(url);
    for t in extra_tabs { add_tab(t); }


    // +Tab button: open about:blank
    {
        btn_new.connect_clicked(move |_| { add_tab("about:blank"); });
    }

    // Change of visible tab updates nav buttons
    {
        let update_nav = update_nav.clone();
    // Close current tab: release pid and kill content-srv when no load remains
    {
        let stack = stack.clone();
        let proc_map = proc_map.clone();
        let children = children.clone();
        let tab_to_pid = tab_to_pid.clone();
        let compositor = compositor.clone();
        let tab_to_surface = tab_to_surface_id.clone();
        btn_close.connect_clicked(move |_| {
            if let Some(name) = stack.visible_child_name() {
                let name_str = name.as_str().to_string();
                if let Some(pid) = tab_to_pid.borrow_mut().remove(&name_str) {
                    let remaining = proc_map.borrow_mut().release_pid(pid);
                    if remaining == 0 {
                        if let Some(mut child) = children.borrow_mut().remove(&pid) {
                            let _ = content_srv_factory::kill(&mut child);
                            println!("P2_S3_KILL pid={} name={}", pid, name_str);
                        }
                    } else {
                        println!("P2_S3_DEC pid={} remaining={}", pid, remaining);
                    }
                }
                if let Some(sid) = tab_to_surface.borrow_mut().remove(&name_str) {
                    let mut comp = compositor.borrow_mut();
                    let _ = comp.remove_surface(sid);
                    if let Ok(n) = comp.render_frame() {
                        let seq = comp.frames_rendered();
                        println!("P2_S4_FRAME seq={} surfaces={}", seq, n);
                    }
                }
                if let Some(widget) = stack.visible_child() {
                    stack.remove(&widget);
                }
            }
        });
    }
    // Keyboard shortcuts: Ctrl+L (focus URL), Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+R (reload)
    {
        let entry = entry_url.clone();
        let btn_new_k = btn_new.clone();
        let btn_close_k = btn_close.clone();
        let btn_reload_k = btn_reload.clone();
        win.connect_key_press_event(move |_, ev| {
            let state = ev.state();
            let key = ev.keyval();
            if state.contains(ModifierType::CONTROL_MASK) {
                if key == keys::L {
                    entry.grab_focus();
                    let len = entry.text().len() as i32;
                    entry.select_region(0, len);
                    return gtk::Inhibit(true);
                } else if key == keys::T {
                    btn_new_k.clicked();
                    return gtk::Inhibit(true);
                } else if key == keys::W {
                    btn_close_k.clicked();
                    return gtk::Inhibit(true);
                } else if key == keys::R {
                    btn_reload_k.clicked();
                    return gtk::Inhibit(true);
                }
            }
            gtk::Inhibit(false)
        });
    }


        stack.connect_visible_child_notify(move |_| { update_nav(); });
    }

    win.show_all();

    // Quit loop when window is closed
    let loop_ = glib::MainLoop::new(None, false);
    let loop_clone = loop_.clone();
    win.connect_destroy(move |_| { loop_clone.quit(); });

    loop_.run();
    Ok(())
}
