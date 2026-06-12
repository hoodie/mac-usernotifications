# Changelog

## [v0.3.0](https://github.com/hoodie/mac-usernotifications/compare/v0.2.0...v0.3.0) (2026-06-13)

### Fixes

* poll deliveredNotifications to detect buttonless dismiss
([f2e3442](https://github.com/hoodie/mac-usernotifications/commit/f2e34429e390e52f8417893b0e61299fb7553da3))
* replace block_on_main with block_on_current
([32b26fe](https://github.com/hoodie/mac-usernotifications/commit/32b26fe02e88693e4f789acf7db03a4041f56568))
* specify semantics around block_on helpers
([73b2f12](https://github.com/hoodie/mac-usernotifications/commit/73b2f1262fead45f59c96dd47d8fbf14e7ab3b77))
* don't register CustomDismissAction on buttonless notifications
([a5c8b85](https://github.com/hoodie/mac-usernotifications/commit/a5c8b85066a4de272cfe65a303c5b564b54c163d))

## [v0.2.0](https://github.com/hoodie/mac-usernotifications/compare/v0.1.1...v0.2.0) (2026-06-08)

### Features

* drop response-blocking wrappers, nobody needs them
([fd1920b](https://github.com/hoodie/mac-usernotifications/commit/fd1920be811759588b3abc98e19851f1470d90d3))
* use RunLoop in block_on_main waker
([3d74b1d](https://github.com/hoodie/mac-usernotifications/commit/3d74b1d47e50ecc252c30bd24b6270b4cf5abe9b))

### [v0.1.1](https://github.com/hoodie/mac-usernotifications/compare/v0.1.0...v0.1.1) (2026-06-06)

#### Fixes

* introduce more precise error when response delivery was interrupted
([8b700b9](https://github.com/hoodie/mac-usernotifications/commit/8b700b96651dee7c01eb91303a10a4a56d69c645))
* simplify worker code
([9204bb8](https://github.com/hoodie/mac-usernotifications/commit/9204bb8a773386473d6b5db411326e382da7c314))

## v0.1.0 (2026-06-01)

### Features

* add timeout
([9ef2c43](https://github.com/hoodie/mac-usernotifications/commit/9ef2c433de0d31bf3b6612a64e84320eb9b3ccd7))
* initial version of mac-usernotifications
([da8dd3d](https://github.com/hoodie/mac-usernotifications/commit/da8dd3d562647506e33e75cf59680e0178bea1ac))
