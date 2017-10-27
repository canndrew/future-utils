#[macro_use]
extern crate futures;
extern crate void;
#[macro_use]
extern crate log;
#[macro_use]
extern crate unwrap;

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
mod next_or_else;
mod finally;
pub mod mpsc;

pub use drop_notify::{drop_notify, DropNotify, DropNotice};
pub use until::Until;
pub use first_ok::FirstOk;
pub use log_errors::LogErrors;
pub use log_error::LogError;
pub use future_ext::FutureExt;
pub use stream_ext::StreamExt;
pub use infallible::Infallible;
pub use next_or_else::NextOrElse;
pub use finally::Finally;

pub type BoxFuture<T, E> = Box<Future<Item=T, Error=E>>;
pub type BoxStream<T, E> = Box<Stream<Item=T, Error=E>>;
pub type BoxSink<T, E> = Box<Sink<SinkItem=T, SinkError=E>>;

pub type IoFuture<T> = Box<Future<Item=T, Error=io::Error>>;
pub type IoStream<T> = Box<Stream<Item=T, Error=io::Error>>;
pub type IoSink<T> = Box<Sink<SinkItem=T, SinkError=io::Error>>;

