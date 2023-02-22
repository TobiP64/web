// MIT License
//
// Copyright (c) 2019-2023 Tobias Pfeiffer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use {super::*, std::{io, pin::Pin, task::{Context, Poll}}};

pub struct AsyncStream<'a, T: traits::AsyncSharedConnection>(pub &'a T, pub StreamId);

impl<'a, T: traits::AsyncSharedConnection> AsyncStream<'a, T> {
	pub fn new(conn: &'a T, id: StreamId) -> Self {
		Self(conn, id)
	}
}

impl<'a, T: traits::AsyncSharedConnection> traits::AsyncStream for AsyncStream<'a, T> {
	fn poll_read_headers(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<Vec<Header>>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		match unsafe { Pin::new_unchecked(&**conn) }.poll_read(cx, *id, &mut []) {
			Poll::Pending                           => Poll::Pending,
			Poll::Ready(Ok(Read::Headers(headers))) => Poll::Ready(Ok(headers)),
			Poll::Ready(Ok(Read::HeadersDone))      => Poll::Ready(Ok(Vec::new())),
			Poll::Ready(Ok(_))                      => Poll::Ready(Err(io::Error::new(
				io::ErrorKind::Other, "invalid state"))),
			Poll::Ready(Err(e))                     => Poll::Ready(Err(e))
		}
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[Header]) -> Poll<std::io::Result<()>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**conn) }.poll_write_headers(cx, *id, headers)
	}
}

impl<'a, T: traits::AsyncSharedConnection> futures_lite::AsyncRead for AsyncStream<'a, T> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		match unsafe { Pin::new_unchecked(&**conn) }.poll_read(cx, *id, buf) {
			Poll::Pending                      => Poll::Pending,
			Poll::Ready(Ok(Read::Body(buf)))   => Poll::Ready(Ok(buf.len())),
			Poll::Ready(Ok(Read::Closed))      => Poll::Ready(Ok(0)),
			Poll::Ready(Ok(_))                 => Poll::Ready(Err(io::Error::new(
				io::ErrorKind::Other, "invalid state"))),
			Poll::Ready(Err(e))                => Poll::Ready(Err(e))
		}
	}
}

impl<'a, T: traits::AsyncSharedConnection> futures_lite::AsyncWrite for AsyncStream<'a, T> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**conn) }.poll_write_body(cx, *id, buf)
			.map(|v| v.map(|_| buf.len()))
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**conn) }.poll_flush(cx, *id)
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
		let Self(conn, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**conn) }.poll_close(cx, *id)
	}
}

pub struct StreamAdapter<'a, T: traits::Connection>(pub &'a mut T, pub StreamId);

impl<'a, T: traits::Connection> StreamAdapter<'a, T> {
	pub fn open(conn: &'a mut T) -> io::Result<Self> {
		match conn.open() {
			Ok(id) => Ok(Self(conn, id)),
			Err(e) => Err(e)
		}
	}
}

impl<'a, T: traits::Connection> io::Read for StreamAdapter<'a, T> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match self.0.read(buf) {
			Ok(Some((_, Read::Body(buf)))) => Ok(buf.len()),
			Ok(Some((_, Read::Closed)))    => Ok(0),
			Ok(_)                          => panic!("invalid state"),
			Err(e)                         => Err(e)
		}
	}
}

impl<'a, T: traits::Connection> io::Write for StreamAdapter<'a, T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0.write_body(self.1, buf)?;
		Ok(buf.len())
	}
	
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
		self.0.write_body(self.1, buf)
	}
	
	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl<'a, T: traits::Connection> traits::Stream for StreamAdapter<'a, T> {
	fn write_headers(&mut self, headers: &[Header]) -> io::Result<()> {
		self.0.write_headers(self.1, headers)
	}
	
	fn read_headers(&mut self) -> io::Result<Vec<Header>> {
		match self.0.read(&mut []) {
			Ok(Some((_, Read::Headers(buf)))) => Ok(buf),
			Ok(Some((_, Read::Closed)))       => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
			Ok(_)                             => panic!("invalid state"),
			Err(e)                            => Err(e)
		}
	}
	
	fn close(&mut self) -> io::Result<()> {
		self.0.close(self.1)
	}
}

pub struct AsyncStreamAdapter<'a, T: traits::AsyncConnection>(pub &'a mut T, pub StreamId);

impl<'a, T: traits::AsyncConnection> AsyncStreamAdapter<'a, T> {
	pub async fn open(conn: &'a mut T) -> io::Result<AsyncStreamAdapter<'a, T>> {
		match traits::AsyncConnectionExt::open(conn).await {
			Ok(id) => Ok(AsyncStreamAdapter(conn, id)),
			Err(e) => Err(e)
		}
	}
}

impl<'a, T: traits::AsyncConnection> futures_lite::io::AsyncRead for AsyncStreamAdapter<'a, T> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
		todo!()
	}
}

impl<'a, T: traits::AsyncConnection> futures_lite::io::AsyncWrite for AsyncStreamAdapter<'a, T> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
		todo!()
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		todo!()
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		todo!()
	}
}

impl<'a, T: traits::AsyncConnection> traits::AsyncStream for AsyncStreamAdapter<'a, T> {
	fn poll_read_headers(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Vec<Header>>> {
		todo!()
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[Header]) -> Poll<io::Result<()>> {
		todo!()
	}
}