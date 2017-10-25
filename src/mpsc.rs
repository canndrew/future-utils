//! channels in the futures-rs crate cannot error, yet they return () for their error type for some
//! stupid reason. This is a wrapper around futures-rs unbounded channels which removes the error.

use futures::{self, Stream, Sink, Async, AsyncSink};
use void::Void;

pub struct UnboundedSender<T> {
    inner: futures::sync::mpsc::UnboundedSender<T>,
}

pub struct UnboundedReceiver<T> {
    inner: futures::sync::mpsc::UnboundedReceiver<T>,
}

pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = futures::sync::mpsc::unbounded();
    (UnboundedSender { inner: tx }, UnboundedReceiver { inner: rx })
}

impl<T> Sink for UnboundedSender<T> {
    type SinkItem = T;
    type SinkError = Void;

    fn start_send(&mut self, item: T) -> Result<AsyncSink<T>, Void> {
        Ok(unwrap!(self.inner.start_send(item)))
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Void> {
        Ok(unwrap!(self.inner.poll_complete()))
    }
}

impl<T> Stream for UnboundedReceiver<T> {
    type Item = T;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<Option<T>>, Void> {
        Ok(unwrap!(self.inner.poll()))
    }
}

