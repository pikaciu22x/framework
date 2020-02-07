use core::fmt::Debug;
use std::{error::Error, sync::Mutex};

use thiserror::Error;

// Some crates represent errors using types that do not implement `std::error::Error` or even
// `core::fmt::Display`. As a result, we cannot convert them into `anyhow::Error` directly.
#[derive(Debug, Error)]
#[error("{0:?}")]
pub struct DebugAsError<E: Debug>(E);

impl<E: Debug> DebugAsError<E> {
    // This is only here for consistency with `SyncError`.
    pub fn new(error: E) -> Self {
        Self(error)
    }
}

// `anyhow` requires that the errors wrapped in `anyhow::Error` implement `Sync`.
// Some crates use `error-chain`, which generates errors that are not `Sync`.
// This is a workaround inspired by `failure::SyncFailure`:
// - <https://docs.rs/failure/0.1.6/failure/struct.SyncFailure.html>
// - <https://github.com/rust-lang-nursery/failure/blob/20f9a9e223b7cd71aed541d050cc73a747fc00c4/src/sync_failure.rs>
#[derive(Debug, Error)]
#[error("{}", _0.lock().expect("another thread panicked while formatting error"))]
pub struct SyncError<E: Error>(Mutex<E>);

impl<E: Error> SyncError<E> {
    pub fn new(error: E) -> Self {
        Self(Mutex::new(error))
    }
}
