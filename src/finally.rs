use futures::{Async, Future, Stream};

/// Wraps a stream or future and runs a callback when the stream/future ends or when `Finally` is
/// dropped.
pub struct Finally<F, D>
where
    D: FnOnce(),
{
    inner: Option<OnDrop<F, D>>,
}

struct OnDrop<F, D>
where
    D: FnOnce(),
{
    future: F,
    on_drop: Option<D>,
}

impl<F, D> Drop for OnDrop<F, D>
where
    D: FnOnce(),
{
    fn drop(&mut self) {
        unwrap!(self.on_drop.take())()
    }
}

impl<F, D> Finally<F, D>
where
    D: FnOnce(),
{
    pub fn new(future: F, on_drop: D) -> Finally<F, D> {
        Finally {
            inner: Some(OnDrop {
                future: future,
                on_drop: Some(on_drop),
            }),
        }
    }
}

impl<F, D> Future for Finally<F, D>
where
    F: Future,
    D: FnOnce(),
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Result<Async<F::Item>, F::Error> {
        let mut on_drop = unwrap!(self.inner.take());
        match on_drop.future.poll()? {
            Async::Ready(x) => Ok(Async::Ready(x)),
            Async::NotReady => {
                self.inner = Some(on_drop);
                Ok(Async::NotReady)
            },
        }
    }
}

impl<S, D> Stream for Finally<S, D>
where
    S: Stream,
    D: FnOnce(),
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Result<Async<Option<S::Item>>, S::Error> {
        let mut on_drop = unwrap!(self.inner.take());
        match on_drop.future.poll()? {
            Async::Ready(x) => Ok(Async::Ready(x)),
            Async::NotReady => {
                self.inner = Some(on_drop);
                Ok(Async::NotReady)
            },
        }
    }
}

