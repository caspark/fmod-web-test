use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::audio::*;

#[wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(js_name = "log", js_namespace = ["window", "console"])]
    fn console_log(message: &str);
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> Result<(), JsValue> {
    let banks = vec!["Master.bank", "Master.strings.bank", "SFX.bank"];

    let fmod_loader = crate::audio::start_loading_audio_backend("/assets/", &banks);

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

        if let Some(load_result) = fmod_loader.get_loaded() {
            match load_result {
                Ok(fmod_web) => match handle_audio(fmod_web, &mut events, i) {
                    Ok(_) => (),
                    Err(e) => {
                        console_log(&format!("Audio error: {:?}", e));
                    }
                },
                Err(e) => {
                    console_log(&format!("Audio loading error: {:?}", e));
                }
            }
        }
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

struct LoadedEvents {
    explosion_description: Box<dyn AudioEventDescription>,
}

fn handle_audio(
    fmod_web: Box<dyn AudioBackend>,
    events: &mut Option<LoadedEvents>,
    i: i32,
) -> AudioResult<()> {
    if events.is_none() {
        let explosion_description = fmod_web.get_event("event:/Weapons/Explosion")?;

        *events = Some(LoadedEvents {
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
