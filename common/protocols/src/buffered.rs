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

use std::{io, pin::Pin, task::{Context, Poll}};

#[cfg(feature = "smol")]
use std::future::Future;

const DEFAULT_BUF_LEN: usize = 0x1000;

pub struct BufStream<T: io::Read + io::Write>(io::BufReader<InternalBufWriter<T>>);

struct InternalBufWriter<T: io::Read + io::Write>(io::BufWriter<T>);

impl<T: io::Read + io::Write> io::Read for InternalBufWriter<T> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.0.get_mut().read(buf)
	}
	
	fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut]) -> io::Result<usize> {
		self.0.get_mut().read_vectored(bufs)
	}
	
	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
		self.0.get_mut().read_to_end(buf)
	}
	
	fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
		self.0.get_mut().read_to_string(buf)
	}
	
	fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
		self.0.get_mut().read_exact(buf)
	}
}

impl<T: io::Read + io::Write> BufStream<T> {
	pub fn new(inner: T) -> Self {
		Self(io::BufReader::new(InternalBufWriter(io::BufWriter::new(inner))))
	}
	
	pub fn with_capacity(capacity: usize, inner: T) -> Self {
		Self(io::BufReader::with_capacity(capacity, InternalBufWriter(io::BufWriter::with_capacity(capacity, inner))))
	}
}

impl<T: io::Read + io::Write> io::Read for BufStream<T> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.0.read(buf)
	}
	
	fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
		self.0.read_vectored(bufs)
	}
	
	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
		self.0.read_to_end(buf)
	}
	
	fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
		self.0.read_to_string(buf)
	}
	
	fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
		self.0.read_exact(buf)
	}
}

impl<T: io::Read + io::Write> io::BufRead for BufStream<T> {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		self.0.fill_buf()
	}
	
	fn consume(&mut self, amt: usize) {
		self.0.consume(amt)
	}
	
	fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
		self.0.read_until(byte, buf)
	}
	
	fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		self.0.read_line(buf)
	}
}

impl<T: io::Read + io::Write> io::Write for BufStream<T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0.get_mut().0.write(buf)
	}
	
	fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> {
		self.0.get_mut().0.write_vectored(bufs)
	}
	
	fn flush(&mut self) -> io::Result<()> {
		self.0.get_mut().0.flush()
	}
	
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
		self.0.get_mut().0.write_all(buf)
	}
	
	fn write_fmt(&mut self, fmt: std::fmt::Arguments) -> io::Result<()> {
		self.0.get_mut().0.write_fmt(fmt)
	}
}

pub struct AsyncBufStream<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite>(
	futures_lite::io::BufReader<InternalAsyncBufWriter<T>>);

pub struct InternalAsyncBufWriter<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite>(
	futures_lite::io::BufWriter<T>);

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> futures_lite::io::AsyncRead for InternalAsyncBufWriter<T> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| v.0.get_mut()) }.poll_read(cx, buf)
	}
	
	fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [io::IoSliceMut<'_>]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| v.0.get_mut()) }.poll_read_vectored(cx, bufs)
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> AsyncBufStream<T> {
	pub fn new(inner: T) -> Self {
		Self(futures_lite::io::BufReader::new(
			InternalAsyncBufWriter(futures_lite::io::BufWriter::new(inner))))
	}
	
	pub fn with_capacity(capacity: usize, inner: T) -> Self {
		Self(futures_lite::io::BufReader::with_capacity(
			capacity, InternalAsyncBufWriter(futures_lite::io::BufWriter::with_capacity(capacity, inner))))
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> futures_lite::io::AsyncRead for AsyncBufStream<T> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_read(cx, buf)
	}
	
	fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [io::IoSliceMut<'_>]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_read_vectored(cx, bufs)
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> futures_lite::io::AsyncBufRead for AsyncBufStream<T> {
	fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
		unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_fill_buf(cx)
	}
	
	fn consume(self: Pin<&mut Self>, amt: usize) {
		unsafe { self.map_unchecked_mut(|Self(v)| v) }.consume(amt)
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> futures_lite::io::AsyncWrite for AsyncBufStream<T> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut v.get_mut().0) }.poll_write(cx, buf)
	}
	
	fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[io::IoSlice<'_>]) -> Poll<io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut v.get_mut().0) }.poll_write_vectored(cx, bufs)
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut v.get_mut().0) }.poll_flush(cx)
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut v.get_mut().0) }.poll_close(cx)
	}
}

pub struct Connector<T: crate::utils::Connector> where T::Connection: io::Read + io::Write {
	inner:   T,
	buf_len: usize
}

impl<T: crate::utils::Connector> Connector<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T) -> Self {
		Self::with_capacity(inner, DEFAULT_BUF_LEN)
	}
	
	pub fn with_capacity(inner: T, capacity: usize) -> Self {
		Self { inner, buf_len: capacity }
	}
}

impl<T: crate::utils::Connector> crate::utils::Connector for Connector<T> where T::Connection: io::Read + io::Write {
	type Connection = BufStream<T::Connection>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.inner.connect()
			.map(|stream| BufStream::with_capacity(self.buf_len, stream))
	}
}

pub struct Acceptor<T: crate::utils::Acceptor> where T::Connection: io::Read + io::Write {
	inner:   T,
	buf_len: usize
}

impl<T: crate::utils::Acceptor> Acceptor<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T) -> Self {
		Self::with_capacity(inner, DEFAULT_BUF_LEN)
	}
	
	pub fn with_capacity(inner: T, capacity: usize) -> Self {
		Self { inner, buf_len: capacity }
	}
}

impl<T: crate::utils::Acceptor> crate::utils::Acceptor for Acceptor<T> where T::Connection: io::Read + io::Write {
	type Connection = BufStream<T::Connection>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.inner.accept()
			.map(|stream| BufStream::with_capacity(self.buf_len, stream))
	}
}

#[cfg(feature = "smol")]
pub struct AsyncConnector<T: crate::utils::AsyncConnector> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	inner:   T,
	buf_len: usize
}

#[cfg(feature = "smol")]
impl<T: crate::utils::AsyncConnector> AsyncConnector<T> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self::with_capacity(inner, DEFAULT_BUF_LEN)
	}
	
	pub fn with_capacity(inner: T, capacity: usize) -> Self {
		Self { inner, buf_len: capacity }
	}
}

#[cfg(feature = "smol")]
impl<T: crate::utils::AsyncConnector> crate::utils::AsyncConnector for AsyncConnector<T> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncBufStream<T::Connection>;
	//type Future = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		let buf_len = self.buf_len;
		Box::pin(async move { f.await.map(|v| AsyncBufStream::with_capacity(buf_len, v)) })
	}
}

#[cfg(feature = "smol")]
pub struct AsyncAcceptor<T: crate::utils::AsyncAcceptor> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	inner:   T,
	buf_len: usize
}

#[cfg(feature = "smol")]
impl<T: crate::utils::AsyncAcceptor> AsyncAcceptor<T> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self::with_capacity(inner, DEFAULT_BUF_LEN)
	}
	
	pub fn with_capacity(inner: T, capacity: usize) -> Self {
		Self { inner, buf_len: capacity }
	}
}

#[cfg(feature = "smol")]
impl<T: crate::utils::AsyncAcceptor> crate::utils::AsyncAcceptor for AsyncAcceptor<T> where T::Connection: futures_lite::AsyncRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncBufStream<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		let Self { inner, buf_len } = unsafe { Pin::into_inner_unchecked(self) };
		let buf_len = *buf_len;
		match unsafe { Pin::new_unchecked(inner).poll_accept(cx) } {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async move { f.await.map(
				|v| AsyncBufStream::with_capacity(buf_len, v)) }))
		}
	}
}