use std::fmt::Display;
use futures::{Async, Stream};
use void::Void;
use log::LogLevel;

/// Removes the errors from a stream and logs them.
pub struct LogErrors<S> {
    stream: S,
    level: LogLevel,
    description: &'static str,
}

impl<S> LogErrors<S> {
    pub fn new(
        stream: S,
        level: LogLevel,
        description: &'static str,
    ) -> LogErrors<S> {
        LogErrors {
            stream,
            level,
            description,
        }
    }
}

impl<S> Stream for LogErrors<S>
where
    S: Stream,
    S::Error: Display,
{
    type Item = S::Item;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<Option<S::Item>>, Void> {
        match self.stream.poll() {
            Ok(x) => Ok(x),
            Err(e) => {
                log!(self.level, "{}: {}", self.description, e);
                Ok(Async::NotReady)
            },
        }
    }
}

