//! Async drop support for Rust.
//!
//! This crate provides [`AsyncDrop`], a trait for types that need to perform
//! asynchronous cleanup, and [`Dropper`], a wrapper that automatically runs
//! the async cleanup when the value is dropped.
//!
//! # Example
//!
//! ```
//! use async_drop::{AsyncDrop, AsyncDropFuture, Dropper};
//!
//! struct DbConnection;
//!
//! impl AsyncDrop for DbConnection {
//!     fn async_drop(&mut self) -> AsyncDropFuture<'_> {
//!         Box::pin(async {
//!             // Close the connection gracefully...
//!             Ok(())
//!         })
//!     }
//! }
//!
//! let conn = Dropper::new(DbConnection);
//! // Use `conn` as if it were a `DbConnection` (via Deref).
//! // When `conn` goes out of scope, `async_drop` runs automatically.
//! ```

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

/// The future type returned by [`AsyncDrop::async_drop`].
pub type AsyncDropFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

/// A trait for types that require asynchronous cleanup.
///
/// Implement this trait to define async teardown logic (e.g. flushing buffers,
/// closing network connections, or releasing distributed locks). Use
/// [`Dropper`] to ensure `async_drop` is called automatically when the value
/// goes out of scope.
pub trait AsyncDrop {
    /// Performs asynchronous cleanup of this value.
    ///
    /// Returns `Ok(())` on success or `Err` with a message that will be
    /// printed to stderr.
    fn async_drop(&mut self) -> AsyncDropFuture<'_>;
}

/// A wrapper that calls [`AsyncDrop::async_drop`] when dropped.
///
/// `Dropper<T>` dereferences to `T`, so the wrapped value can be used
/// transparently. On drop, it spawns a dedicated thread with a short-lived
/// Tokio runtime to drive the cleanup future to completion. This avoids
/// deadlocks when the `Dropper` is dropped inside a single-threaded async
/// executor.
///
/// # Panics
///
/// Panics if the internal Tokio runtime cannot be created.
pub struct Dropper<T>
where
    T: AsyncDrop,
{
    inner: Option<T>,
}

impl<T> Dropper<T>
where
    T: AsyncDrop,
{
    /// Wraps `inner` so that its [`AsyncDrop::async_drop`] implementation is
    /// called automatically when this `Dropper` is dropped.
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<T> Drop for Dropper<T>
where
    T: AsyncDrop,
{
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };

        let future = inner.async_drop();

        // Spawn a dedicated thread with its own tokio runtime so we don't
        // deadlock when dropped inside a single-threaded tokio executor.
        std::thread::scope(|s| {
            s.spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build async-drop runtime");
                if let Err(e) = rt.block_on(future) {
                    eprintln!("{}", e);
                }
            });
        });
    }
}

impl<T> Deref for Dropper<T>
where
    T: AsyncDrop + 'static,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
            .as_ref()
            .expect("Dropper value has already been dropped")
    }
}

impl<T> DerefMut for Dropper<T>
where
    T: AsyncDrop + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .as_mut()
            .expect("Dropper value has already been dropped")
    }
}
