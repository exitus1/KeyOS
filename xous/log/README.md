# Xous: Log Service

This crate is a simple log relay with per-client memory buffering. It does not implement logging, please use at least one log reader daemon to actually see the logs.

Services relying on the log facility should refer to the [`xous-api-log`](https://crates.io/crates/xous-api-log) crate for instructions on initialization and example code.

