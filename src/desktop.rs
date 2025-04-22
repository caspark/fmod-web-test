use crossterm::{
    cursor::*,
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::*,
};
use fmod::{Utf8CStr, Utf8CString, c};
use fmod_sys;
use std::ffi::{c_char, c_int};
use std::io::Write;

unsafe extern "C" fn my_debug_callback(
    flags_raw: fmod_sys::FMOD_DEBUG_FLAGS,
    _file_ptr: *const c_char,
    _line: c_int,
    func_ptr: *const c_char,
    message_ptr: *const c_char,
) -> fmod_sys::FMOD_RESULT {
    let flags = fmod::debug::DebugFlags::from(flags_raw);

    let flag_str = if flags.contains(fmod::debug::DebugFlags::ERROR) {
        "ERROR"
    } else if flags.contains(fmod::debug::DebugFlags::WARNING) {
        "WARN"
    } else if flags.contains(fmod::debug::DebugFlags::LOG) {
        "LOG"
    } else if flags.contains(fmod::debug::DebugFlags::MEMORY) {
        "MEMORY"
    } else if flags.contains(fmod::debug::DebugFlags::FILE) {
        "FILE"
    } else if flags.contains(fmod::debug::DebugFlags::CODEC) {
        "CODE"
    } else if flags.contains(fmod::debug::DebugFlags::TRACE) {
        "TRACE"
    } else {
        "NONE"
    };

    let func = unsafe { Utf8CStr::from_ptr(func_ptr).ok() };
    let message = unsafe { Utf8CStr::from_ptr(message_ptr).ok() };
    if let Some((func, message)) = func.zip(message) {
        print!("FMOD: {flag_str} {func}: {message}");
    }

    fmod_sys::FMOD_RESULT::FMOD_OK
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // read first argument as path to studio examples dir
    let studio_examples_dir = std::env::args().nth(1).unwrap();

    fmod::debug::initialize(
        fmod::debug::DebugFlags::LOG,
        fmod::debug::DebugMode::Callback(my_debug_callback),
    )?;

    let mut builder = unsafe {
        // Safety: we call this before calling any other functions and only in main, so this is safe
        fmod::studio::SystemBuilder::new()?
    };

    // The example Studio project is authored for 5.1 sound, so set up the system output mode to match
    builder
        .core_builder()
        .software_format(0, fmod::SpeakerMode::FivePointOne, 0)?;

    // make sure the expected examples directory exists
    let examples_dir = std::path::Path::new(&studio_examples_dir);
    if !examples_dir.exists() {
        return Err(format!("examples directory not found: {}", studio_examples_dir).into());
    }

    let system = builder.build(
        1024,
        fmod::studio::InitFlags::LIVEUPDATE,
        fmod::InitFlags::NORMAL,
    )?;

    system.load_bank_file(
        &Utf8CString::new(examples_dir.join("media/Master.bank").to_str().unwrap())?,
        fmod::studio::LoadBankFlags::NORMAL,
    )?;
    system.load_bank_file(
        &Utf8CString::new(
            examples_dir
                .join("media/Master.strings.bank")
                .to_str()
                .unwrap(),
        )?,
        fmod::studio::LoadBankFlags::NORMAL,
    )?;
    system.load_bank_file(
        &Utf8CString::new(examples_dir.join("media/SFX.bank").to_str().unwrap())?,
        fmod::studio::LoadBankFlags::NORMAL,
    )?;

    // Get the Looping Ambience event
    let looping_ambience_description = system.get_event(c!("event:/Ambience/Country"))?;
    let looping_ambiance_instance = looping_ambience_description.create_instance()?;

    // Get the 4 Second Surge event
    let cancel_description = system.get_event(c!("event:/Ui/Cancel"))?;
    let cancel_instance = cancel_description.create_instance()?;

    // Get the Single Explosion event
    let explosion_description = system.get_event(c!("event:/Weapons/Explosion"))?;
    // Start loading explosion sample data and keep it in memory
    explosion_description.load_sample_data()?;

    // use alternate screen
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    crossterm::terminal::enable_raw_mode()?;

    'main_loop: loop {
        while crossterm::event::poll(std::time::Duration::from_micros(1000))? {
            let event = crossterm::event::read()?;

            let Event::Key(KeyEvent {
                code: KeyCode::Char(character),
                ..
            }) = event
            else {
                continue;
            };

            match character {
                '1' => {
                    let explosion_instance = explosion_description.create_instance()?;

                    explosion_instance.start()?;

                    // Release will clean up the instance when it completes
                    explosion_instance.release()?;
                }
                '2' => {
                    looping_ambiance_instance.start()?;
                }
                '3' => {
                    looping_ambiance_instance.stop(fmod::studio::StopMode::Immediate)?;
                }
                '4' => {
                    // Calling start on an instance will cause it to restart if it's already playing
                    cancel_instance.start()?;
                }
                'q' => {
                    break 'main_loop;
                }
                _ => {}
            }
        }

        system.update()?;

        execute!(stdout, Clear(ClearType::All))?;

        execute!(stdout, MoveTo(0, 0))?;
        crossterm::terminal::disable_raw_mode()?;

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

        crossterm::terminal::enable_raw_mode()?;

        stdout.flush()?;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // reset terminal
    crossterm::terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    unsafe {
        // Safety: we don't use any fmod api calls after this, so this is ok
        system.release()?;
    }

    Ok(())
}
