use futures::{Stream, Sink, Async, AsyncSink};
use futures::sync::mpsc::SendError;
use void::Void;
use mpsc::{self, UnboundedSender, UnboundedReceiver};

#[derive(Debug)]
pub struct UnboundedBiChannel<T> {
    tx: UnboundedSender<T>,
    rx: UnboundedReceiver<T>,
}

impl<T> UnboundedBiChannel<T> {
    pub fn unbounded_send(&self, val: T) -> Result<(), SendError<T>> {
        self.tx.unbounded_send(val)
    }
}

impl<T> Sink for UnboundedBiChannel<T> {
    type SinkItem = T;
    type SinkError = SendError<T>;

    fn start_send(&mut self, item: T) -> Result<AsyncSink<T>, SendError<T>> {
        self.tx.start_send(item)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, SendError<T>> {
        self.tx.poll_complete()
    }
}

impl<T> Stream for UnboundedBiChannel<T> {
    type Item = T;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<Option<T>>, Void> {
        self.rx.poll()
    }
}

pub fn unbounded<T>() -> (UnboundedBiChannel<T>, UnboundedBiChannel<T>) {
    let (tx0, rx0) = mpsc::unbounded();
    let (tx1, rx1) = mpsc::unbounded();

    (
        UnboundedBiChannel {
            tx: tx0,
            rx: rx1,
        },
        UnboundedBiChannel {
            tx: tx1,
            rx: rx0,
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio;
    use futures::{Future, Stream};

    #[test]
    fn test() {
        let (ch0, ch1) = unbounded();

        let res = tokio::executor::current_thread::block_on_all({
            let data = 123u32;

            ch0
            .send(data)
            .map_err(|_| panic!("oh no"))
            .and_then(move |ch0| {
                ch1
                .into_future()
                .map_err(|_| panic!("oh no"))
                .and_then(move |(msg_opt, ch1)| {
                    let msg = unwrap!(msg_opt);

                    ch1
                    .send(msg)
                    .map_err(|_| panic!("oh no"))
                    .and_then(move |_ch1| {
                        ch0
                        .into_future()
                        .map_err(|_| panic!("oh no"))
                        .map(move |(msg_opt, _ch0)| {
                            let msg = unwrap!(msg_opt);
                            assert_eq!(msg, data);
                        })
                    })
                })
            })
        });

        unwrap!(res)
    }
}

