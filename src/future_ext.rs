use std::fmt::Display;
use futures::Future;
use log::LogLevel;
use void::Void;

use log_error::LogError;
use until::Until;
use infallible::Infallible;
use finally::Finally;
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
}

impl<T: Future + Sized> FutureExt for T {}

