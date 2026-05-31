//! # Pitfall: `response_blocking` from a background thread without a main run loop
//!
//! This example deliberately demonstrates the **wrong** pattern to help you recognise the warning in your own logs:
//!
//! ```text
//! WARN response_blocking called from a background thread but the main
//!      NSRunLoop does not appear to be running. The call may hang
//!      indefinitely. ...
//! ```
//!
//! ## Why does this hang?
//!
//! `UNUserNotificationCenter` always delivers `didReceiveNotificationResponse`
//! on the **main thread's run loop**, regardless of which thread sent the
//! notification. If nothing is pumping the main run loop, the callback never
//! fires and `response_blocking` waits forever.
//!
//! ## The fix
//!
//! Drive the main run loop from the main thread while your background work
//! runs. See `actions_blocking.rs` for the correct pattern:
//!
//! ```rust,no_run
//! // main thread
//! mac_usernotifications::run_main_loop_while(|| !done.load(Ordering::Acquire));
//!
//! // background thread
//! let response = handle.response_blocking()?;
//! ```

use mac_usernotifications::{Action, Notification, blocking};
use std::time::Duration;

mod common;

fn main() {
    if !common::setup(file!()) {
        return;
    }

    // Spawn the notification work onto a background thread, just like a real
    // app might do to avoid blocking the main thread.
    let worker = std::thread::spawn(|| {
        let notification = Notification::new()
            .title("Pitfall demo")
            .message("Main run loop is not being pumped — watch the warning above.")
            .action(Action::button("ok", "OK"))
            // Short timeout so the example exits on its own rather than
            // hanging indefinitely. In a real app without a timeout this
            // would block forever.
            .timeout(Duration::from_secs(8));

        // ⚠️  This is the problematic call. The main thread is sitting idle
        // below (no `run_main_loop_while`), so the OS callback can never fire.
        // The crate logs a WARN before blocking, which is what we want to show.
        let response = blocking::send_with_actions(notification);

        match response {
            Ok(resp) if resp.is_timed_out() => {
                log::warn!("timed out — as expected, the callback never arrived");
            }
            Ok(resp) => {
                // This branch is only reachable when running inside a GUI app
                // (AppKit/Tauri) whose framework drives the main run loop
                // independently. In that case the warning is suppressed and
                // the response arrives normally.
                log::info!("got response: {:?}", resp.action_identifier);
            }
            Err(err) => log::error!("error: {err}"),
        }
    });

    // ❌ Nothing pumps the main run loop here — that's the whole point.
    //    In a correct CLI app you would call:
    //
    //      mac_usernotifications::run_main_loop_while(|| !done.load(Ordering::Acquire));
    //
    //    See `actions_blocking.rs` for the right approach.

    worker.join().expect("worker thread panicked");
    log::info!("done");
}
