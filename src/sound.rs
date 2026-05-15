use objc2_foundation::NSString;

/// Which sound to play when a notification is delivered.
///
/// The named variants correspond to the `.aiff` files that ship with every macOS install under `/System/Library/Sounds/`.
/// They are not part of the `UNNotificationSound` API itself;
/// they are referenced by name via `UNNotificationSound::soundNamed`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sound {
    /// Play the default system notification sound.
    Default,

    /// `/System/Library/Sounds/Basso.aiff`
    Basso,
    /// `/System/Library/Sounds/Blow.aiff`
    Blow,
    /// `/System/Library/Sounds/Bottle.aiff`
    Bottle,
    /// `/System/Library/Sounds/Frog.aiff`
    Frog,
    /// `/System/Library/Sounds/Funk.aiff`
    Funk,
    /// `/System/Library/Sounds/Glass.aiff`
    Glass,
    /// `/System/Library/Sounds/Hero.aiff`
    Hero,
    /// `/System/Library/Sounds/Morse.aiff`
    Morse,
    /// `/System/Library/Sounds/Ping.aiff`
    Ping,
    /// `/System/Library/Sounds/Pop.aiff`
    Pop,
    /// `/System/Library/Sounds/Purr.aiff`
    Purr,
    /// `/System/Library/Sounds/Sosumi.aiff`
    Sosumi,
    /// `/System/Library/Sounds/Submarine.aiff`
    Submarine,
    /// `/System/Library/Sounds/Tink.aiff`
    Tink,

    /// A named sound from the app bundle or system library.
    ///
    /// Use this for custom sound files bundled with your app, or any
    /// sound not covered by the variants above.
    Custom(String),
}

impl Sound {
    /// Returns the sound name passed to `UNNotificationSound::soundNamed`, or
    /// `None` for [`Sound::Default`] which uses `UNNotificationSound::defaultSound` instead.
    pub fn sound_name(&self) -> Option<&str> {
        match self {
            Sound::Default => None,
            Sound::Basso => Some("Basso"),
            Sound::Blow => Some("Blow"),
            Sound::Bottle => Some("Bottle"),
            Sound::Frog => Some("Frog"),
            Sound::Funk => Some("Funk"),
            Sound::Glass => Some("Glass"),
            Sound::Hero => Some("Hero"),
            Sound::Morse => Some("Morse"),
            Sound::Ping => Some("Ping"),
            Sound::Pop => Some("Pop"),
            Sound::Purr => Some("Purr"),
            Sound::Sosumi => Some("Sosumi"),
            Sound::Submarine => Some("Submarine"),
            Sound::Tink => Some("Tink"),
            Sound::Custom(name) => Some(name.as_str()),
        }
    }

    pub(crate) fn unnotificationsound(
        name: &str,
    ) -> Option<objc2::rc::Retained<objc2_user_notifications::UNNotificationSound>> {
        Some(objc2_user_notifications::UNNotificationSound::soundNamed(
            &NSString::from_str(name),
        ))
    }
    pub(crate) fn to_unnotificationsound(
        &self,
    ) -> Option<objc2::rc::Retained<objc2_user_notifications::UNNotificationSound>> {
        let name = self.sound_name()?;
        Some(objc2_user_notifications::UNNotificationSound::soundNamed(
            &NSString::from_str(name),
        ))
    }
}

impl From<&str> for Sound {
    fn from(value: &str) -> Self {
        match value {
            "default" | "Default" => Sound::Default,
            "basso" | "Basso" => Sound::Basso,
            "blow" | "Blow" => Sound::Blow,
            "bottle" | "Bottle" => Sound::Bottle,
            "frog" | "Frog" => Sound::Frog,
            "funk" | "Funk" => Sound::Funk,
            "glass" | "Glass" => Sound::Glass,
            "hero" | "Hero" => Sound::Hero,
            "morse" | "Morse" => Sound::Morse,
            "ping" | "Ping" => Sound::Ping,
            "pop" | "Pop" => Sound::Pop,
            "purr" | "Purr" => Sound::Purr,
            "sosumi" | "Sosumi" => Sound::Sosumi,
            "submarine" | "Submarine" => Sound::Submarine,
            "tink" | "Tink" => Sound::Tink,
            other => Sound::Custom(other.to_string()),
        }
    }
}

impl From<String> for Sound {
    fn from(value: String) -> Self {
        Sound::from(value.as_str())
    }
}
