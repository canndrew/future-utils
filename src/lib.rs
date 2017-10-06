#[macro_use]
extern crate futures;
extern crate void;
#[macro_use]
extern crate log;

use std::io;
use futures::{Future, Stream, Sink};

mod drop_notify;
mod until;
mod future_ext;
mod stream_ext;
mod first_ok;
mod log_errors;
mod log_error;
mod infallible;

pub use drop_notify::{drop_notify, DropNotify, DropNotice};
pub use until::Until;
pub use log_errors::LogErrors;
pub use future_ext::FutureExt;
pub use stream_ext::StreamExt;
pub use infallible::Infallible;

pub type BoxFuture<T, E> = Box<Future<Item=T, Error=E>>;
pub type BoxStream<T, E> = Box<Stream<Item=T, Error=E>>;
pub type BoxSink<T, E> = Box<Sink<SinkItem=T, SinkError=E>>;

pub type IoFuture<T> = Box<Future<Item=T, Error=io::Error>>;
pub type IoStream<T> = Box<Stream<Item=T, Error=io::Error>>;
pub type IoSink<T> = Box<Sink<SinkItem=T, SinkError=io::Error>>;

