/// Errors returned by this crate.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// No bundle identifier (need valid `.app` bundle).
    #[error("No bundle identifier found. UNUserNotificationCenter requires a valid .app bundle.")]
    NoBundleIdentifier,

    /// macOS rejected the request.
    #[error("macOS rejected the notification request")]
    NotificationRejected,

    /// Timeout waiting for user interaction.
    ///
    /// (e.g. "Clear All" in Notification Center, or timeout elapsed).
    #[error("Timed out waiting for notification response")]
    ResponseTimeout,
}
