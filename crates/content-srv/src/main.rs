use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
use std::path::Path;
use servo_lite::html_to_display_list;

// GTK/WebKit imports (for real rendering path)
use gtk::prelude::*;
use webkit2gtk::WebViewExt;
use gdk_pixbuf::prelude::*;

#[derive(Parser, Debug)]
#[command(name="content-srv")]
struct Args {
    /// HTML file to render via servo-lite (fallback/demo)
    #[arg(long)]
    html: Option<String>,

    /// URL to load with WebKitGTK (use --screenshot to save PNG or --show to open a window)
    #[arg(long)]
    url: Option<String>,

    /// PNG path to save real render (requires --url)
    #[arg(long)]
    screenshot: Option<String>,

    /// Show a real WebKit window (requires --url)
    #[arg(long)]
    show: bool,

    /// Headless persistent mode (offscreen, keep running) (requires --url)
    #[arg(long)]
    headless_keepalive: bool,

    /// viewport width
    #[arg(long, default_value_t = 800)]
    width: u32,
    /// viewport height
    #[arg(long, default_value_t = 600)]
    height: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Real WebKit window or headless keepalive
    if let Some(url) = args.url.as_deref() {
        if args.show {
            show_url_window(url, args.width, args.height)?;
            println!("content-srv SHOW: url={}", url);
            return Ok(());
        }
        if args.headless_keepalive {
            headless_keepalive(url, args.width, args.height)?;
            println!("content-srv HEADLESS: url={}", url);
            return Ok(());
        }
    }

    // URL + --screenshot => offscreen snapshot
    if let (Some(url), Some(out)) = (args.url.as_deref(), args.screenshot.as_deref()) {
        render_url_to_png(url, out, args.width, args.height)?;
        println!("content-srv REAL: url={} saved={}", url, out);
        return Ok(());
    }

    // Fallback: servo-lite DL path (Phase-1 demo)
    if let Some(html_path) = args.html.as_deref() {
        let html = fs::read_to_string(&html_path)?;
        let dl = html_to_display_list(&html, (args.width, args.height))?;
        println!("content-srv OK: items={}", dl.items.len());
        return Ok(());
    }

    Err(anyhow!("must provide either --url with --screenshot, or --url --show, or --url --headless-keepalive, or --html"))
}

fn render_url_to_png(url: &str, out_png: &str, width: u32, height: u32) -> Result<()> {
    if !out_png.to_lowercase().ends_with(".png") {
        return Err(anyhow!("--screenshot must be a .png path"));
    }
    if let Some(dir) = Path::new(out_png).parent() {
        if !dir.as_os_str().is_empty() { std::fs::create_dir_all(dir)?; }
    }

    // Init GTK (must be called on main thread)
    gtk::init()?;

    let off = gtk::OffscreenWindow::new();
    off.set_default_size(width as i32, height as i32);

    let webview = webkit2gtk::WebView::new();
    off.add(&webview);
    off.show_all();

    let loop_ = glib::MainLoop::new(None, false);
    let loop_clone = loop_.clone();
    let off_clone = off.clone();
    let out_path = out_png.to_string();

    webview.connect_load_changed(move |wv, ev| {
        use webkit2gtk::LoadEvent;
        if ev == LoadEvent::Finished {
            // Inject a minimal marker to verify DOM injection path
            let js = r#"document.documentElement.setAttribute('data-monazite','1')"#;
            let out2 = out_path.clone();
            let loop2 = loop_clone.clone();
            let off2 = off_clone.clone();
            // Fire-and-forget; no callback to avoid Send bounds in closure
            let _ = wv.run_javascript(js, None::<&gio::Cancellable>, |_| {});

            // Give the engine a short moment to paint, then snapshot
            glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
                if let Some(pixbuf) = off2.pixbuf() {
                    // Save as PNG
                    if let Err(e) = pixbuf.savev(&out2, "png", &[]) {
                        eprintln!("save png failed: {}", e);
                    } else {
                        println!("saved png: {}", out2);
                    }
                } else {
                    eprintln!("snapshot: pixbuf is None");
                }
                loop2.quit();
            });
        }
    });

    webview.load_uri(url);
    loop_.run();
    Ok(())
}

fn headless_keepalive(url: &str, width: u32, height: u32) -> Result<()> {
    // Require a GUI display environment (offscreen still needs a display server)
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if !has_display {
        return Err(anyhow!("No GUI display (DISPLAY/WAYLAND_DISPLAY not set). Use xvfb or --screenshot."));
    }

    gtk::init()?;

    let off = gtk::OffscreenWindow::new();
    off.set_default_size(width as i32, height as i32);

    let webview = webkit2gtk::WebView::new();
    off.add(&webview);
    off.show_all();

    let loop_ = glib::MainLoop::new(None, false);

    // Inject a minimal marker after load finished
    webview.connect_load_changed(move |wv, ev| {
        use webkit2gtk::LoadEvent;
        if ev == LoadEvent::Finished {
            let js = r#"document.documentElement.setAttribute('data-monazite','1')"#;
            let _ = wv.run_javascript(js, None::<&gio::Cancellable>, |_| {});
        }
    });

    webview.load_uri(url);
    loop_.run();
    Ok(())
}

fn show_url_window(url: &str, width: u32, height: u32) -> Result<()> {
    // Require a GUI display environment
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if !has_display {
        return Err(anyhow!("No GUI display (DISPLAY/WAYLAND_DISPLAY not set). Use --screenshot for headless."));
    }

    // Init GTK
    gtk::init()?;

    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_title("Monazite - content");
    win.set_default_size(width as i32, height as i32);

    let webview = webkit2gtk::WebView::new();
    win.add(&webview);
    win.show_all();

    // Quit loop when window is closed
    let loop_ = glib::MainLoop::new(None, false);
    let loop_clone = loop_.clone();
    win.connect_destroy(move |_| {
        loop_clone.quit();
    });

    // Inject a minimal marker after load finished
    webview.connect_load_changed(move |wv, ev| {
        use webkit2gtk::LoadEvent;
        if ev == LoadEvent::Finished {
            let js = r#"document.documentElement.setAttribute('data-monazite','1')"#;
            let _ = wv.run_javascript(js, None::<&gio::Cancellable>, |_| {});
        }
    });

    webview.load_uri(url);
    loop_.run();
    Ok(())
}
