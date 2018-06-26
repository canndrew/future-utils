use bytes::{Bytes, BytesMut};
use tokio_io::{AsyncRead, AsyncWrite};
use futures::{Stream, Sink, Async, AsyncSink};
use std::{io, mem};

fn zeros(n: usize) -> BytesMut {
    let mut ret = BytesMut::with_capacity(n);
    unsafe {
        ret.set_len(n);
        for i in 0..n {
            ret[i] = 0;
        }
    }
    ret
}

/// An alternative to tokio_io's `Framed` which doesn't internally buffer data.
/// This gives it much lower performance but means that you can use `.into_inner()` without losing
/// data.
pub struct FramedUnbuffered<T> {
    stream: T,
    read_state: ReadState,
    write_state: WriteState,
}

impl<T> FramedUnbuffered<T> {
    pub fn new(stream: T) -> FramedUnbuffered<T> {
        FramedUnbuffered {
            stream,
            read_state: ReadState::ReadingSize {
                bytes_read: 0,
                size_buffer: [0u8; 4],
            },
            write_state: WriteState::WaitingForInput,
        }
    }

    pub fn into_inner(self) -> Option<T> {
        if let ReadState::ReadingSize { bytes_read: 0, .. } = self.read_state {
            if let WriteState::WaitingForInput = self.write_state {
                return Some(self.stream);
            }
        }
        None
    }
}

enum ReadState {
    Invalid,
    ReadingSize {
        bytes_read: u8,
        size_buffer: [u8; 4],
    },
    ReadingData {
        bytes_read: u32,
        data_buffer: BytesMut,
    },
}

enum WriteState {
    Invalid,
    WaitingForInput,
    WritingSize {
        size_buffer: [u8; 4],
        data_buffer: Bytes,
        bytes_written: u8,
    },
    WritingData {
        data_buffer: Bytes,
        bytes_written: u32,
    }
}

impl<T> Stream for FramedUnbuffered<T>
where
    T: AsyncRead,
{
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> io::Result<Async<Option<BytesMut>>> {
        loop {
            let read_state = mem::replace(&mut self.read_state, ReadState::Invalid);
            match read_state {
                ReadState::Invalid => unreachable!(),
                ReadState::ReadingSize { mut bytes_read, mut size_buffer } => {
                    match self.stream.read(&mut size_buffer[(bytes_read as usize)..]) {
                        Ok(n) => {
                            if n == 0 {
                                if bytes_read == 0 {
                                    return Ok(Async::Ready(None));
                                } else {
                                    return Err(io::Error::from(io::ErrorKind::BrokenPipe));
                                }
                            }
                            bytes_read += n as u8;
                            if bytes_read == 4 {
                                let len =
                                    ((size_buffer[0] as u32) << 24) +
                                    ((size_buffer[1] as u32) << 16) +
                                    ((size_buffer[2] as u32) << 8) +
                                    (size_buffer[3] as u32);
                                self.read_state = ReadState::ReadingData {
                                    bytes_read: 0,
                                    data_buffer: zeros(len as usize),
                                };
                            } else {
                                self.read_state = ReadState::ReadingSize {
                                    bytes_read, size_buffer,
                                };
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            self.read_state = ReadState::ReadingSize {
                                bytes_read, size_buffer,
                            };
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e),
                    }
                },
                ReadState::ReadingData { mut bytes_read, mut data_buffer } => {
                    match self.stream.read(&mut data_buffer[(bytes_read as usize)..]) {
                        Ok(n) => {
                            if n == 0 {
                                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
                            }
                            bytes_read += n as u32;
                            if bytes_read == data_buffer.len() as u32 {
                                self.read_state = ReadState::ReadingSize {
                                    bytes_read: 0,
                                    size_buffer: [0u8; 4],
                                };
                                return Ok(Async::Ready(Some(data_buffer)));
                            }
                            else {
                                self.read_state = ReadState::ReadingData {
                                    bytes_read, data_buffer,
                                }
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            self.read_state = ReadState::ReadingData {
                                bytes_read, data_buffer,
                            };
                            return Ok(Async::NotReady);
                        }
                        Err(e) => return Err(e),
                    }
                },
            }
        }
    }
}

impl<T> Sink for FramedUnbuffered<T>
where
    T: AsyncWrite,
{
    type SinkItem = Bytes;
    type SinkError = io::Error;

    fn start_send(&mut self, data_buffer: Bytes) -> io::Result<AsyncSink<Bytes>> {
        let write_state = mem::replace(&mut self.write_state, WriteState::Invalid);
        match write_state {
            WriteState::Invalid => unreachable!(),
            WriteState::WaitingForInput => {
                let len = data_buffer.len() as u32;
                let size_buffer = [
                    (len >> 24) as u8,
                    ((len >> 16) & 0xff) as u8,
                    ((len >> 8) & 0xff) as u8,
                    (len & 0xff) as u8,
                ];
                self.write_state = WriteState::WritingSize {
                    bytes_written: 0,
                    size_buffer,
                    data_buffer,
                };
                return Ok(AsyncSink::Ready);
            },
            WriteState::WritingSize { .. } | WriteState::WritingData { .. } => {
                self.write_state = write_state;
                return Ok(AsyncSink::NotReady(data_buffer));
            },
        }
    }

    fn poll_complete(&mut self) -> io::Result<Async<()>> {
        loop {
            let write_state = mem::replace(&mut self.write_state, WriteState::Invalid);
            match write_state {
                WriteState::Invalid => unreachable!(),
                WriteState::WaitingForInput => {
                    self.write_state = WriteState::WaitingForInput;
                    return Ok(Async::Ready(()));
                },
                WriteState::WritingSize { size_buffer, data_buffer, mut bytes_written } => {
                    match self.stream.write(&size_buffer[(bytes_written as usize)..]) {
                        Ok(n) => {
                            if n == 0 {
                                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
                            }
                            bytes_written += n as u8;
                            if bytes_written == 4 {
                                self.write_state = WriteState::WritingData {
                                    data_buffer,
                                    bytes_written: 0,
                                };
                            } else {
                                self.write_state = WriteState::WritingSize {
                                    size_buffer, data_buffer, bytes_written,
                                }
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            self.write_state = WriteState::WritingSize {
                                size_buffer, data_buffer, bytes_written,
                            };
                            return Ok(Async::NotReady);
                        },
                        Err(e) => return Err(e),
                    }
                },
                WriteState::WritingData { data_buffer, mut bytes_written } => {
                    match self.stream.write(&data_buffer[(bytes_written as usize)..]) {
                        Ok(n) => {
                            if n == 0 {
                                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
                            }
                            bytes_written += n as u32;
                            if bytes_written == data_buffer.len() as u32 {
                                self.write_state = WriteState::WaitingForInput;
                            } else {
                                self.write_state = WriteState::WritingData {
                                    data_buffer, bytes_written,
                                }
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            self.write_state = WriteState::WritingData {
                                data_buffer, bytes_written,
                            };
                            return Ok(Async::NotReady);
                        },
                        Err(e) => return Err(e),
                    }
                },
            }
        }
    }
}

