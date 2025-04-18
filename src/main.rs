#![no_main]

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
mod run;

#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
mod run;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn main() -> Result<(), wasm_bindgen::prelude::JsValue> {
    run::run()
}
