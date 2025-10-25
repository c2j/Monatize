#![forbid(unsafe_code)]
//! Static HTML source for Phase 0 (alternates red/blue)

use std::sync::atomic::{AtomicUsize, Ordering};
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Returns alternating red/blue full-screen HTML pages.
pub fn next_html() -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    if n % 2 == 0 {
        html_with_color("red")
    } else {
        html_with_color("blue")
    }
}

/// Returns a minimal HTML page that fills the viewport with the given color.
pub fn html_with_color(color: &str) -> String {
    format!(r#"<!doctype html><html><body style=\"margin:0;background:{color};\"><div style=\"width:100vw;height:100vh;background:{color};\"></div></body></html>"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn alternates() {
        let a = next_html();
        let b = next_html();
        assert_ne!(a, b);
    }

    #[test]
    fn concurrent_next_html() {
        let mut handles = Vec::new();
        for _ in 0..100 {
            handles.push(thread::spawn(|| next_html()));
        }
        let pages: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(pages.iter().any(|s| s.contains("red")));
        assert!(pages.iter().any(|s| s.contains("blue")));
    }
}

