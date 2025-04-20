use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = ["window", "fmodController"])]
    fn is_loaded() -> bool;

    #[wasm_bindgen(js_namespace = ["window", "fmodController"])]
    fn tick();

    #[wasm_bindgen(js_namespace = ["window", "fmodController"])]
    fn play_event(sound_id: i32);

}

#[cfg(target_arch = "wasm32")]
pub fn run() -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

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

        if is_loaded() {
            if i % 100 == 0 {
                play_event(0);
            }

            tick();
        }
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
