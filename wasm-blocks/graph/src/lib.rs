use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(name: &str) -> String {
    format!("Hello, {}!", name)
}
