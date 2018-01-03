use futures::{Async, Future};
use std::mem;

pub struct WhileDriving<A, B: Future> {
    inner: WhileDrivingInner<A, B>,
}

enum WhileDrivingInner<A, B: Future> {
    Driving(A, B),
    Drove(A, Result<B::Item, B::Error>),
    Finished,
}

impl<A, B: Future> WhileDriving<A, B> {
    pub fn new(a: A, b: B) -> WhileDriving<A, B> {
        WhileDriving {
            inner: WhileDrivingInner::Driving(a, b),
        }
    }
}

impl<A, B> Future for WhileDriving<A, B>
where
    A: Future,
    B: Future,
{
    type Item = (A::Item, Finish<B>);
    type Error = (A::Error, Finish<B>);

    fn poll(&mut self) -> Result<Async<(A::Item, Finish<B>)>, (A::Error, Finish<B>)> {
        let inner = mem::replace(&mut self.inner, WhileDrivingInner::Finished);
        match inner {
            WhileDrivingInner::Driving(mut a, mut b) => {
                match a.poll() {
                    Ok(Async::Ready(x)) => {
                        let finish = Finish {
                            inner: FinishInner::Running(b),
                        };
                        Ok(Async::Ready((x, finish)))
                    },
                    Ok(Async::NotReady) => {
                        match b.poll() {
                            Ok(Async::Ready(x)) => {
                                self.inner = WhileDrivingInner::Drove(a, Ok(x));
                            },
                            Ok(Async::NotReady) => {
                                self.inner = WhileDrivingInner::Driving(a, b);
                            },
                            Err(e) => {
                                self.inner = WhileDrivingInner::Drove(a, Err(e));
                            },
                        }
                        Ok(Async::NotReady)
                    },
                    Err(e) => {
                        let finish = Finish {
                            inner: FinishInner::Running(b),
                        };
                        Err((e, finish))
                    },
                }
            },
            WhileDrivingInner::Drove(mut a, res) => {
                match a.poll() {
                    Ok(Async::Ready(x)) => {
                        let finish = Finish {
                            inner: FinishInner::Ran(res),
                        };
                        Ok(Async::Ready((x, finish)))
                    },
                    Ok(Async::NotReady) => {
                        self.inner = WhileDrivingInner::Drove(a, res);
                        Ok(Async::NotReady)
                    },
                    Err(e) => {
                        let finish = Finish {
                            inner: FinishInner::Ran(res),
                        };
                        Err((e, finish))
                    },
                }
            },
            WhileDrivingInner::Finished => {
                panic!("poll() called on WhileDriving which has already finished");
            },
        }
    }
}

pub struct Finish<B: Future> {
    inner: FinishInner<B>,
}

enum FinishInner<B: Future> {
    Running(B),
    Ran(Result<B::Item, B::Error>),
    Finished,
}

impl<B: Future> Future for Finish<B> {
    type Item = B::Item;
    type Error = B::Error;

    fn poll(&mut self) -> Result<Async<B::Item>, B::Error> {
        let inner = mem::replace(&mut self.inner, FinishInner::Finished);
        match inner {
            FinishInner::Running(mut b) => {
                match b.poll() {
                    Ok(Async::Ready(x)) => {
                        Ok(Async::Ready(x))
                    },
                    Ok(Async::NotReady) => {
                        self.inner = FinishInner::Running(b);
                        Ok(Async::NotReady)
                    },
                    Err(e) => {
                        Err(e)
                    },
                }
            },
            FinishInner::Ran(res) => Ok(Async::Ready(res?)),
            FinishInner::Finished => {
                panic!("poll() called on Finish which has already finished");
            },
        }
    }
}

