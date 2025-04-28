#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
mod run;

#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
mod run;

mod audio;
mod prelude;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let studio_examples_dir = std::env::args().nth(1).unwrap();

    run_game_loop(&studio_examples_dir)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn main() -> Result<(), wasm_bindgen::prelude::JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    run::run()
}

fn run_game_loop(studio_examples_path: &str) -> anyhow::Result<()> {
    let bank_files = vec!["Master.bank", "Master.strings.bank", "SFX.bank"];

    let audio_loader = audio::start_loading_audio_backend(studio_examples_path, &bank_files);

    // Wait for audio backend to be loaded
    let audio_backend = match audio_loader.get_loaded() {
        Some(result) => result?,
        None => return Err(anyhow::anyhow!("Failed to load audio backend")),
    };

    // Get event descriptions
    let events = audio_backend.get_event_list()?;

    // Find our events
    let mut looping_ambience = None;
    let mut cancel = None;
    let mut explosion = None;

    for event in events {
        println!("event: {:?}", event.get_path()?);
        let path = event.get_path()?;
        match path.as_str() {
            "event:/Ambience/Country" => looping_ambience = Some(event),
            "event:/UI/Cancel" => cancel = Some(event),
            "event:/Weapons/Explosion" => explosion = Some(event),
            _ => {}
        }
    }

    // Create instances for our events
    let looping_ambience =
        looping_ambience.ok_or_else(|| anyhow::anyhow!("Could not find ambience event"))?;
    let looping_ambience_instance = looping_ambience.create_instance()?;

    let cancel = cancel.ok_or_else(|| anyhow::anyhow!("Could not find cancel event"))?;
    let cancel_instance = cancel.create_instance()?;

    let explosion = explosion.ok_or_else(|| anyhow::anyhow!("Could not find explosion event"))?;

    // Use alternate screen
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    'main_loop: loop {
        while event::poll(std::time::Duration::from_micros(1000))? {
            let evt = event::read()?;

            let Event::Key(KeyEvent {
                code: KeyCode::Char(character),
                ..
            }) = evt
            else {
                continue;
            };

            match character {
                '1' => {
                    let explosion_instance = explosion.create_instance()?;
                    explosion_instance.start()?;
                    explosion_instance.release()?;
                }
                '2' => {
                    looping_ambience_instance.start()?;
                }
                '3' => {
                    looping_ambience_instance.stop()?;
                }
                '4' => {
                    cancel_instance.start()?;
                }
                'q' => {
                    break 'main_loop;
                }
                _ => {}
            }
        }

        audio_backend.update()?;

        execute!(stdout, terminal::Clear(ClearType::All))?;
        execute!(stdout, cursor::MoveTo(0, 0))?;
        terminal::disable_raw_mode()?;

        stdout.write_all(b"==================================================\n")?;
        stdout.write_all(b"Simple Event Example.\n")?;
        stdout.write_all(b"Adapted from the official FMOD example\n")?;
        stdout.write_all(b"==================================================")?;
        stdout.write_all(b"\n")?;
        stdout.write_all(b"Press 1 to fire and forget the explosion\n")?;
        stdout.write_all(b"Press 2 to start the looping ambiance\n")?;
        stdout.write_all(b"Press 3 to stop the looping ambiance\n")?;
        stdout.write_all(b"Press 4 to start/restart the cancel sound\n")?;
        stdout.write_all(b"Press Q to quit\n")?;

        terminal::enable_raw_mode()?;
        stdout.flush()?;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Reset terminal
    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;

    // Shutdown audio backend
    audio_backend.shutdown()?;

    Ok(())
}
