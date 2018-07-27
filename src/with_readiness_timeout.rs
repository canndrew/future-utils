use std::time::{Instant, Duration};
use Delay;
use futures::{Future, Stream, Async};
use void::ResultVoidExt;

pub struct WithReadinessTimeout<S> {
    stream: S,
    duration: Duration,
    delay: Delay,
}

impl<S> WithReadinessTimeout<S> {
    pub fn new(stream: S, duration: Duration) -> WithReadinessTimeout<S> {
        let delay = Delay::new(Instant::now() + duration);
        WithReadinessTimeout {
            stream, duration, delay,
        }
    }
}

impl<S: Stream> Stream for WithReadinessTimeout<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Result<Async<Option<S::Item>>, S::Error> {
        match self.stream.poll() {
            Ok(Async::Ready(Some(item))) => {
                self.delay.reset(Instant::now() + self.duration);
                Ok(Async::Ready(Some(item)))
            }
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => {
                match self.delay.poll().void_unwrap() {
                    Async::Ready(()) => Ok(Async::Ready(None)),
                    Async::NotReady => Ok(Async::NotReady),
                }
            },
            Err(e) => Err(e),
        }
    }
}

