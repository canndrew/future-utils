use futures::{Async, Future, Stream};

/// Runs a stream or future until some condition is met.
pub struct Until<T, C> {
    orig: T,
    condition: C,
}

impl<T, C> Until<T, C> {
    pub fn new(orig: T, condition: C) -> Until<T, C> {
        Until {
            orig,
            condition,
        }
    }
}

impl<T, C> Future for Until<T, C>
where
    T: Future,
    C: Future<Item=()>,
    T::Error: From<C::Error>
{
    type Item = Option<T::Item>;
    type Error = T::Error;

    fn poll(&mut self) -> Result<Async<Option<T::Item>>, T::Error> {
        if let Async::Ready(()) = self.condition.poll()? {
            return Ok(Async::Ready(None));
        }

        let res = try_ready!(self.orig.poll());
        Ok(Async::Ready(Some(res)))
    }
}

impl<T, C> Stream for Until<T, C>
where
    T: Stream,
    C: Future<Item=()>,
    T::Error: From<C::Error>
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Result<Async<Option<T::Item>>, T::Error> {
        if let Async::Ready(()) = self.condition.poll()? {
            return Ok(Async::Ready(None));
        }

        self.orig.poll()
    }
}

