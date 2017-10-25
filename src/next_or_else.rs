use futures::{Async, Future, Stream};

pub struct NextOrElse<S, F> {
    inner: Option<Inner<S, F>>,
}

struct Inner<S, F> {
    stream: S,
    f: F,
}

impl<S, F> NextOrElse<S, F> {
    pub fn new(stream: S, f: F) -> NextOrElse<S, F> {
        let inner = Inner {
            stream, f,
        };
        NextOrElse {
            inner: Some(inner),
        }
    }
}

impl<S, F> Future for NextOrElse<S, F>
where
    S: Stream,
    F: FnOnce() -> S::Error,
{
    type Item = (S::Item, S);
    type Error = S::Error;

    fn poll(&mut self) -> Result<Async<(S::Item, S)>, S::Error> {
        let mut inner = self.inner.take().unwrap();
        match inner.stream.poll() {
            Err(e) => Err(e),
            Ok(Async::NotReady) => {
                self.inner = Some(inner);
                Ok(Async::NotReady)
            },
            Ok(Async::Ready(None)) => Err((inner.f)()),
            Ok(Async::Ready(Some(x))) => Ok(Async::Ready((x, inner.stream))),
        }
    }
}

