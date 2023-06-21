use anyhow::Error;
use std::convert::TryInto;
use std::io::{self, Read, Write};

use crate::preview2::{HostInputStream, HostOutputStream, HostPollable, StreamState};

pub struct Stdin(std::io::Stdin);

pub fn stdin() -> Stdin {
    Stdin(std::io::stdin())
}

#[async_trait::async_trait]
impl HostInputStream for Stdin {
    async fn read(&mut self, buf: &mut [u8]) -> Result<(u64, StreamState), Error> {
        match Read::read(&mut self.0, buf) {
            Ok(0) => Ok((0, StreamState::Closed)),
            Ok(n) => Ok((n as u64, StreamState::Open)),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok((0, StreamState::Open)),
            Err(err) => Err(err.into()),
        }
    }
    async fn read_vectored<'a>(
        &mut self,
        bufs: &mut [io::IoSliceMut<'a>],
    ) -> Result<(u64, StreamState), Error> {
        match Read::read_vectored(&mut self.0, bufs) {
            Ok(0) => Ok((0, StreamState::Closed)),
            Ok(n) => Ok((n as u64, StreamState::Open)),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok((0, StreamState::Open)),
            Err(err) => Err(err.into()),
        }
    }
    /* this method can be implemented once `can_vector` stabilizes in std:
    fn is_read_vectored(&self) -> bool {
        Read::is_read_vectored(&mut self.0)
    }
    */

    async fn skip(&mut self, nelem: u64) -> Result<(u64, StreamState), Error> {
        let num = io::copy(&mut io::Read::take(&mut self.0, nelem), &mut io::sink())?;
        Ok((
            num,
            if num < nelem {
                StreamState::Closed
            } else {
                StreamState::Open
            },
        ))
    }

    fn pollable(&self) -> HostPollable {
        // TODO(elliottt): this can be a read with an empty buffer to check for ready, but on
        // windows there is a special function that needs to be called in a worker thread, as stdin
        // is special. There is already code in wasi-common for creating the worker thread, copy
        // that.
        HostPollable::new(|| Box::pin(async { todo!("pollable on stdin") }))
    }
}

macro_rules! wasi_output_stream_impl {
    ($ty:ty, $ident:ident) => {
        #[async_trait::async_trait]
        impl HostOutputStream for $ty {
            async fn write(&mut self, buf: &[u8]) -> Result<u64, Error> {
                let n = Write::write(&mut self.0, buf)?;
                Ok(n.try_into()?)
            }
            async fn write_vectored<'a>(&mut self, bufs: &[io::IoSlice<'a>]) -> Result<u64, Error> {
                let n = Write::write_vectored(&mut self.0, bufs)?;
                Ok(n.try_into()?)
            }
            /* this method can be implemented once `can_vector` stablizes in std
            fn is_write_vectored(&self) -> bool {
                Write::is_write_vectored(&mut self.0)
            }
            */
            async fn write_zeroes(&mut self, nelem: u64) -> Result<u64, Error> {
                let num = io::copy(&mut io::Read::take(io::repeat(0), nelem), &mut self.0)?;
                Ok(num)
            }

            fn pollable(&self) -> HostPollable {
                // TODO(elliottt): not clear how to implement this, but writing an empty buffer is
                // probably the right next step. It's not clear how stdout/stderr could not be
                // ready for writing.
                HostPollable::new(|| Box::pin(async { todo!("pollable on stdio, stderr writes") }))
            }
        }
    };
}

pub struct Stdout(std::io::Stdout);

pub fn stdout() -> Stdout {
    Stdout(std::io::stdout())
}
wasi_output_stream_impl!(Stdout, Stdout);

pub struct Stderr(std::io::Stderr);

pub fn stderr() -> Stderr {
    Stderr(std::io::stderr())
}
wasi_output_stream_impl!(Stderr, Stderr);

pub struct EmptyStream;

#[async_trait::async_trait]
impl HostInputStream for EmptyStream {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<(u64, StreamState), Error> {
        Ok((0, StreamState::Open))
    }

    fn pollable(&self) -> HostPollable {
        struct Never;

        impl std::future::Future for Never {
            type Output = anyhow::Result<()>;
            fn poll(
                self: std::pin::Pin<&mut Self>,
                _ctx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                std::task::Poll::Pending
            }
        }

        // This stream is never ready for reading.
        HostPollable::new(|| Box::pin(Never))
    }
}

#[async_trait::async_trait]
impl HostOutputStream for EmptyStream {
    async fn write(&mut self, buf: &[u8]) -> Result<u64, Error> {
        Ok(buf.len() as u64)
    }

    fn pollable(&self) -> HostPollable {
        // This stream is always ready for writing.
        HostPollable::new(|| Box::pin(async { Ok(()) }))
    }
}
