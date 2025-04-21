use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

type FmodResult<T> = Result<T, JsValue>;

#[wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(js_name = "log", js_namespace = ["window", "console"])]
    fn console_log(message: &str);
}

#[wasm_bindgen(module = "/fmod-web.js")]
extern "C" {

    #[wasm_bindgen(js_name = "default")]
    fn load_fmod(base_path: &str, banks: Vec<String>) -> FmodLoader;

    type FmodLoader;

    #[wasm_bindgen(method, catch)]
    fn get_loaded(this: &FmodLoader) -> FmodResult<FmodWeb>;

    type FmodWeb;

    /// Run the update loop for fmod, to actually play audio.
    #[wasm_bindgen(method, catch)]
    fn update(this: &FmodWeb) -> FmodResult<()>;

    /// Get an event description from fmod.
    #[wasm_bindgen(method, catch)]
    fn get_event(this: &FmodWeb, event_name: &str) -> FmodResult<FmodEvent>;

    type FmodEvent;

    /// Create an instance of an event.
    #[wasm_bindgen(method, catch)]
    fn create_instance(this: &FmodEvent) -> FmodResult<FmodInstance>;

    /// Load the sample data for an event.
    #[wasm_bindgen(method, catch)]
    fn load_sample_data(this: &FmodEvent) -> FmodResult<()>;

    type FmodInstance;

    /// Start playing an event.
    #[wasm_bindgen(method, catch)]
    fn start(this: &FmodInstance) -> FmodResult<()>;

    /// Stop playing an event.
    #[wasm_bindgen(method, catch)]
    fn stop(this: &FmodInstance) -> FmodResult<()>;

    /// Release an event instance
    #[wasm_bindgen(method, catch)]
    fn release(this: &FmodInstance) -> FmodResult<()>;
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> Result<(), JsValue> {
    let banks = vec![
        "Master.bank".to_string(),
        "Master.strings.bank".to_string(),
        "SFX.bank".to_string(),
    ];

    let fmod_loader = load_fmod("/assets/", banks);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut events: Option<LoadedEvents> = None;

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

        if let Ok(fmod_web) = fmod_loader.get_loaded() {
            match handle_audio(&fmod_web, &mut events, i) {
                Ok(_) => (),
                Err(e) => {
                    console_log(&format!("Audio error: {:?}", e));
                }
            }
        }
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[allow(dead_code)]
struct LoadedEvents {
    looping_ambience_description: FmodEvent,
    looping_ambience_instance: FmodInstance,
    cancel_description: FmodEvent,
    cancel_instance: FmodInstance,
    explosion_description: FmodEvent,
}

fn handle_audio(fmod_web: &FmodWeb, events: &mut Option<LoadedEvents>, i: i32) -> FmodResult<()> {
    if events.is_none() {
        let looping_ambience_description = fmod_web.get_event("event:/Ambience/Country")?;
        let looping_ambience_instance = looping_ambience_description.create_instance()?;
        let cancel_description = fmod_web.get_event("event:/UI/Cancel")?;
        let cancel_instance = cancel_description.create_instance()?;
        let explosion_description = fmod_web.get_event("event:/Weapons/Explosion")?;
        explosion_description.load_sample_data()?;

        *events = Some(LoadedEvents {
            looping_ambience_description,
            looping_ambience_instance,
            cancel_description,
            cancel_instance,
            explosion_description,
        });
    }

    if i % 100 == 0 {
        if let Some(events) = &events {
            let explosion_instance = events.explosion_description.create_instance()?;
            explosion_instance.start()?;
            explosion_instance.release()?;
        }
    }

    fmod_web.update()?;

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
