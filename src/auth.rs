//! Request macOS permission to display alerts and play sounds.
//!
//! Uses [`UNUserNotificationCenter`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter)
//! and [`UNAuthorizationOptions`](https://developer.apple.com/documentation/usernotifications/unauthorizationoptions).

use std::{cell::Cell, future::Future, ptr::NonNull};

use block2::RcBlock;
use futures_channel::oneshot;
use futures_lite::future;
use objc2::runtime::Bool;
use objc2_user_notifications::{UNAuthorizationOptions, UNUserNotificationCenter};

use crate::{Error, check_bundle, settings::NotificationSettings, worker};

fn request_auth_inner() -> impl Future<Output = Result<bool, Error>> + Send + 'static {
    log::trace!("dispatching to worker");
    let (tx, rx) = oneshot::channel::<Result<bool, Error>>();
    worker::dispatch(move || {
        log::debug!("closure entered on worker");
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let tx = Cell::new(Some(tx));
        let block = RcBlock::new(move |granted: Bool, err: *mut objc2_foundation::NSError| {
            log::debug!(
                "completion handler fired (granted={}, err.is_null={})",
                granted.as_bool(),
                err.is_null()
            );
            if let Some(tx) = tx.take() {
                if tx.send(Ok(granted.as_bool())).is_err() {
                    log::warn!("receiver dropped before completion");
                }
            } else {
                log::warn!("completion fired twice");
            }
        });
        log::debug!("calling requestAuthorizationWithOptions");
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &block,
        );
        log::debug!("requestAuthorizationWithOptions returned");
    });
    async move { rx.await.unwrap_or(Err(Error::NotificationRejected)) }
}

/// Ask for permission to display notifications.
///
/// Returns `Ok(true)` if granted, `Ok(false)` if denied.
/// Prefer this over [`request_auth_blocking`] from async contexts.
pub async fn request_auth() -> Result<bool, Error> {
    check_bundle()?;
    request_auth_inner().await
}

/// Ask for permission to display notifications (blocks caller).
///
/// Returns `Ok(true)` if granted, `Ok(false)` if denied.
///
/// # Note on run loop
///
/// This uses [`futures_lite::future::block_on`] rather than [`crate::block_on_main`].
/// The [`requestAuthorization(options:completionHandler:)`](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter/requestauthorization(options:completionhandler:)) completion handler fires on an internal
/// Apple dispatch queue, not the main run loop, so there is no need to pump
/// [`NSRunLoop`](https://developer.apple.com/documentation/foundation/nsrunloop) here. Do not "fix" this to use `block_on_main`.
pub fn request_auth_blocking() -> Result<bool, Error> {
    check_bundle()?;
    future::block_on(request_auth_inner())
}

fn get_settings_inner() -> impl Future<Output = Result<NotificationSettings, Error>> + Send + 'static
{
    let (tx, rx) = oneshot::channel::<NotificationSettings>();
    let tx = Cell::new(Some(tx));
    worker::dispatch(move || {
        let block = RcBlock::new(
            move |settings: NonNull<objc2_user_notifications::UNNotificationSettings>| {
                let settings_ref = unsafe { settings.as_ref() };
                let result = NotificationSettings {
                    authorization_status: settings_ref.authorizationStatus().into(),
                    alert_enabled: settings_ref.alertSetting().into(),
                    badge_enabled: settings_ref.badgeSetting().into(),
                    sound_enabled: settings_ref.soundSetting().into(),
                    lock_screen_enabled: settings_ref.lockScreenSetting().into(),
                    notification_center_enabled: settings_ref.notificationCenterSetting().into(),
                };
                if let Some(sender) = tx.take() {
                    let _ = sender.send(result);
                }
            },
        );
        UNUserNotificationCenter::currentNotificationCenter()
            .getNotificationSettingsWithCompletionHandler(&block);
    });
    async move { rx.await.map_err(|_| Error::NotificationRejected) }
}

/// Query the current notification authorization settings without prompting.
///
/// Returns the current [`NotificationSettings`] for the app. Does not ask the user
/// for permission; use [`request_auth`] for that.
pub async fn get_notification_settings() -> Result<NotificationSettings, Error> {
    check_bundle()?;
    get_settings_inner().await
}

/// Query the current notification authorization settings, blocking the caller.
///
/// See [`get_notification_settings`] for details.
pub fn get_notification_settings_blocking() -> Result<NotificationSettings, Error> {
    check_bundle()?;
    future::block_on(get_settings_inner())
}
