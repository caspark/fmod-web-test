#![cfg_attr(target_arch = "wasm32", no_main)]

use crate::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
mod desktop;

#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
mod web;

mod audio;
mod prelude;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let studio_examples_dir = std::env::args().nth(1).expect(
        "Error: Studio examples dir should be provided as first argument - e.g. if \
        fmod sdk is at fmod/ then specify: fmod/linux/api/studio/examples/media/",
    );
    info!("Will load examples from {}", studio_examples_dir);

    desktop::run(&studio_examples_dir)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn main() -> Result<(), wasm_bindgen::prelude::JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    web::run()
}
