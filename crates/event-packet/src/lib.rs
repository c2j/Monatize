#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Input event IDL for Phase 0 (serde optional)

/// Canonical input event payloads exchanged across processes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Key pressed
    KeyDown {
        /// Printable key or identifier
        key: String,
        /// Platform keycode (hardware scancode or DOM code)
        code: u32,
    },
    /// Key released
    KeyUp {
        /// Printable key or identifier
        key: String,
        /// Platform keycode (hardware scancode or DOM code)
        code: u32,
    },
    /// Cursor moved
    MouseMove {
        /// X coordinate in logical pixels
        x: f32,
        /// Y coordinate in logical pixels
        y: f32,
    },
    /// Mouse button pressed
    MouseDown {
        /// Button index (0=left, 1=middle, 2=right)
        button: u8,
        /// X coordinate in logical pixels
        x: f32,
        /// Y coordinate in logical pixels
        y: f32,
    },
    /// Mouse button released
    MouseUp {
        /// Button index (0=left, 1=middle, 2=right)
        button: u8,
        /// X coordinate in logical pixels
        x: f32,
        /// Y coordinate in logical pixels
        y: f32,
    },
    /// Scroll wheel
    Wheel {
        /// Horizontal scroll delta
        delta_x: f32,
        /// Vertical scroll delta
        delta_y: f32,
    },
    /// Window resized
    Resize {
        /// New width in pixels
        w: u32,
        /// New height in pixels
        h: u32,
    },
}
/// IPC message payloads for Phase 0 UDS framing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// A raw RGBA8 frame with size and stride.
    Frame {
        /// Pixel buffer in RGBA8
        pixels: Vec<u8>,
        /// (width, height)
        size: (u32, u32),
        /// Bytes per row
        stride: u32,
    },
    /// An input event
    Event(InputEvent),
    /// Termination signal
    Quit,
}

/// Write a length-prefixed (u32 LE) bincode Message to the writer.
#[cfg(all(feature = "serde", feature = "bincode"))]
pub fn write_len_prefixed<W: std::io::Write>(mut w: W, msg: &Message) -> std::io::Result<()> {
    let body = bincode::serialize(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = body.len() as u32;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&body)?;
    Ok(())
}

/// Read a length-prefixed (u32 LE) bincode Message from the reader.
#[cfg(all(feature = "serde", feature = "bincode"))]
pub fn read_len_prefixed<R: std::io::Read>(mut r: R) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    bincode::deserialize::<Message>(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_construct() {
        let e = InputEvent::KeyDown { key: "A".into(), code: 65 };
        match e { InputEvent::KeyDown { .. } => (), _ => panic!("mismatch"), }
    }

    #[cfg(all(feature = "serde", feature = "bincode"))]
    #[test]
    fn bincode_round_trip() {
        let e = InputEvent::MouseDown { button: 1, x: 10.5, y: 20.25 };
        let bytes = bincode::serialize(&e).unwrap();
        let d: InputEvent = bincode::deserialize(&bytes).unwrap();
        assert_eq!(e, d);
    }

    #[cfg(all(feature = "serde", feature = "bincode"))]
    #[test]
    fn unknown_variant_no_panic() {
        #[allow(missing_docs)]
        #[derive(serde::Serialize, serde::Deserialize)]
        enum AnotherEvent {
            KeyDown { key: String, code: u32 },
            KeyUp { key: String, code: u32 },
            MouseMove { x: f32, y: f32 },
            MouseDown { button: u8, x: f32, y: f32 },
            MouseUp { button: u8, x: f32, y: f32 },
            Wheel { delta_x: f32, delta_y: f32 },
            Resize { w: u32, h: u32 },
            Other,
        }
        let bytes = bincode::serialize(&AnotherEvent::Other).unwrap();
        let res: Result<InputEvent, _> = bincode::deserialize(&bytes);
        assert!(res.is_err());
    }

    #[cfg(all(feature = "serde", feature = "bincode"))]
    #[test]
    fn message_len_prefix_round_trip() {
        use std::io::Cursor;
        let mut buf = Vec::new();
        let m1 = Message::Event(InputEvent::KeyUp { key: "A".into(), code: 65 });
        let m2 = Message::Quit;
        super::write_len_prefixed(&mut buf, &m1).unwrap();
        super::write_len_prefixed(&mut buf, &m2).unwrap();
        let mut c = Cursor::new(buf);
        let d1 = super::read_len_prefixed(&mut c).unwrap();
        let d2 = super::read_len_prefixed(&mut c).unwrap();
        assert_eq!(m1, d1);
        assert_eq!(m2, d2);
    }
}


