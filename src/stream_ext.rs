use std::fmt::Display;
use futures::{Future, Stream};
use log::LogLevel;
use void::Void;

use until::Until;
use first_ok::FirstOk;
use log_errors::LogErrors;
use infallible::Infallible;
use BoxStream;

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

    /// Run this stream until some condition is met. `condition` is a future which returns `()`,
    /// after which this stream will be finished.
    ///
    /// # Example
    /// ```rust
    /// let my_stream_with_timeout = my_stream.until(Timeout::new(Duration::from_secs(1)));
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
    fn log_errors(self, level: LogLevel, description: &'static str) -> LogErrors<Self>
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
}

impl<T: Stream + Sized> StreamExt for T {}

