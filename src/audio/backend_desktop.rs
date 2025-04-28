use std::ffi::{c_char, c_int};

use fmod::{Utf8CStr, Utf8CString};

use super::*;

pub fn load_audio_backend(
    banks_path: &str,
    bank_filenames: &[&str],
) -> Box<dyn AudioBackendLoader> {
    Box::new(FmodOxideAudioBackendLoader {
        banks_path: banks_path.to_owned(),
        bank_filenames: bank_filenames
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    })
}

struct FmodOxideAudioBackendLoader {
    banks_path: String,
    bank_filenames: Vec<String>,
}

impl AudioBackendLoader for FmodOxideAudioBackendLoader {
    fn get_loaded(&self) -> Option<AudioResult<Box<dyn AudioBackend>>> {
        // we just load the backend synchronously here but obviously this could be done
        // asynchronously in the future
        match FmodOxideAudioBackend::init(&self.bank_filenames, &self.banks_path) {
            Ok(backend) => {
                info!("FMOD Oxide audio backend initialized");
                Some(Ok(backend))
            }
            Err(init_err) => {
                error!("Error initializing fmod_oxide audio backend: {init_err}");
                Some(Err(init_err.into()))
            }
        }
    }
}

struct FmodOxideAudioBackend {
    system: fmod::studio::System,
    banks: Vec<(String, fmod::studio::Bank)>,
}

impl FmodOxideAudioBackend {
    pub fn init(banks: &[String], base_path: &str) -> AudioResult<Box<Self>> {
        fmod::debug::initialize(
            fmod::debug::DebugFlags::LOG,
            fmod::debug::DebugMode::Callback(fmod_log_msg_callback),
        )
        .expect("Failed to set FMOD debug settings");

        let mut builder = unsafe {
            // Safety: we call this before calling any other functions and only in main, so this is safe
            fmod::studio::SystemBuilder::new()?
        };

        // The example Studio project is authored for 5.1 sound, so set up the system output mode to match
        builder
            .core_builder()
            .software_format(0, fmod::SpeakerMode::FivePointOne, 0)?;

        // make sure the expected audio directory exists
        //TODO audio: load banks via asset manager instead of filesystem paths?
        let audio_dir = std::path::Path::new(base_path);
        if !audio_dir.exists() {
            bail!("audio directory not found: {}", audio_dir.display())
        }

        let studio_flags = if cfg!(debug_assertions) {
            info!("FMOD Studio live update enabled");
            fmod::studio::InitFlags::LIVEUPDATE
        } else {
            fmod::studio::InitFlags::NORMAL
        };
        let system = builder.build(1024, studio_flags, fmod::InitFlags::NORMAL)?;

        let mut loaded_banks = Vec::new();
        for bank_filename in banks {
            let loaded_bank = system.load_bank_file(
                &Utf8CString::new(
                    audio_dir
                        .join(bank_filename.as_str())
                        .to_str()
                        .expect("Bank path should be valid UTF-8 string"),
                )?,
                fmod::studio::LoadBankFlags::NORMAL,
            )?;
            loaded_banks.push((bank_filename.to_owned(), loaded_bank));
        }

        Ok(Box::new(FmodOxideAudioBackend {
            system,
            banks: loaded_banks,
        }))
    }
}

impl AudioBackend for FmodOxideAudioBackend {
    fn shutdown(self: Box<Self>) -> AudioResult<()> {
        unsafe {
            self.system.release()?;
        }

        Ok(())
    }

    fn update(&self) -> AudioResult<()> {
        self.system.update()?;
        Ok(())
    }

    fn get_event_list(&self) -> AudioResult<Vec<Box<dyn AudioEventDescription>>> {
        let mut all_events = Vec::new();
        for (_bank_name, bank) in &self.banks {
            let bank_events = bank
                .get_event_list()
                .with_context(|| format!("Getting event list for bank: {}", _bank_name))?;
            all_events.extend(
                bank_events
                    .into_iter()
                    .map(|e| Box::new(e) as Box<dyn AudioEventDescription>)
                    .collect::<Vec<_>>(),
            );
        }

        Ok(all_events)
    }

    fn set_listeners(&self, listeners: &[AudioListener]) -> AudioResult<()> {
        // make sure we don't exceed the max number of listeners
        let listener_count = listeners.len().min(fmod::MAX_LISTENERS as usize);
        let listeners = &listeners[..listener_count];

        self.system
            .set_listener_count(listeners.len() as c_int)
            .context("Setting listener count")?;

        // update listener positions
        for (i, listener) in listeners.iter().enumerate() {
            self.system
                .set_listener_attributes(
                    i as c_int,
                    build_3d_attrs(listener.position, listener.velocity),
                    None,
                )
                .with_context(|| format!("Setting listener {i} 3D attributes"))?;
            self.system
                .set_listener_weight(i as c_int, listener.weight)
                .with_context(|| format!("Setting listener {i} weight"))?;
        }
        Ok(())
    }

    fn set_parameter_by_name(&self, name: &str, value: f32) -> AudioResult<()> {
        fmod::studio::System::set_parameter_by_name(
            &self.system,
            &Utf8CString::new(name)?,
            value,
            false,
        )?;
        Ok(())
    }
}

unsafe extern "C" fn fmod_log_msg_callback(
    flags_raw: fmod_sys::FMOD_DEBUG_FLAGS,
    _file_ptr: *const c_char,
    _line: c_int,
    func_ptr: *const c_char,
    message_ptr: *const c_char,
) -> fmod_sys::FMOD_RESULT {
    let flags = fmod::debug::DebugFlags::from(flags_raw);

    let type_str = if flags.contains(fmod::debug::DebugFlags::MEMORY) {
        "MEMORY: "
    } else if flags.contains(fmod::debug::DebugFlags::FILE) {
        "FILE: "
    } else if flags.contains(fmod::debug::DebugFlags::CODEC) {
        "CODEC: "
    } else {
        ""
    };

    //Safety: according to fmod docs, these should always be valid UTF-8 strings
    let func = unsafe { Utf8CStr::from_ptr_unchecked(func_ptr) };
    // trim the message to remove the trailing newline
    let message = unsafe { Utf8CStr::from_ptr_unchecked(message_ptr) }.trim_end();

    // actually logging can panic in some rare circumstances, and panicking from
    // within an ffi callback is very bad, so protect against that
    if let Err(tracing_panic) = std::panic::catch_unwind(|| {
        if flags.contains(fmod::debug::DebugFlags::ERROR) {
            error!("FMOD: {type_str}{func}: {}", message);
        } else if flags.contains(fmod::debug::DebugFlags::WARNING) {
            warn!("FMOD: {type_str}{func}: {}", message);
        } else if flags.contains(fmod::debug::DebugFlags::LOG) {
            info!("FMOD: {type_str}{func}: {}", message);
        } else {
            trace!("FMOD: {type_str}{func}: {}", message);
        }
    }) {
        // attempt to extract more detailed panic information
        let panic_info = if let Some(s) = tracing_panic.downcast_ref::<String>() {
            format!("with msg: {}", s)
        } else if let Some(s) = tracing_panic.downcast_ref::<&str>() {
            format!("with msg: {}", s)
        } else if let Some(e) = tracing_panic.downcast_ref::<Box<dyn std::error::Error>>() {
            format!("with error: {}", e)
        } else {
            "(unable to determine panic cause)".to_string()
        };

        // we panicked when we tried to log, so just print to stderr
        eprintln!(
            "FMOD: {type_str}{func}: {message} (additionally, our FMOD logging callback panicked \
             {panic_info})"
        );
    }

    fmod_sys::FMOD_RESULT::FMOD_OK
}

fn build_3d_attrs(position: Vec2, velocity: Vec2) -> fmod::Attributes3D {
    // this is specific to my game: positive y is down, and we have 10 pixels to 1 meter.
    // (Theoretically, we should be able to just set the core system's 3d settings, but seems like
    // sounds are still attenuated too much that way, so we scale things here instead.)
    const UP_DIR: f32 = -1.0;
    fmod::Attributes3D {
        position: fmod::Vector {
            x: -position.x / 10.0,
            y: position.y / 10.0,
            z: 0.0,
        },
        velocity: fmod::Vector {
            x: -velocity.x / 10.0,
            y: velocity.y / 10.0,
            z: 0.0,
        },
        forward: fmod::Vector {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        up: fmod::Vector {
            x: 0.0,
            y: UP_DIR,
            z: 0.0,
        },
    }
}

impl AudioEventDescription for fmod::studio::EventDescription {
    fn create_instance(&self) -> AudioResult<Box<dyn AudioEventInstance>> {
        let instance = fmod::studio::EventDescription::create_instance(&self)?;
        Ok(Box::new(instance))
    }

    fn get_path(&self) -> AudioResult<String> {
        let path = fmod::studio::EventDescription::get_path(&self)?;
        Ok(path.as_str().to_owned())
    }
}

impl AudioEventInstance for fmod::studio::EventInstance {
    fn release(self: Box<Self>) -> AudioResult<()> {
        fmod::studio::EventInstance::release(*self)?;
        Ok(())
    }

    fn start(&self) -> AudioResult<()> {
        fmod::studio::EventInstance::start(self)?;
        Ok(())
    }

    fn stop(&self) -> AudioResult<()> {
        fmod::studio::EventInstance::stop(self, fmod::studio::StopMode::AllowFadeout)?;
        Ok(())
    }

    fn set_3d_attributes(&self, position: Vec2, velocity: Vec2) -> AudioResult<()> {
        fmod::studio::EventInstance::set_3d_attributes(self, build_3d_attrs(position, velocity))?;
        Ok(())
    }

    fn get_playback_state(&self) -> AudioResult<AudioPlaybackState> {
        let state = fmod::studio::EventInstance::get_playback_state(self)?;
        Ok(match state {
            fmod::studio::PlaybackState::Playing => AudioPlaybackState::Playing,
            fmod::studio::PlaybackState::Sustaining => AudioPlaybackState::Sustaining,
            fmod::studio::PlaybackState::Stopped => AudioPlaybackState::Stopped,
            fmod::studio::PlaybackState::Starting => AudioPlaybackState::Starting,
            fmod::studio::PlaybackState::Stopping => AudioPlaybackState::Stopping,
        })
    }
}
