use std::env;
use std::os::unix::net::UnixStream;

use event_packet::{write_len_prefixed, Message};

fn usage() {
    println!("min-web-process [--headless] --uds <path> [--url <url>]");
}

#[cfg(feature = "port_gtk")]
mod webkit_gtk {
    use std::cell::RefCell;
    use std::rc::Rc;

    use glib::{clone, MainLoop, Continue, timeout_add_local};
    use gtk::prelude::*;
    use gtk::cairo::{ImageSurface, Format, Context};
    use webkit2gtk::{LoadEvent, WebContext, WebView};
    use webkit2gtk::WebViewExt;

    pub fn render_rgba(url: Option<&str>, w: u32, h: u32) -> Option<(Vec<u8>, u32, u32, u32)> {
        // Initialize GTK (requires a display; use xvfb-run for CI)
        if gtk::init().is_err() {
            return None;
        }
        let ctx = WebContext::default()?;
        let view = WebView::with_context(&ctx);
        let off = gtk::OffscreenWindow::new();
        off.set_default_size(w as i32, h as i32);
        off.add(&view);
        off.show_all();

        let html = "<!doctype html><meta charset='utf-8'><style>html,body{margin:0;padding:0;height:100%;background:#f00;}</style>\n<div></div>".to_string();
        if let Some(u) = url { view.load_uri(u); } else { view.load_html(&html, None); }

        // Wait until load finished
        let loop1 = MainLoop::new(None, false);
        view.connect_load_changed(clone!(@strong loop1 as loop1 => move |_, ev| {
            if ev == LoadEvent::Finished { loop1.quit(); }
        }));
        loop1.run();

        // Give GTK a moment to draw into the offscreen before capture
        let loop2 = MainLoop::new(None, false);
        timeout_add_local(std::time::Duration::from_millis(120), clone!(@strong loop2 as loop2 => move || {
            loop2.quit();
            Continue(false)
        }));
        loop2.run();

        // Extract a Pixbuf from the offscreen window
        let pb = match off.pixbuf() { Some(p) => p, None => return None };
        let pw = pb.width() as u32;
        let ph = pb.height() as u32;
        let rowstride = pb.rowstride() as usize;
        let n_channels = pb.n_channels() as usize; // 3 or 4
        let has_alpha = n_channels == 4;
        let data = pb.pixel_bytes()?.to_vec();
        let mut out = vec![0u8; (pw * ph * 4) as usize];
        for y in 0..ph as usize {
            let src_off = y * rowstride;
            for x in 0..pw as usize {
                let i = src_off + x * n_channels;
                let j = (y * pw as usize + x) * 4;
                out[j + 0] = data[i + 0];
                out[j + 1] = data[i + 1];
                out[j + 2] = data[i + 2];
                out[j + 3] = if has_alpha { data[i + 3] } else { 255 };
            }
        }
        Some((out, pw, ph, pw * 4))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
        return;
    }
    let uds_path = match args.iter().position(|a| a == "--uds").and_then(|i| args.get(i + 1)).cloned() {
        Some(p) => p,
        None => {
            usage();
            return;
        }
    };
    let headless = args.iter().any(|a| a == "--headless");
    let url = args.iter().position(|a| a == "--url").and_then(|i| args.get(i + 1)).cloned();

    let mut stream = UnixStream::connect(&uds_path).expect("connect uds");

    let (mut maybe_pixels, mut maybe_w, mut maybe_h, mut maybe_stride): (Option<Vec<u8>>, u32, u32, u32) = (None, 64, 64, 64 * 4);

    // Try WebKitGTK path when not headless (if compiled in)
    #[cfg(feature = "port_gtk")]
    if !headless {
        if let Some((p, w, h, s)) = webkit_gtk::render_rgba(url.as_deref(), 64, 64) {
            maybe_w = w; maybe_h = h; maybe_stride = s; maybe_pixels = Some(p);
        }
    }

    // Fallback stub: solid red 64x64
    let pixels = maybe_pixels.unwrap_or_else(|| {
        let w = maybe_w; let h = maybe_h;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for px in buf.chunks_exact_mut(4) { px[0] = 255; px[3] = 255; }
        buf
    });

    let msg = Message::Frame { pixels, size: (maybe_w, maybe_h), stride: maybe_stride };
    write_len_prefixed(&mut stream, &msg).expect("write frame");
    write_len_prefixed(&mut stream, &Message::Quit).expect("write quit");

    println!("min-web-process: sent frame and quit");
}


#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::path::PathBuf;

    fn has(cmd: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", cmd))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    // Minimal integration smoke test that runs the workspace E2E under Xvfb.
    // Skips when xvfb-run or python3 are missing.
    #[test]
    fn xvfb_e2e_smoke0() {
        if !has("xvfb-run") || !has("python3") {
            eprintln!("skip: xvfb-run or python3 not found");
            return;
        }
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let status = Command::new("xvfb-run")
            .current_dir(&workspace)
            .args(["-s", "-screen 0 800x600x24", "python3", "scripts/smoke-0.py"])
            .status()
            .expect("failed to run xvfb-run python3 scripts/smoke-0.py");
        assert!(status.success());
    }
}
