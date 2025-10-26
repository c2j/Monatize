//! M10 servo-lite: extremely small layout engine producing a DisplayList

use message_defs::{DisplayList, DrawCmd};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("empty html")] Empty,
}

/// Very small parser: places a header rect if <h1> is present (case-insensitive)
/// and otherwise still draws a default header to avoid a blank look.
/// This is not a real layout engine; it only serves Phase-1 integration.
pub fn html_to_display_list(html: &str, viewport: (u32, u32)) -> Result<DisplayList, LayoutError> {
    if html.trim().is_empty() { return Err(LayoutError::Empty); }
    let (w, h) = viewport;

    // Background: white
    let mut items = Vec::new();
    items.push(DrawCmd::Rect { x: 0, y: 0, w, h, rgba: rgba_u32(255, 255, 255, 255) });

    // Header bar: always draw, darker if <h1> exists (case-insensitive)
    let lower = html.to_lowercase();
    let has_h1 = lower.contains("<h1") && lower.contains("</h1>");
    let header_h = 48u32;
    let (r, g, b) = if has_h1 { (32, 32, 32) } else { (200, 200, 200) };
    items.push(DrawCmd::Rect { x: 0, y: 0, w, h: header_h, rgba: rgba_u32(r, g, b, 255) });

    Ok(DisplayList { items })
}

#[inline]
pub fn rgba_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
    // Pack as 0xAARRGGBB for simplicity (documented contract within Phase-1)
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dl_has_background() {
        let dl = html_to_display_list("<html></html>", (800, 600)).unwrap();
        assert!(!dl.items.is_empty());
    }

    #[test]
    fn header_rect_present_when_h1() {
        let dl = html_to_display_list("<h1>Example</h1>", (800, 600)).unwrap();
        assert!(dl.items.len() >= 2, "expect bg + header");
    }
}

