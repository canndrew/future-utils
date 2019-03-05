use std::fmt::Display;
use std::time::{Instant, Duration};
use futures::{Future, Stream};
use log;
use void::Void;

use until::Until;
use first_ok::FirstOk;
use log_errors::LogErrors;
use infallible::Infallible;
use next_or_else::NextOrElse;
use finally::Finally;
use with_timeout::WithTimeout;
use with_readiness_timeout::WithReadinessTimeout;
use {BoxStream, BoxSendStream};

/// Extension trait for `Stream`.
pub trait StreamExt: Stream + Sized {
    /// Wraps a stream into a boxed stream, making type-checking easier at the expense of an extra
    /// layer of indirection at runtime.
    fn into_boxed(self) -> BoxStream<Self::Item, Self::Error>
    where
        Self: 'static
    {
        Box::new(self)
    }

    fn into_send_boxed(self) -> BoxSendStream<Self::Item, Self::Error>
    where
        Self: Send + 'static,
    {
        Box::new(self)
    }

    /// Run this stream until some condition is met. `condition` is a future which returns `()`,
    /// after which this stream will be finished.
    ///
    /// # Example
    /// ```rust
    /// let my_stream_with_timeout = my_stream.until(Delay::new(Instant::now() + Duration::from_secs(1)));
    /// ```
    fn until<C>(self, condition: C) -> Until<Self, C>
    where
        C: Future<Item=()>,
        Self::Error: From<C::Error>
    {
        Until::new(self, condition)
    }

    /// Adapts a stream to a future by taking the first successful item yielded by the stream. If
    /// the stream ends before yielding an `Ok` then all the errors that were yielded by the stream
    /// are returned in a vector.
    fn first_ok(self) -> FirstOk<Self> {
        FirstOk::new(self)
    }

    /// Removes the errors from this stream and log them. `description` is prepended to the log
    /// messages. The returned stream has error type `Void` since the errors have been removed.
    fn log_errors(self, level: log::Level, description: &'static str) -> LogErrors<Self>
    where
        Self::Error: Display
    {
        LogErrors::new(self, level, description)
    }

    /// For streams which can't fail (ie. which have error type `Void`), cast the error type to
    /// some inferred type.
    fn infallible<E>(self) -> Infallible<Self, E>
    where
        Self: Stream<Error=Void>
    {
        Infallible::new(self)
    }

    /// Returns a future which returns the next item from the stream, along with the stream itself.
    /// If the stream errors then just the error is returned. If the stream ends then the provided
    /// closure is used to produce an error value.
    fn next_or_else<F, E>(self, f: F) -> NextOrElse<Self, F>
    where
        F: FnOnce() -> E,
        E: From<Self::Error>,
    {
        NextOrElse::new(self, f)
    }

    /// Yields items from the stream and runs the provided callback when the stream finishes. The
    /// callback will also be run if the entire stream is dropped.
    fn finally<D>(self, on_drop: D) -> Finally<Self, D>
    where
        D: FnOnce()
    {
        Finally::new(self, on_drop)
    }

    /// Runs the stream for the given duration.
    fn with_timeout(self, duration: Duration) -> WithTimeout<Self> {
        WithTimeout::new(self, duration)
    }

    /// Runs the stream until the given timeout.
    fn with_timeout_at(self, instant: Instant) -> WithTimeout<Self> {
        WithTimeout::new_at(self, instant)
    }

    fn with_readiness_timeout(self, duration: Duration) -> WithReadinessTimeout<Self> {
        WithReadinessTimeout::new(self, duration)
    }
}

impl<T: Stream + Sized> StreamExt for T {}

