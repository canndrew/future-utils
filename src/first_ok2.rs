use futures::{Future, Async};
use std::mem;

pub struct FirstOk2<A: Future, B: Future> {
    inner: FirstOk2Inner<A, B>,
}

impl<A: Future, B: Future> FirstOk2<A, B> {
    pub fn new(a: A, b: B) -> FirstOk2<A, B> {
        FirstOk2 {
            inner: FirstOk2Inner::BothRunning(a, b),
        }
    }
}

enum FirstOk2Inner<A: Future, B: Future> {
    BothRunning(A, B),
    RunningA(A, B::Error),
    RunningB(A::Error, B),
    Finished,
}

impl<A, B> Future for FirstOk2<A, B>
where
    A: Future,
    B: Future<Item=A::Item>
{
    type Item = A::Item;
    type Error = (A::Error, B::Error);

    fn poll(&mut self) -> Result<Async<A::Item>, (A::Error, B::Error)> {
        let inner = mem::replace(&mut self.inner, FirstOk2Inner::Finished);
        match inner {
            FirstOk2Inner::BothRunning(mut a, mut b) => {
                match a.poll() {
                    Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
                    Ok(Async::NotReady) => {
                        match b.poll() {
                            Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
                            Ok(Async::NotReady) => {
                                self.inner = FirstOk2Inner::BothRunning(a, b);
                                Ok(Async::NotReady)
                            },
                            Err(eb) => {
                                self.inner = FirstOk2Inner::RunningA(a, eb);
                                Ok(Async::NotReady)
                            },
                        }
                    },
                    Err(ea) => {
                        match b.poll() {
                            Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
                            Ok(Async::NotReady) => {
                                self.inner = FirstOk2Inner::RunningB(ea, b);
                                Ok(Async::NotReady)
                            },
                            Err(eb) => Err((ea, eb)),
                        }
                    },
                }
            },
            FirstOk2Inner::RunningA(mut a, eb) => {
                match a.poll() {
                    Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
                    Ok(Async::NotReady) => {
                        self.inner = FirstOk2Inner::RunningA(a, eb);
                        Ok(Async::NotReady)
                    },
                    Err(ea) => Err((ea, eb)),
                }
            },
            FirstOk2Inner::RunningB(ea, mut b) => {
                match b.poll() {
                    Ok(Async::Ready(x)) => Ok(Async::Ready(x)),
                    Ok(Async::NotReady) => {
                        self.inner = FirstOk2Inner::RunningB(ea, b);
                        Ok(Async::NotReady)
                    },
                    Err(eb) => Err((ea, eb)),
                }
            },
            FirstOk2Inner::Finished => {
                panic!("poll() called on FirstOk2 which has already finished");
            },
        }
    }
}

