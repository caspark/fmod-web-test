//! An abstraction over the audio backend to support both desktop and web.
//!
//! Since loading on web needs to happen asynchronously (due to needing to instantiate fmod via a
//! callback, and usually also waiting for audio files to be preloaded onto the emscripten
//! filesystem), this API forces loading to happen asynchronously. Could be improved in the future
//! by letting desktop loading happen synchronously and only making the loading asynchronous on web.
//!
//! This wrapper also boxes everything to make it possible to move types as necessary (e.g.
//! [AudioEventInstance::release] takes self by value to ensure that the instance is not again used
//! later). There might be a neater way to do that instead.
//!
//! Lastly, this wrapper flattens banks into the system, which is just because in my game I only use
//! one bank - a more flexible approach would be to expose banks as separate from the audiobackend
//! (/fmod system).
#![allow(dead_code)]

use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
mod backend_desktop;
#[cfg(target_arch = "wasm32")]
mod backend_web;

pub type AudioResult<T> = anyhow::Result<T>;

pub fn start_loading_audio_backend(
    banks_path: &str,
    bank_filenames: &[&str],
) -> Box<dyn AudioBackendLoader> {
    info!("Loading audio backend");

    #[cfg(target_arch = "wasm32")]
    return backend_web::load_audio_backend(banks_path, bank_filenames);
    #[cfg(not(target_arch = "wasm32"))]
    return backend_desktop::load_audio_backend(banks_path, bank_filenames);
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

    fn get_event(&self, event_name: &str) -> AudioResult<Box<dyn AudioEventDescription>>;

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
