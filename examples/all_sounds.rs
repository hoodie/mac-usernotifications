mod common;

use mac_usernotifications::{Notification, Sound};

/// All built-in macOS system sounds. These correspond to `.aiff` files
/// that ship with every macOS install in `/System/Library/Sounds/`.
const SYSTEM_SOUNDS: &[Sound] = &[
    Sound::Basso,
    Sound::Blow,
    Sound::Bottle,
    Sound::Frog,
    Sound::Funk,
    Sound::Glass,
    Sound::Hero,
    Sound::Morse,
    Sound::Ping,
    Sound::Pop,
    Sound::Purr,
    Sound::Sosumi,
    Sound::Submarine,
    Sound::Tink,
];

fn main() {
    if !common::setup(file!()) {
        return;
    }

    // Start with the default system notification sound.
    log::info!("Playing: Default");
    let _ = Notification::new()
        .title("Sound Demo")
        .subtitle("Default")
        .message("The default system notification sound")
        .default_sound()
        .send_blocking();

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Then cycle through every built-in macOS system sound.
    for sound in SYSTEM_SOUNDS {
        let name = sound.sound_name().unwrap();
        log::info!("Playing: {name}");

        let _ = Notification::new()
            .title("Sound Demo")
            .subtitle(name)
            .message(format!("/System/Library/Sounds/{name}.aiff"))
            .sound(sound.clone())
            .send_blocking();
        log::info!("Done Playing: {name}");

        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    log::info!("Done! Played {} sounds.", SYSTEM_SOUNDS.len() + 1);
}
