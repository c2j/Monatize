//! M8 dom-embed: sanitize and inject shadow DOM helpers (Phase-1 minimal)

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomError {
    #[error("invalid input")] Invalid,
}

/// Remove <script>...</script> blocks and inline event attributes (on*="...")
/// This is a naive sanitizer for Phase-1 only; not production-ready.
pub fn sanitize_html(input: &str) -> Result<String, DomError> {
    if input.is_empty() { return Err(DomError::Invalid); }
    let mut s = String::with_capacity(input.len());

    // Remove <script> blocks (very naive)
    let mut rest = input;
    loop {
        if let Some(i) = rest.to_lowercase().find("<script") {
            let (head, after_head) = rest.split_at(i);
            s.push_str(head);
            if let Some(j) = after_head.to_lowercase().find("</script>") {
                rest = &after_head[j + "</script>".len()..];
            } else {
                // No closing tag; drop the tail
                rest = "";
                break;
            }
        } else {
            s.push_str(rest);
            break;
        }
    }

    // Remove inline on* attributes naively
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'o' || bytes[i] == b'O' {
            // check for onXxx=
            let slice = &s[i..];
            if slice.len() > 3 && slice[..2].eq_ignore_ascii_case("on") {
                if let Some(eq) = slice.find('=') {
                    // Skip until after the attribute value (quoted or unquoted)
                    let after_eq = i + eq + 1;
                    if after_eq < s.len() && (s.as_bytes()[after_eq] == b'\"' || s.as_bytes()[after_eq] == b'\'') {
                        let quote = s.as_bytes()[after_eq];
                        let mut k = after_eq + 1;
                        while k < s.len() && s.as_bytes()[k] != quote { k += 1; }
                        i = (k + 1).min(s.len());
                        continue;
                    } else {
                        // unquoted: skip until space or '>'
                        let mut k = after_eq;
                        while k < s.len() && s.as_bytes()[k] != b' ' && s.as_bytes()[k] != b'>' { k += 1; }
                        i = k;
                        continue;
                    }
                }
            }
        }
        out.push(s.as_bytes()[i] as char);
        i += 1;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_blocks() {
        let html = "<div>ok<script>alert(1)</script>end";
        let out = sanitize_html(html).unwrap();
        assert_eq!(out, "<div>okend");
    }

    #[test]
    fn strips_onclick() {
        let html = "<a href='#' onclick=\"evil()\">x</a>";
        let out = sanitize_html(html).unwrap();
        assert!(!out.to_lowercase().contains("onclick"));
    }
}

