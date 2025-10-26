//! M6 ai-runtime: Phase-1 mock CPU path (no external model).

use message_defs::{AiRequest, AiResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("empty prompt")] Empty,
}

pub fn ask(req: AiRequest) -> Result<AiResponse, AiError> {
    let p = req.prompt.trim();
    if p.is_empty() { return Err(AiError::Empty); }
    Ok(AiResponse { text: summarize_text(p, req.max_tokens as usize) })
}

/// Very small summarizer: if the input contains "Example Domain", return it;
/// otherwise return the first up to `max_tokens` words.
pub fn summarize_text(input: &str, max_tokens: usize) -> String {
    let lower = input.to_lowercase();
    if lower.contains("example domain") { return "Example Domain".to_string(); }
    let mut out = String::new();
    for (i, w) in input.split_whitespace().take(max_tokens.max(1)).enumerate() {
        if i > 0 { out.push(' '); }
        out.push_str(w);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_keyword_if_present() {
        let s = summarize_text("<h1>Example Domain</h1>", 10);
        assert_eq!(s, "Example Domain");
    }
}

