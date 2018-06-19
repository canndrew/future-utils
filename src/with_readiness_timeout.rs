use std::time::{Instant, Duration};
use Timeout;
use tokio_core::reactor::Handle;
use futures::{Future, Stream, Async};
use void::ResultVoidExt;

pub struct WithReadinessTimeout<S> {
    stream: S,
    duration: Duration,
    timeout: Timeout,
}

impl<S> WithReadinessTimeout<S> {
    pub fn new(stream: S, duration: Duration, handle: &Handle) -> WithReadinessTimeout<S> {
        let timeout = Timeout::new(duration, handle);
        WithReadinessTimeout {
            stream, duration, timeout,
        }
    }
}

impl<S: Stream> Stream for WithReadinessTimeout<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Result<Async<Option<S::Item>>, S::Error> {
        match self.stream.poll() {
            Ok(Async::Ready(Some(item))) => {
                self.timeout.reset(Instant::now() + self.duration);
                Ok(Async::Ready(Some(item)))
            }
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => {
                match self.timeout.poll().void_unwrap() {
                    Async::Ready(()) => Ok(Async::Ready(None)),
                    Async::NotReady => Ok(Async::NotReady),
                }
            },
            Err(e) => Err(e),
        }
    }
}

