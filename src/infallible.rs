use std::marker::PhantomData;
use futures::{Async, Future, Stream};
use void::{self, Void};

/// Wraps a future or stream which can't fail (ie. has error type `Void`) and casts the error type
/// to some inferred type.
pub struct Infallible<F, E> {
    inner: F,
    _ph: PhantomData<E>,
}

impl<F, E> Infallible<F, E> {
    pub fn new(inner: F) -> Infallible<F, E> {
        Infallible {
            inner: inner,
            _ph: PhantomData,
        }
    }
}

impl<F, E> Future for Infallible<F, E>
where
    F: Future<Error=Void>
{
    type Item = F::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<F::Item>, E> {
        self.inner.poll().map_err(|e| void::unreachable(e))
    }
}

impl<F, E> Stream for Infallible<F, E>
where
    F: Stream<Error=Void>
{
    type Item = F::Item;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Option<F::Item>>, E> {
        self.inner.poll().map_err(|e| void::unreachable(e))
    }
}

