#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use window_ai_api::ask;

// Run these tests in Node.js


#[wasm_bindgen_test]
fn ask_returns_example_domain_when_present() {
    let out = ask("This is the Example Domain page.");
    assert!(out.contains("Example Domain"), "unexpected output: {}", out);
}

