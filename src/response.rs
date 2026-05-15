//! The response a user gave to a notification.
//!
//! Delivered by the delegate after the user interacts with a notification.
//! Callers receive this via the `Future` returned from `send_with_actions`.

use std::sync::OnceLock;

use objc2_user_notifications::{
    UNNotificationDefaultActionIdentifier, UNNotificationDismissActionIdentifier,
};

/// What the user did with a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationResponse {
    /// The unique identifier of the notification this response belongs to.
    ///
    /// This is the same identifier that can be passed to [`close_delivered`] or
    /// [`cancel_pending`]. Useful when the caller did not set an explicit ID via
    /// [`Notification::id`] and therefore did not know the auto-generated UUID.
    ///
    /// [`close_delivered`]: crate::close_delivered
    /// [`cancel_pending`]: crate::cancel_pending
    /// [`Notification::id`]: crate::Notification::id
    pub notification_id: String,

    /// The action identifier the user chose.
    ///
    /// Use [`is_default_action`] and [`is_dismiss_action`] for built-in cases,
    /// or compare directly with custom action identifiers.
    ///
    /// [`is_default_action`]: NotificationResponse::is_default_action
    /// [`is_dismiss_action`]: NotificationResponse::is_dismiss_action
    pub action_identifier: String,

    /// Text entered in a reply action, `None` for button actions, default-action, and dismiss.
    ///
    /// Use [`is_reply`] to check, or call `.reply_text.as_deref()` to borrow as `&str`.
    ///
    /// [`is_reply`]: NotificationResponse::is_reply
    pub reply_text: Option<String>,
}

/// Returns the dismiss action identifier string.
///
/// The result is computed once and cached for the lifetime of the process.
/// The statics are `extern "C"` symbols — reading them requires `unsafe`, so
/// we isolate that here.
fn dismiss_action_id() -> &'static str {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(|| {
        // SAFETY: `UNNotificationDismissActionIdentifier` is a valid, non-null
        // `NSString` backed by a well-known Apple framework constant that lives
        // for the lifetime of the process.  Reading an `extern "C"` static
        // requires `unsafe` in Rust; the value itself is sound to use.
        unsafe { UNNotificationDismissActionIdentifier.to_string() }
    })
}

/// Returns the default action identifier string.
///
/// See [`dismiss_action_id`] for the rationale.
fn default_action_id() -> &'static str {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(|| {
        // SAFETY: same rationale as `dismiss_action_id`.
        unsafe { UNNotificationDefaultActionIdentifier.to_string() }
    })
}

impl NotificationResponse {
    /// Construct a synthetic dismissed response for a notification with the given ID.
    ///
    /// Used for fire-and-forget (no-action) notifications where the caller still
    /// needs the auto-generated UUID back.
    pub fn dismissed(notification_id: String) -> Self {
        Self {
            notification_id,
            action_identifier: dismiss_action_id().to_owned(),
            reply_text: None,
        }
    }

    /// Returns `true` if the user clicked the notification body (default action).
    ///
    /// Corresponds to [`UNNotificationDefaultActionIdentifier`](https://developer.apple.com/documentation/usernotifications/unnotificationdefaultactionidentifier).
    pub fn is_default_action(&self) -> bool {
        self.action_identifier == default_action_id()
    }

    /// Returns `true` if the user dismissed the notification.
    ///
    /// Corresponds to [`UNNotificationDismissActionIdentifier`](https://developer.apple.com/documentation/usernotifications/unnotificationdismissactionidentifier).
    pub fn is_dismiss_action(&self) -> bool {
        self.action_identifier == dismiss_action_id()
    }

    /// Returns `true` if the user submitted text via a reply action.
    pub fn is_reply(&self) -> bool {
        self.reply_text.is_some()
    }
}
