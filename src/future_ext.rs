use std::fmt::Display;
use std::time::{Instant, Duration};
use tokio_core::reactor::Handle;
use futures::Future;
use log::LogLevel;
use void::Void;

use log_error::LogError;
use until::Until;
use infallible::Infallible;
use finally::Finally;
use with_timeout::WithTimeout;
use first_ok2::FirstOk2;
use while_driving::WhileDriving;
use BoxFuture;

/// Extension trait for `Future`.
pub trait FutureExt: Future + Sized {
    /// Wraps a future into a boxed future, making type-checking easier at the expense of an extra
    /// layer of indirection at runtime.
    fn into_boxed(self) -> BoxFuture<Self::Item, Self::Error>
    where
        Self: 'static
    {
        Box::new(self)
    }

    /// Run this future until some condition is met. If `condition` resolves before `self` then
    /// `None` is returned.
    ///
    /// # Example
    /// ```rust
    /// let my_future_with_timeout = my_future.until(Timeout::new(Duration::from_secs(1)));
    /// ```
    fn until<C>(self, condition: C) -> Until<Self, C>
    where
        C: Future<Item=()>,
        Self::Error: From<C::Error>
    {
        Until::new(self, condition)
    }

    /// For futures which can't fail (ie. which have error type `Void`), cast the error type to
    /// some inferred type.
    fn infallible<E>(self) -> Infallible<Self, E>
    where
        Self: Future<Error=Void>
    {
        Infallible::new(self)
    }

    /// Take a future which returns `()` and log its error if it fails. The returned future cannot
    /// fail and will always resolve to `()`.
    fn log_error(self, level: LogLevel, description: &'static str) -> LogError<Self>
    where
        Self: Future<Item=()>,
        Self::Error: Display,
    {
        LogError::new(self, level, description)
    }

    /// Executes the future and runs the provided callback when the future finishes. The callback
    /// will also be run if the entire future is dropped.
    fn finally<D>(self, on_drop: D) -> Finally<Self, D>
    where
        D: FnOnce()
    {
        Finally::new(self, on_drop)
    }

    /// Runs the future for the given duration, returning its value in an option, or returning
    /// `None` if the timeout expires.
    fn with_timeout(self, duration: Duration, handle: &Handle) -> WithTimeout<Self> {
        WithTimeout::new(self, duration, handle)
    }

    /// Runs the future until the given instant, returning its value in an option, or returning
    /// `None` if the timeout expires.
    fn with_timeout_at(self, instant: Instant, handle: &Handle) -> WithTimeout<Self> {
        WithTimeout::new_at(self, instant, handle)
    }

    /// Run two futures in parallel and yield the value of the first to return success. If both
    /// futures fail, return both errors.
    fn first_ok2<F>(self, other: F) -> FirstOk2<Self, F>
    where
        F: Future<Item = Self::Item>
    {
        FirstOk2::new(self, other)
    }

    /// Resolves `self` while driving `other` in parallel.
    fn while_driving<F: Future>(self, other: F) -> WhileDriving<Self, F> {
        WhileDriving::new(self, other)
    }
}

impl<T: Future + Sized> FutureExt for T {}

