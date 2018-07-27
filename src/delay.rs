//! tokio timeouts cannot actually error, but for some stupid reason tokio decided to put an io
//! error on them. This is just a wrapper which removes the error and makes error handling easier
//! when using timeouts.

use futures::{Async, Future};
use std::time::Instant;
use void::Void;
use tokio;

pub struct Delay {
    inner: tokio::timer::Delay,
}

impl Delay {
    pub fn new(at: Instant) -> Delay {
        Delay {
            inner: tokio::timer::Delay::new(at),
        }
    }

    pub fn reset(&mut self, at: Instant) {
        self.inner.reset(at)
    }
}

impl Future for Delay {
    type Item = ();
    type Error = Void;

    fn poll(&mut self) -> Result<Async<()>, Void> {
        Ok(unwrap!(self.inner.poll()))
    }
}

