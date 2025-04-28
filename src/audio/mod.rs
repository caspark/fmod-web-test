use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
mod backend_desktop;
#[cfg(target_arch = "wasm32")]
mod backend_web;

pub type AudioResult<T> = anyhow::Result<T>;

pub fn start_loading_audio_backend() -> Box<dyn AudioBackendLoader> {
    let banks = vec![
        // we expect the master bank to contain all samples (at least for now)
        "Master.bank".to_owned(),
        // load the strings bank so we can look up events by name
        "Master.strings.bank".to_owned(),
    ];

    info!("Loading audio backend");

    #[cfg(target_arch = "wasm32")]
    return backend_web::load_audio_backend(banks, "./");
    #[cfg(not(target_arch = "wasm32"))]
    return backend_desktop::load_audio_backend(banks, "assets/banks");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AudioPlaybackState {
    Playing,
    Sustaining,
    Stopped,
    Starting,
    Stopping,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioListener {
    pub weight: f32,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub trait AudioBackendLoader {
    fn get_loaded(&self) -> Option<AudioResult<Box<dyn AudioBackend>>>;
}

pub trait AudioBackend {
    fn shutdown(self: Box<Self>) -> AudioResult<()>;

    fn update(&self) -> AudioResult<()>;

    fn get_event_list(&self) -> AudioResult<Vec<Box<dyn AudioEventDescription>>>;

    fn set_listeners(&self, listeners: &[AudioListener]) -> AudioResult<()>;

    fn set_parameter_by_name(&self, name: &str, value: f32) -> AudioResult<()>;
}

pub trait AudioEventDescription {
    fn create_instance(&self) -> AudioResult<Box<dyn AudioEventInstance>>;
    fn get_path(&self) -> AudioResult<String>;
}

pub trait AudioEventInstance {
    fn release(self: Box<Self>) -> AudioResult<()>;
    fn start(&self) -> AudioResult<()>;
    fn stop(&self) -> AudioResult<()>;
    fn set_3d_attributes(&self, position: Vec2, velocity: Vec2) -> AudioResult<()>;
    fn get_playback_state(&self) -> AudioResult<AudioPlaybackState>;
}
