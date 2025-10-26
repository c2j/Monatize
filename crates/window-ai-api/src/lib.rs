//! M9 window-ai-api (stub): Provide a Rust API analogous to window.ai.ask()
//! For Phase-1, this is a Rust stub over ai-runtime; WASM bindings to be added later.

pub fn ask(prompt: &str) -> String {
    ai_runtime::summarize_text(prompt, 16)
}
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// wasm-bindgen export: expose ask() to JS as ask_wasm
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn ask_wasm(prompt: String) -> String {
    ask(&prompt)
}


#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn concurrent_calls() {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(thread::spawn(|| ask("Example Domain content")));
        }
        for h in handles { assert_eq!(h.join().unwrap(), "Example Domain"); }
    }
}

