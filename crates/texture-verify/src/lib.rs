#![forbid(unsafe_code)]
//! Minimal pixel compare helpers for Phase 0

/// Returns true if the average RGB is within tolerance of the target color.
pub fn is_roughly_color_rgba8(pixels: &[u8], color: (u8, u8, u8), tolerance: u8) -> bool {
    let mut sum = (0u64, 0u64, 0u64);
    let mut count = 0u64;
    for chunk in pixels.chunks(4) {
        if chunk.len() < 4 { break; }
        sum.0 += chunk[0] as u64;
        sum.1 += chunk[1] as u64;
        sum.2 += chunk[2] as u64;
        count += 1;
    }
    if count == 0 { return false; }
    let avg = ((sum.0 / count) as i64, (sum.1 / count) as i64, (sum.2 / count) as i64);
    let target = (color.0 as i64, color.1 as i64, color.2 as i64);
    (avg.0 - target.0).abs() as u8 <= tolerance &&
    (avg.1 - target.1).abs() as u8 <= tolerance &&
    (avg.2 - target.2).abs() as u8 <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mostly_red() {
        let pixels = [255, 0, 0, 255, 250, 5, 5, 255, 240, 10, 10, 255, 255, 0, 0, 255];
        assert!(is_roughly_color_rgba8(&pixels, (255, 0, 0), 16));
        assert!(!is_roughly_color_rgba8(&pixels, (0, 0, 255), 16));
    }
}

