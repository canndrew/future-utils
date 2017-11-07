use std::time::{Duration, Instant};
use tokio_core::reactor::Handle;
use futures::{Async, Future, Stream};
use timeout::Timeout;
use void::ResultVoidExt;

pub struct WithTimeout<F> {
    inner: F,
    timeout: Timeout,
}

impl<F> WithTimeout<F> {
    /// Creates a new `WithTimeout` which runs `inner` for the given duration.
    pub fn new(inner: F, duration: Duration, handle: &Handle) -> WithTimeout<F> {
        WithTimeout {
            inner: inner,
            timeout: Timeout::new(duration, handle),
        }
    }

    /// Creates a new `WithTimeout` which runs `inner` until the given instant.
    pub fn new_at(inner: F, instant: Instant, handle: &Handle) -> WithTimeout<F> {
        WithTimeout {
            inner: inner,
            timeout: Timeout::new_at(instant, handle),
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
        if let Async::Ready(()) = self.timeout.poll().void_unwrap() {
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
        if let Async::Ready(()) = self.timeout.poll().void_unwrap() {
            return Ok(Async::Ready(None));
        }

        self.inner.poll()
    }
}

