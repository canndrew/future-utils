//! channels in the futures-rs crate cannot error, yet they return () for their error type for some
//! stupid reason. This is a wrapper around futures-rs unbounded channels which removes the error.

use futures::{self, Stream, Async};
use void::Void;

pub use futures::sync::mpsc::{UnboundedSender, SendError};

#[derive(Debug)]
pub struct UnboundedReceiver<T> {
    inner: futures::sync::mpsc::UnboundedReceiver<T>,
}

pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = futures::sync::mpsc::unbounded();
    (tx, UnboundedReceiver { inner: rx })
}

impl<T> Stream for UnboundedReceiver<T> {
    type Item = T;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<Option<T>>, Void> {
        Ok(unwrap!(self.inner.poll()))
    }
}

