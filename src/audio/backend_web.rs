use wasm_bindgen::prelude::*;

use super::*;

type FmodResult<T> = Result<T, JsValue>;

trait FmodResultExt<T> {
    fn to_result(self) -> AudioResult<T>;
}

impl<T> FmodResultExt<T> for FmodResult<T> {
    fn to_result(self) -> AudioResult<T> {
        self.map_err(|e| anyhow::anyhow!("fmod-web.js error: {:?}", e))
    }
}

#[wasm_bindgen(module = "/fmod-web.js")]
extern "C" {

    #[wasm_bindgen(js_name = "default")]
    fn load_fmod(base_path: &str, banks: Vec<String>) -> FmodLoader;

    type FmodLoader;

    #[wasm_bindgen(method, catch)]
    fn get_loaded(this: &FmodLoader) -> FmodResult<FmodWebBackend>;

    type FmodWebBackend;

    /// Run the update loop for fmod, to actually play audio.
    #[wasm_bindgen(method, catch)]
    fn update(this: &FmodWebBackend) -> FmodResult<()>;

    /// Release the fmod system.
    #[wasm_bindgen(method, catch)]
    fn shutdown(this: &FmodWebBackend) -> FmodResult<()>;

    /// Get an event description from fmod.
    #[wasm_bindgen(method, catch)]
    fn get_event(this: &FmodWebBackend, event_name: &str) -> FmodResult<FmodEventDescription>;

    /// Get a list of all events in the fmod system.
    #[wasm_bindgen(method, catch)]
    fn get_event_list(this: &FmodWebBackend) -> FmodResult<Vec<FmodEventDescription>>;

    /// Set the listeners for the fmod system.
    #[wasm_bindgen(method, catch)]
    fn set_listeners(this: &FmodWebBackend, listeners: Vec<JsValue>) -> FmodResult<()>;

    /// Set a global parameter by name for the fmod system.
    #[wasm_bindgen(method, catch)]
    fn set_parameter_by_name(this: &FmodWebBackend, name: &str, value: f32) -> FmodResult<()>;

    type FmodEventDescription;

    /// Create an instance of an event.
    #[wasm_bindgen(method, catch)]
    fn create_instance(this: &FmodEventDescription) -> FmodResult<FmodEventInstance>;

    /// Get the path of an event.
    #[wasm_bindgen(method, catch)]
    fn get_path(this: &FmodEventDescription) -> FmodResult<String>;

    /// Load the sample data for an event.
    #[wasm_bindgen(method, catch)]
    fn load_sample_data(this: &FmodEventDescription) -> FmodResult<()>;

    type FmodEventInstance;

    /// Release an event instance
    #[wasm_bindgen(method, catch)]
    fn release(this: &FmodEventInstance) -> FmodResult<()>;

    /// Start playing an event.
    #[wasm_bindgen(method, catch)]
    fn start(this: &FmodEventInstance) -> FmodResult<()>;

    /// Stop playing an event.
    #[wasm_bindgen(method, catch)]
    fn stop(this: &FmodEventInstance) -> FmodResult<()>;

    /// Set the 3d attributes of an event instance.
    #[wasm_bindgen(method, catch)]
    fn set_3d_attributes(
        this: &FmodEventInstance,
        position: JsValue,
        velocity: JsValue,
    ) -> FmodResult<()>;

    /// Get the playback state of an event instance.
    #[wasm_bindgen(method, catch)]
    fn get_playback_state(this: &FmodEventInstance) -> FmodResult<JsValue>;
}

pub fn load_audio_backend(banks: Vec<String>, base_path: &str) -> Box<dyn AudioBackendLoader> {
    Box::new(load_fmod(base_path, banks))
}

impl AudioBackendLoader for FmodLoader {
    fn get_loaded(&self) -> Option<AudioResult<Box<dyn AudioBackend>>> {
        match FmodLoader::get_loaded(self) {
            Ok(backend) => {
                info!("FMOD backend loaded");
                Some(Ok(Box::new(backend)))
            }
            Err(e) => {
                //TODO audio: check for whether there was an error, or whether we're still waiting
                trace!("FMOD load state: {:?}", e);
                // try again later
                None
            }
        }
    }
}

impl AudioBackend for FmodWebBackend {
    fn update(&self) -> AudioResult<()> {
        FmodWebBackend::update(&self).to_result()
    }
    fn shutdown(self: Box<Self>) -> AudioResult<()> {
        FmodWebBackend::shutdown(&self).to_result()
    }

    fn get_event_list(&self) -> AudioResult<Vec<Box<dyn AudioEventDescription>>> {
        FmodWebBackend::get_event_list(&self)
            .map(|r| {
                r.into_iter()
                    .map(|e| Box::new(e) as Box<dyn AudioEventDescription>)
                    .collect()
            })
            .to_result()
    }

    fn set_listeners(&self, listeners: &[AudioListener]) -> AudioResult<()> {
        FmodWebBackend::set_listeners(
            &self,
            listeners
                .iter()
                .map(|l| {
                    serde_wasm_bindgen::to_value(&l).expect("listener serialization should succeed")
                })
                .collect(),
        )
        .to_result()
    }

    fn set_parameter_by_name(&self, name: &str, value: f32) -> AudioResult<()> {
        FmodWebBackend::set_parameter_by_name(&self, name, value).to_result()
    }
}

impl AudioEventDescription for FmodEventDescription {
    fn create_instance(&self) -> AudioResult<Box<dyn AudioEventInstance>> {
        FmodEventDescription::create_instance(&self)
            .map(|r| Box::new(r) as Box<dyn AudioEventInstance>)
            .to_result()
    }

    fn get_path(&self) -> AudioResult<String> {
        FmodEventDescription::get_path(&self).to_result()
    }
}

impl AudioEventInstance for FmodEventInstance {
    fn release(self: Box<Self>) -> AudioResult<()> {
        FmodEventInstance::release(&self).to_result()
    }

    fn start(&self) -> AudioResult<()> {
        FmodEventInstance::start(&self).to_result()
    }

    fn stop(&self) -> AudioResult<()> {
        FmodEventInstance::stop(&self).to_result()
    }

    fn set_3d_attributes(&self, position: Vec2, velocity: Vec2) -> AudioResult<()> {
        FmodEventInstance::set_3d_attributes(
            &self,
            serde_wasm_bindgen::to_value(&position)
                .expect("instance 3d attr position serialization should succeed"),
            serde_wasm_bindgen::to_value(&velocity)
                .expect("instance 3d attr velocity serialization should succeed"),
        )
        .to_result()
    }

    fn get_playback_state(&self) -> AudioResult<AudioPlaybackState> {
        FmodEventInstance::get_playback_state(&self)
            .map(|s| {
                serde_wasm_bindgen::from_value(s)
                    .expect("event instance playback state deserialization should succeed")
            })
            .to_result()
    }
}
