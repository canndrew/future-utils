use futures::{Future, Async};
use futures::sync::oneshot;
use std::{thread, panic};
use void::Void;

/// Wraps a synchronous function into a future by running it in its own thread. Created using
/// [`thread_future`].
pub struct ThreadFuture<R> {
    join_handle: Option<thread::JoinHandle<()>>,
    rx: oneshot::Receiver<R>,
}

/// Run a synchronous function in a separate thread and return its result as a `Future`.
///
/// # Note
///
/// If the given function panics then so will this future.
pub fn thread_future<F, R>(f: F) -> ThreadFuture<R>
where
    R: Send + 'static,
    F: FnOnce() -> R + Send + 'static
{
    let (tx, rx) = oneshot::channel();
    let join_handle = Some(thread::spawn(|| {
        let x = f();
        let _ = tx.send(x);
    }));
    ThreadFuture {
        rx,
        join_handle,
    }
}

impl<R> Future for ThreadFuture<R> {
    type Item = R;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<R>, Void> {
        match self.rx.poll() {
            Ok(x) => Ok(x),
            Err(oneshot::Canceled) => {
                // The thread must have died. Get the error.
                match unwrap!(self.join_handle.take()).join() {
                    Ok(()) => unreachable!(),
                    Err(e) => panic::resume_unwind(e),
                }
            },
        }
    }
}

