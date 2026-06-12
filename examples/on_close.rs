use std::time::Duration;

use mac_usernotifications::{Notification, block_on_current};

mod common;

const ACTION: &str = "action.something";

fn main() {
    if !common::setup(file!()) {
        return;
    }

    let response = Notification::new()
        .title("on_close demo")
        .message("Click the body, press \"Close\", swipe away, or wait 10 s.")
        .action(Action::button(ACTION, "Something"))
        .timeout(Duration::from_secs(10))
        .send_blocking()
        .and_then(|handle| block_on_current(handle.response()).unwrap())
        .unwrap();

    if response.is_default_action() {
        log::info!("opened — user clicked the notification body");
    } else if response.action_identifier == ACTION {
        log::info!("closed — user pressed the Action button");
    } else if response.is_dismiss_action() {
        log::info!("dismissed — user swiped it away or cleared it in Notification Center");
    } else if response.is_timed_out() {
        log::info!("expired — 10 s elapsed with no interaction");
    } else {
        log::info!("unknown response: {:?}", response.action_identifier);
    }
}
