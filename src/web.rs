use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/fmod-web.js")]
extern "C" {
    type FmodWeb;
    type FmodEvent;
    type FmodInstance;

    #[wasm_bindgen(js_name = "default")]
    fn load_fmod(base_path: &str, banks: Vec<String>) -> FmodWeb;

    #[wasm_bindgen(method)]
    fn is_loaded(this: &FmodWeb) -> bool;

    #[wasm_bindgen(method)]
    fn tick(this: &FmodWeb);

    #[wasm_bindgen(method)]
    fn play_event(this: &FmodWeb, sound_id: i32);

    #[wasm_bindgen(method)]
    fn get_event(this: &FmodWeb, event_name: &str) -> FmodEvent;

    #[wasm_bindgen(method)]
    fn create_instance(this: &FmodEvent) -> FmodInstance;

    #[wasm_bindgen(method)]
    fn load_sample_data(this: &FmodEvent);

    #[wasm_bindgen(method)]
    fn start(this: &FmodInstance);

    #[wasm_bindgen(method)]
    fn stop(this: &FmodInstance);

    #[wasm_bindgen(method)]
    fn release(this: &FmodInstance);
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> Result<(), JsValue> {
    let banks = vec![
        "Master.bank".to_string(),
        "Master.strings.bank".to_string(),
        "SFX.bank".to_string(),
    ];

    let fmod_controller = load_fmod("/assets/", banks);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    struct LoadedEvents {
        loadingAmbienceDescription: FmodEvent,
        loadingAmbienceInstance: FmodInstance,
        cancelDescription: FmodEvent,
        cancelInstance: FmodInstance,
        explosionDescription: FmodEvent,
    }

    let mut events = None;

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if i > 300 {
            body().set_text_content(Some("All done!"));

            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        let text = format!("requestAnimationFrame has been called {} times.", i);
        body().set_text_content(Some(&text));

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());

        if fmod_controller.is_loaded() {
            if events.is_none() {
                let looping_ambience_description =
                    fmod_controller.get_event("event:/Ambience/Country");
                let looping_ambience_instance = looping_ambience_description.create_instance();
                let cancel_description = fmod_controller.get_event("event:/UI/Cancel");
                let cancel_instance = cancel_description.create_instance();
                let explosion_description = fmod_controller.get_event("event:/Weapons/Explosion");
                explosion_description.load_sample_data();

                events = Some(LoadedEvents {
                    loadingAmbienceDescription: looping_ambience_description,
                    loadingAmbienceInstance: looping_ambience_instance,
                    cancelDescription: cancel_description,
                    cancelInstance: cancel_instance,
                    explosionDescription: explosion_description,
                });
            }

            if i % 100 == 0 {
                if let Some(events) = &events {
                    let explosion_instance = events.explosionDescription.create_instance();
                    explosion_instance.start();
                    explosion_instance.release();
                }
            }
        }
        fmod_controller.tick();
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::Element {
    document()
        .query_selector("#output")
        .expect("query should be valid")
        .expect("document should have an #output element")
}
