use std::any::Any;
use std::panic;
use futures::{Future, Async};
use void::Void;

/// Propogates the result of a `.catch_unwind`, panicking if the future resolves to an `Err`
pub struct ResumeUnwind<F> {
    future: F,
}

impl<F> ResumeUnwind<F> {
    pub fn new(future: F) -> ResumeUnwind<F> {
        ResumeUnwind {
            future,
        }
    }
}

impl<F> Future for ResumeUnwind<F>
where
    F: Future<Error=Box<Any + Send + 'static>>,
{
    type Item = F::Item;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<F::Item>, Void> {
        match self.future.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => panic::resume_unwind(e),
        }
    }
}

