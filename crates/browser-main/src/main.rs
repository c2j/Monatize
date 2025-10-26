use anyhow::{anyhow, Result};
use clap::Parser;
use sha2::{Digest, Sha256};
use message_defs::{HttpRequest, DisplayList, DrawCmd};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use gtk::prelude::*;
use webkit2gtk::WebViewExt;


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
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

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
        if let Err(e) = show_gui_webview(&args.url, 1024, 768) {
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
        if let DrawCmd::Rect { x, y, w, h, rgba } = *cmd {
            let (a, r, g, b) = unpack_argb_u32(rgba);
            for yy in y..y.saturating_add(h).min(height) {
                for xx in x..x.saturating_add(w).min(width) {
                    let idx = ((yy * width + xx) * 4) as usize;
                    // Alpha composite over existing pixel (very naive: src over dest)
                    let sa = a as u32;
                    let da = 255u32 - sa;
                    buf[idx + 0] = ((r as u32 * sa + buf[idx + 0] as u32 * da) / 255) as u8;
                    buf[idx + 1] = ((g as u32 * sa + buf[idx + 1] as u32 * da) / 255) as u8;
                    buf[idx + 2] = ((b as u32 * sa + buf[idx + 2] as u32 * da) / 255) as u8;
                    buf[idx + 3] = 255u8;
                }
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

fn show_gui_webview(url: &str, width: i32, height: i32) -> Result<()> {
    // Init GTK on main thread
    gtk::init()?;

    // Root window and vertical container
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_title("Monazite - Browser");
    win.set_default_size(width, height);
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    win.add(&vbox);

    // Minimal toolbar: Back | Forward | Reload
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    toolbar.set_margin_start(6);
    toolbar.set_margin_end(6);
    toolbar.set_margin_top(6);
    toolbar.set_margin_bottom(6);
    let btn_back = gtk::Button::with_label("← Back");
    let btn_fwd = gtk::Button::with_label("→ Forward");
    let btn_reload = gtk::Button::with_label("⟳ Reload");
    btn_back.set_sensitive(false);
    btn_fwd.set_sensitive(false);
    toolbar.pack_start(&btn_back, false, false, 0);
    toolbar.pack_start(&btn_fwd, false, false, 0);
    toolbar.pack_start(&btn_reload, false, false, 0);
    vbox.pack_start(&toolbar, false, false, 0);

    // WebView area
    let webview = webkit2gtk::WebView::new();
    vbox.pack_start(&webview, true, true, 0);

    // Wire buttons
    {
        let wv = webview.clone();
        btn_back.connect_clicked(move |_| { wv.go_back(); });
    }
    {
        let wv = webview.clone();
        btn_fwd.connect_clicked(move |_| { wv.go_forward(); });
    }
    {
        let wv = webview.clone();
        btn_reload.connect_clicked(move |_| { wv.reload(); });
    }

    // Update nav button sensitivity on load changes
    {
        let b_back = btn_back.clone();
        let b_fwd = btn_fwd.clone();
        webview.connect_load_changed(move |wv, _ev| {
            b_back.set_sensitive(wv.can_go_back());
            b_fwd.set_sensitive(wv.can_go_forward());
        });
    }

    win.show_all();

    // Quit loop when window is closed
    let loop_ = glib::MainLoop::new(None, false);
    let loop_clone = loop_.clone();
    win.connect_destroy(move |_| {
        loop_clone.quit();
    });

    // Inject a visible badge + data attribute after load finished
    webview.connect_load_changed(move |wv, ev| {
        use webkit2gtk::LoadEvent;
        if ev == LoadEvent::Finished {
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
        }
    });

    webview.load_uri(url);
    loop_.run();
    Ok(())
}
