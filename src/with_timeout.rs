use std::time::{Duration, Instant};
use futures::{Async, Future, Stream};
use delay::Delay;
use void::ResultVoidExt;

pub struct WithTimeout<F> {
    inner: F,
    delay: Delay,
}

impl<F> WithTimeout<F> {
    /// Creates a new `WithTimeout` which runs `inner` for the given duration.
    pub fn new(inner: F, duration: Duration) -> WithTimeout<F> {
        let deadline = Instant::now() + duration;
        WithTimeout {
            inner: inner,
            delay: Delay::new(deadline),
        }
    }

    /// Creates a new `WithTimeout` which runs `inner` until the given instant.
    pub fn new_at(inner: F, instant: Instant) -> WithTimeout<F> {
        WithTimeout {
            inner: inner,
            delay: Delay::new(instant),
        }
    }

    /// Unpack the `WithTimeout`, returning the inner future or stream.
    pub fn into_inner(self) -> F {
        self.inner
    }
}

impl<F> Future for WithTimeout<F>
where
    F: Future
{
    type Item = Option<F::Item>;
    type Error = F::Error;

    fn poll(&mut self) -> Result<Async<Option<F::Item>>, F::Error> {
        if let Async::Ready(()) = self.delay.poll().void_unwrap() {
            return Ok(Async::Ready(None));
        }

        match self.inner.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(Some(x))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

impl<F> Stream for WithTimeout<F>
where
    F: Stream
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Result<Async<Option<F::Item>>, F::Error> {
        if let Async::Ready(()) = self.delay.poll().void_unwrap() {
            return Ok(Async::Ready(None));
        }

        self.inner.poll()
    }
}

