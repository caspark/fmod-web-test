use crate::audio::*;
use crate::prelude::*;

pub fn run(studio_examples_path: &str) -> anyhow::Result<()> {
    let bank_files = vec!["Master.bank", "Master.strings.bank", "SFX.bank"];

    let audio_loader = start_loading_audio_backend(studio_examples_path, &bank_files);

    // Wait for audio backend to be loaded
    let audio_backend = match audio_loader.get_loaded() {
        Some(result) => result?,
        None => return Err(anyhow::anyhow!("Failed to load audio backend")),
    };

    let explosion = audio_backend.get_event("event:/Weapons/Explosion")?;

    let limit = 300;
    for i in 0..limit {
        if i % 100 == 0 {
            info!("Playing explosion {}/{}", i / 100, limit / 100);
            let explosion_instance = explosion.create_instance()?;
            explosion_instance.start()?;
            explosion_instance.release()?;
        }

        std::thread::sleep(std::time::Duration::from_millis(16));

        audio_backend.update()?;
    }

    // Shutdown audio backend
    audio_backend.shutdown()?;

    Ok(())
}
