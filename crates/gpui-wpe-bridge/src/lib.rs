#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Minimal frame/bridge types (Phase 0)

/// External frame in RGBA8 copy path
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFrame {
    /// Pixel buffer in RGBA8
    pub pixels: Vec<u8>,
    /// (width, height)
    pub size: (u32, u32),
    /// Bytes per row
    pub stride: u32,
}

impl ExternalFrame {
    /// Create a zeroed frame of given size
    pub fn new(size: (u32, u32)) -> Self {
        let (w, h) = size;
        let stride = w.saturating_mul(4);
        let len = (stride as u64).saturating_mul(h as u64) as usize;
        let pixels = vec![0u8; len];
        Self { pixels, size, stride }
    }
}

/// Minimal event translation helpers (Phase 0 stub)
pub mod events {
    use event_packet::InputEvent;

    /// Translate a window resize to InputEvent::Resize
    pub fn resize(w: u32, h: u32) -> InputEvent { InputEvent::Resize { w, h } }

    /// Translate mouse move to InputEvent::MouseMove
    pub fn mouse_move(x: f32, y: f32) -> InputEvent { InputEvent::MouseMove { x, y } }

    #[test]
    fn event_translate_stability() {
        use crate::events as ev;
        for i in 0..100u32 {
            let r = ev::resize(100 + i, 200 + i);
            match r { event_packet::InputEvent::Resize { w, h } => { assert_eq!(w, 100 + i); assert_eq!(h, 200 + i); }, _ => panic!("bad") }
            let m = ev::mouse_move(i as f32 * 0.5, i as f32 * 0.25);
            match m { event_packet::InputEvent::MouseMove { x, y } => { assert_eq!(x, i as f32 * 0.5); assert_eq!(y, i as f32 * 0.25); }, _ => panic!("bad") }
            let md = ev::mouse_down((i % 3) as u8, i as f32, i as f32 + 1.0);
            match md { event_packet::InputEvent::MouseDown { button, x, y } => { assert_eq!(button, (i % 3) as u8); assert_eq!(x, i as f32); assert!((y - (i as f32 + 1.0)).abs() < f32::EPSILON); }, _ => panic!("bad") }
            let mu = ev::mouse_up((i % 3) as u8, i as f32, i as f32 + 2.0);
            match mu { event_packet::InputEvent::MouseUp { button, x, y } => { assert_eq!(button, (i % 3) as u8); assert_eq!(x, i as f32); assert!((y - (i as f32 + 2.0)).abs() < f32::EPSILON); }, _ => panic!("bad") }
            let w = ev::wheel(i as f32, -(i as f32));
            match w { event_packet::InputEvent::Wheel { delta_x, delta_y } => { assert_eq!(delta_x, i as f32); assert_eq!(delta_y, -(i as f32)); }, _ => panic!("bad") }
            let kd = ev::key_down("A", i);
            match kd { event_packet::InputEvent::KeyDown { ref key, code } => { assert_eq!(key, "A"); assert_eq!(code, i); }, _ => panic!("bad") }
            let ku = ev::key_up("A", i);
            match ku { event_packet::InputEvent::KeyUp { ref key, code } => { assert_eq!(key, "A"); assert_eq!(code, i); }, _ => panic!("bad") }
        }
    }

    /// Translate mouse press
    pub fn mouse_down(button: u8, x: f32, y: f32) -> InputEvent { InputEvent::MouseDown { button, x, y } }

    /// Translate mouse release
    pub fn mouse_up(button: u8, x: f32, y: f32) -> InputEvent { InputEvent::MouseUp { button, x, y } }

    /// Translate scroll
    pub fn wheel(dx: f32, dy: f32) -> InputEvent { InputEvent::Wheel { delta_x: dx, delta_y: dy } }

    /// Translate key down (Phase 0: pass-through string + numeric code)
    pub fn key_down(key: &str, code: u32) -> InputEvent { InputEvent::KeyDown { key: key.to_string(), code } }

    /// Translate key up (Phase 0: pass-through string + numeric code)
    pub fn key_up(key: &str, code: u32) -> InputEvent { InputEvent::KeyUp { key: key.to_string(), code } }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_size() {
        let f = ExternalFrame::new((2, 3));
        assert_eq!(f.pixels.len(), (2 * 4 * 3) as usize);
        assert_eq!(f.stride, 8);
    }
}

