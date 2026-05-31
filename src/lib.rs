//! A Rust wrapper around [`UNUserNotificationCenter`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter),
//! designed for use in [notify-rust](https://docs.rs/notify-rust).
//!
//! # Bundling Requirement
//!
//! Contrary to [mac-notification-sys](https://docs.rs/mac-notification-sys),
//! this crate requires that the binary is bundled  and be code-signed,
//! an ad-hoc signature is sufficient.
//! See the bundled examples for how to set this up with `cargo-bundle`.
//!
//! # Quick start
//!
//! ```no_run
//! # use mac_usernotifications::{Action, blocking, Notification, check_bundle};
//! # use std::time::Duration;
//! # fn main() {
//! // 1. verify the process has a bundle identifier
//! check_bundle().unwrap();
//!
//! // 2. verify user gave permission
//! blocking::request_auth().unwrap();
//!
//! // 3a. fire-and-forgeta (handle.notification_id() has the UUID for later use)
//! let handle = Notification::new()
//!     .title("Hello")
//!     .message("World")
//!     .send_blocking()
//!     .unwrap();
//!
//! println!("notification id: {}", handle.notification_id());
//!
//! // 3b. actionable: blocks until the user responds (use send().await in async contexts)
//! let response = Notification::new()
//!     .title("Pick one")
//!     .action(Action::button("ok", "OK"))
//!     .action(Action::button("cancel", "Cancel"))
//!     .timeout(Duration::from_secs(30)) // 4. always set a timeout for actionable notifications
//!     .send_blocking()
//!     .and_then(|handle| handle.response_blocking())
//!     .unwrap();
//!
//! println!("User chose: {}", response.action_identifier);
//! # }
//! ```
//!
//! # Threading model
//!
//! macOS delivers [`didReceiveNotificationResponse`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenterdelegate/usernotificationcenter(_:didreceive:withcompletionhandler:)) on the main thread's [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop),
//! regardless of which thread the delegate was registered from ([Apple docs](https://developer.apple.com/documentation/usernotifications/unusernotificationcenterdelegate)).
//! The main thread's run loop must be spinning whenever you expect the user to interact with a notification.
//!
//! This crate uses a lazily-created worker thread for all Objective-C calls.
//! That thread pumps its own [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop), but response callbacks still arrive on the **main** thread.
//!
//! ## GUI apps (`AppKit` / `SwiftUI` / Tauri)
//!
//! The framework drives the main run loop automatically. Both `send` and `send_blocking` work from any thread without extra setup.
//!
//! ## CLI tools
//!
//! Nothing pumps the main run loop by default, so you have to do it yourself.
//!
//! **Blocking:** `send_blocking` handles this automatically when called from
//! the main thread. It pumps [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) between polls via [`block_on_main`].
//! Called from a background thread, it parks the caller and expects the main
//! run loop to be driven externally. See `examples/actions_blocking.rs`.
//!
//! **Async with Tokio:** `#[tokio::main]` occupies the main thread inside
//! Tokio's scheduler, so [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) never runs and callbacks never fire.
//! Keep the main thread free and run Tokio on background threads instead:
//!
//! ```no_run
//! # use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
//! # fn main() {
//! // Multi-thread runtime lives entirely on background threads.
//! let rt = tokio::runtime::Builder::new_multi_thread()
//!     .enable_all()
//!     .build()
//!     .unwrap();
//!
//! let done = Arc::new(AtomicBool::new(false));
//! let done2 = done.clone();
//!
//! rt.spawn(async move {
//!     // ... your async code, using send() etc. ...
//!     done2.store(true, Ordering::Release);
//! });
//!
//! // Main thread pumps NSRunLoop until async work signals completion.
//! mac_usernotifications::run_main_loop_while(|| !done.load(Ordering::Acquire));
//! # }
//! ```
//!
//! See `examples/simple_tokio.rs` for a complete working example.
//!
//! ## "Clear All" caveat
//!
//! If the user clicks **"Clear All"** in Notification Center,
//! [`didReceiveNotificationResponse`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenterdelegate/usernotificationcenter(_:didreceive:withcompletionhandler:)) is never called and the future will never
//! resolve. Always set a timeout via [`Notification::timeout`] for actionable
//! notifications.

#![warn(missing_docs)]
#![forbid(trivial_numeric_casts, unused_import_braces)]
#![warn(unstable_features)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    unused_qualifications
)]
#![warn(
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::inconsistent_struct_constructor,
    clippy::map_unwrap_or,
    clippy::match_same_arms
)]

use objc2_foundation::{NSBundle, NSDate, NSDefaultRunLoopMode, NSRunLoop};
use std::future::Future;

mod auth;
mod delegate;
mod error;
mod interrupt;
mod notification;
mod send;
mod settings;
pub mod sound;
mod worker;

pub mod action;
pub mod response;

pub use crate::{
    action::Action,
    auth::{get_notification_settings, request_auth},
    error::Error,
    interrupt::InterruptionLevel,
    notification::Notification,
    response::{CloseReason, NotificationResponse},
    send::{
        NotificationHandle, cancel_pending, close_delivered, get_delivered_notification_ids,
        get_pending_notification_ids, send, send_with_actions,
    },
    settings::{AuthorizationStatus, NotificationSettingStatus, NotificationSettings},
};

#[cfg(feature = "blocking-wrappers")]
pub mod blocking {
    //! Blocking wrappers for the notification API.
    pub use crate::{
        auth::{
            get_notification_settings_blocking as get_notification_settings,
            request_auth_blocking as request_auth,
        },
        send::{
            cancel_pending_blocking as cancel_pending, close_delivered_blocking as close_delivered,
            send_blocking as send, send_with_actions_blocking as send_with_actions,
        },
    };
}

#[cfg(feature = "blocking-wrappers")]
pub use futures_lite::future::block_on;

/// Pump the main thread's [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) until `should_continue` returns `false`.
///
/// **Must be called from the main thread.** Required because [`UNUserNotificationCenter`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter)
/// always delivers callbacks on the main thread's run loop; async runtimes that occupy
/// the main thread will never fire callbacks.
pub fn run_main_loop_while<F: Fn() -> bool>(should_continue: F) {
    let run_loop = NSRunLoop::mainRunLoop();
    while should_continue() {
        let until = NSDate::dateWithTimeIntervalSinceNow(0.05);
        unsafe { run_loop.runMode_beforeDate(NSDefaultRunLoopMode, &until) };
    }
}

/// Run a future to completion on the main thread while pumping [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop).
///
/// **Must be called from the main thread.** Polls the future with a no-op waker,
/// pumping the main [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) between polls to allow callbacks to fire.
/// GUI apps (Tauri, `AppKit`, `SwiftUI`) pump [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) automatically; CLI tools need this.
pub fn block_on_main<F: Future>(future: F) -> F::Output {
    use std::task::{Context, Poll};

    let mut future = std::pin::pin!(future);
    let waker = std::task::Waker::noop();
    let mut cx = Context::from_waker(waker);

    let run_loop = NSRunLoop::mainRunLoop();
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
        let until = NSDate::dateWithTimeIntervalSinceNow(0.05);
        unsafe { run_loop.runMode_beforeDate(NSDefaultRunLoopMode, &until) };
    }
}

/// Verify the process has a bundle identifier.
///
/// [`UNUserNotificationCenter`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter) requires this and crashes without it.
pub fn check_bundle() -> Result<(), Error> {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .ok_or(Error::NoBundleIdentifier)?;
    Ok(())
}
