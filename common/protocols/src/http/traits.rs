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

use {
	super::Header,
	crate::utils::connection::*,
	std::{io, future::Future, pin::Pin, task::{Context, Poll}}
};

pub type StreamId = usize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Version {
	V1 = 1,
	V2 = 2,
	V3 = 3
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConnectionInfo {
	/// the HTTP version of the connection
	pub version:      Version,
	/// If true, streams can be written to out of order
	pub out_of_order: bool,
	/// If true, multiple streams can be open at the same time
	pub parallel:     bool
}

#[derive(Clone, Debug)]
pub enum Read<'a> {
	Opened,
	Headers(Vec<Header>),
	HeadersDone,
	Body(&'a [u8]),
	Closed
}

// SYNC

pub type BoxedConnector = Box<dyn Connector<Connection = BoxedConnection>>;

impl Connector for BoxedConnector {
	type Connection = BoxedConnection;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.as_ref().connect()
	}
}

pub struct DynConnector<T: Connector>(T);

impl<T: Connector> Connector for DynConnector<T>  where T::Connection: 'static + Connection {
	type Connection = BoxedConnection;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.0.connect().map(|v| {
			let v: BoxedConnection = Box::new(v);
			v
		})
	}
}

pub type BoxedAcceptor = Box<dyn Acceptor<Connection = BoxedConnection>>;

impl Acceptor for BoxedAcceptor {
	type Connection = BoxedConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.as_mut().accept()
	}
}

pub struct DynAcceptor<T: Acceptor>(T);

impl<T: Acceptor> Acceptor for DynAcceptor<T>  where T::Connection: 'static + Connection  {
	type Connection = BoxedConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.0.accept().map(|v| {
			let v: BoxedConnection = Box::new(v);
			v
		})
	}
}

pub trait Connection {
	fn open(&mut self) -> io::Result<StreamId>;
	
	fn read<'a>(&mut self, buf: &'a mut [u8]) -> io::Result<Option<(StreamId, Read<'a>)>>;
	
	fn read_vectored<'a>(&mut self, _buf: &'a [&'a [u8]]) -> io::Result<Option<(StreamId, Read<'a>)>> {
		Err(io::Error::from(io::ErrorKind::Unsupported))
	}
	
	fn write_headers(&mut self, id: StreamId, headers: &[Header]) -> io::Result<()>;
	
	fn write_body(&mut self, id: StreamId, buf: &[u8]) -> io::Result<()>;
	
	fn close(&mut self, id: StreamId) -> io::Result<()>;
}

pub type BoxedConnection = Box<dyn Connection>;

impl Connection for BoxedConnection {
	fn open(&mut self) -> io::Result<StreamId> {
		self.as_mut().open()
	}
	
	fn read<'a>(&mut self, buf: &'a mut [u8]) -> io::Result<Option<(StreamId, Read<'a>)>> {
		self.as_mut().read(buf)
	}
	
	fn read_vectored<'a>(&mut self, buf: &'a [&'a [u8]]) -> io::Result<Option<(StreamId, Read<'a>)>> {
		self.as_mut().read_vectored(buf)
	}
	
	fn write_headers(&mut self, id: StreamId, headers: &[Header]) -> io::Result<()> {
		self.as_mut().write_headers(id, headers)
	}
	
	fn write_body(&mut self, id: StreamId, buf: &[u8]) -> io::Result<()> {
		self.as_mut().write_body(id, buf)
	}
	
	fn close(&mut self, id: StreamId) -> io::Result<()> {
		self.as_mut().close(id)
	}
}

pub trait SharedConnection {
	fn open(&self) -> io::Result<StreamId>;
	
	fn opened(&self) -> io::Result<StreamId>;
	
	fn read<'a>(&self, id: StreamId, buf: &'a mut [u8]) -> io::Result<Read<'a>>;
	
	fn read_vectored<'a>(&self, id: StreamId, buf: &'a [&'a [u8]]) -> io::Result<Read<'a>>;
	
	fn write_headers(&self, id: StreamId, headers: &[Header]) -> io::Result<()>;
	
	fn write_body(&self, id: StreamId, buf: &[u8]) -> io::Result<()>;
	
	fn close(&self, id: StreamId) -> io::Result<()>;
}

pub type BoxedSharedConnection = Box<dyn SharedConnection>;

impl SharedConnection for BoxedSharedConnection {
	fn open(&self) -> io::Result<StreamId> {
		self.as_ref().open()
	}
	
	fn opened(&self) -> io::Result<StreamId> {
		self.as_ref().opened()
	}
	
	fn read<'a>(&self, id: StreamId, buf: &'a mut [u8]) -> io::Result<Read<'a>> {
		self.as_ref().read(id, buf)
	}
	
	fn read_vectored<'a>(&self, id: StreamId, buf: &'a [&'a [u8]]) -> io::Result<Read<'a>> {
		self.as_ref().read_vectored(id, buf)
	}
	
	fn write_headers(&self, id: StreamId, headers: &[Header]) -> io::Result<()> {
		self.as_ref().write_headers(id, headers)
	}
	
	fn write_body(&self, id: StreamId, buf: &[u8]) -> io::Result<()> {
		self.as_ref().write_body(id, buf)
	}
	
	fn close(&self, id: StreamId) -> io::Result<()> {
		self.as_ref().close(id)
	}
}

pub trait Stream: io::Read + io::Write {
	fn write_headers(&mut self, headers: &[Header]) -> io::Result<()>;
	
	fn read_headers(&mut self) -> io::Result<Vec<Header>>;
	
	fn close(&mut self) -> io::Result<()>;
}

// ASYNC

pub type BoxedAsyncConnector = crate::utils::connection::BoxedAsyncConnector<dyn AsyncConnection>;

pub struct DynAsyncConnector<T: AsyncConnector>(T);

impl<T: AsyncConnector> DynAsyncConnector<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

impl<T: AsyncConnector> AsyncConnector for DynAsyncConnector<T> where T::Connection: 'static + AsyncConnection {
	type Connection = BoxedAsyncConnection;
	//type Future     = Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.0.connect();
		Box::pin(async move { f.await.map(|v| { let v: BoxedAsyncConnection = Box::pin(v); v }) })
	}
}

pub type BoxedAsyncAcceptor = crate::utils::connection::BoxedAsyncAcceptor<dyn AsyncConnection>;

pub struct DynAsyncAcceptor<T: AsyncAcceptor>(T);

impl<T: AsyncAcceptor> AsyncAcceptor for DynAsyncAcceptor<T> where T::Connection: 'static + AsyncConnection {
	type Connection = BoxedAsyncConnection;
	
		fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_accept(cx) {
			Poll::Ready(f)  => Poll::Ready(Box::pin(async { f.await.map::<Self::Connection, _>(|v| Box::pin(v)) })),
			Poll::Pending   => Poll::Pending
		}
	}
}

pub trait AsyncConnection: Send + 'static {
	fn poll_open(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<StreamId>>;
	
	fn poll_read<'a>(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &'a mut [u8]) -> Poll<io::Result<Option<(StreamId, Read<'a>)>>>;
	
	fn poll_read_vectored<'a>(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &'a mut [&'a mut [u8]]) -> Poll<io::Result<Option<(StreamId, Read<'a>)>>> {
		Poll::Ready(Err(io::Error::from(io::ErrorKind::Unsupported)))
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId, headers: &[Header]) -> Poll<io::Result<()>>;
	
	fn poll_write_body(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId, buf: &[u8]) -> Poll<io::Result<()>>;
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>>;
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>>;
}

pub trait AsyncConnectionExt: AsyncConnection {
	fn open(&mut self) -> AsyncConnectionOpen<Self> {
		AsyncConnectionOpen(self)
	}
	
	fn read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> AsyncConnectionRead<'a, 'b, Self> {
		AsyncConnectionRead(self, buf)
	}
	
	fn incoming<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> AsyncConnectionIncoming<'a, 'b, Self> {
		AsyncConnectionIncoming(self, buf)
	}
	
	fn write_headers<'a>(&'a mut self, id: StreamId, headers: &'a [Header]) -> AsyncConnectionWriteHeaders<'a, Self> {
		AsyncConnectionWriteHeaders(self, id, headers)
	}
	
	fn write_body<'a>(&'a mut self, id: StreamId, buf: &'a [u8]) -> AsyncConnectionWriteBody<'a, Self> {
		AsyncConnectionWriteBody(self, id, buf)
	}
	
	fn flush(&mut self, id: StreamId) -> AsyncConnectionFlush<Self> {
		AsyncConnectionFlush(self, id)
	}
	
	fn close(&mut self, id: StreamId) -> AsyncConnectionClose<Self> {
		AsyncConnectionClose(self, id)
	}
}

impl<T: AsyncConnection + ?Sized> AsyncConnectionExt for T {}

pub struct AsyncConnectionOpen<'a, T: AsyncConnection + ?Sized>(&'a mut T);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionOpen<'a, T> {
	type Output = io::Result<StreamId>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut**v) }.poll_open(cx)
	}
}

pub struct AsyncConnectionRead<'a, 'b, T: AsyncConnection + ?Sized>(&'a mut T, &'b mut [u8]);

impl<'a, 'b, T: AsyncConnection + ?Sized> Future for AsyncConnectionRead<'a, 'b, T> {
	type Output = io::Result<Option<(StreamId, Read<'b>)>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, buf) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut **inner) }
			.poll_read(cx, unsafe { std::mem::transmute(&mut**buf) })
	}
}

pub struct AsyncConnectionIncoming<'a, 'b, T: AsyncConnection + ?Sized>(&'a mut T, &'b mut [u8]);

impl<'a, 'b, T: AsyncConnection + ?Sized> futures_lite::Stream for AsyncConnectionIncoming<'a, 'b, T> {
	type Item = io::Result<(StreamId, Read<'b>)>;
	
	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let Self(inner, buf) = unsafe { Pin::into_inner_unchecked(self) };
		match unsafe { Pin::new_unchecked(&mut **inner) }
			.poll_read(cx, unsafe { std::mem::transmute(&mut**buf) }) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(v) => Poll::Ready(v.transpose())
		}
	}
}

pub struct AsyncConnectionWriteHeaders<'a, T: AsyncConnection + ?Sized>(&'a mut T, StreamId, &'a [Header]);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionWriteHeaders<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, headers) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_headers(cx, *id, headers)
	}
}

pub struct AsyncConnectionWriteBody<'a, T: AsyncConnection + ?Sized>(&'a mut T, StreamId, &'a [u8]);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionWriteBody<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, buf) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_body(cx, *id, buf)
	}
}

pub struct AsyncConnectionFlush<'a, T: AsyncConnection + ?Sized>(&'a mut T, StreamId);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionFlush<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_flush(cx, *id)
	}
}

pub struct AsyncConnectionClose<'a, T: AsyncConnection + ?Sized>(&'a mut T, StreamId);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionClose<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_close(cx, *id)
	}
}

pub type BoxedAsyncConnection = Pin<Box<dyn AsyncConnection>>;

impl AsyncConnection for BoxedAsyncConnection {
	fn poll_open(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<StreamId>> {
		self.as_mut().poll_open(cx)
	}
	
	fn poll_read<'a>(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &'a mut [u8]) -> Poll<io::Result<Option<(usize, Read<'a>)>>> {
		self.as_mut().poll_read(cx, buf)
	}
	
	fn poll_write_headers(mut self: Pin<&mut Self>, cx: &mut Context<'_>, id: usize, headers: &[Header]) -> Poll<io::Result<()>> {
		self.as_mut().poll_write_headers(cx, id, headers)
	}
	
	fn poll_write_body(mut self: Pin<&mut Self>, cx: &mut Context<'_>, id: usize, buf: &[u8]) -> Poll<io::Result<()>> {
		self.as_mut().poll_write_body(cx, id, buf)
	}
	
	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>> {
		self.as_mut().poll_flush(cx, id)
	}
	
	fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>> {
		self.as_mut().poll_close(cx, id)
	}
}

// ASYNC SHARED

pub type BoxedAsyncSharedConnector = crate::utils::connection::BoxedAsyncConnector<dyn AsyncConnection>;

pub struct DynAsyncSharedConnector<T: AsyncConnector>(T);

impl<T: AsyncConnector> DynAsyncSharedConnector<T> {
	pub fn new(v: T) -> Self {
		Self(v)
	}
}

impl<T: AsyncConnector> AsyncConnector for DynAsyncSharedConnector<T> where T::Connection: 'static + AsyncSharedConnection {
	type Connection = BoxedAsyncSharedConnection;
	//type Future     = Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.0.connect();
		Box::pin(async move { f.await.map(|v| { let v: BoxedAsyncSharedConnection = Box::pin(v); v }) })
	}
}

pub type BoxedAsyncSharedAcceptor = crate::utils::connection::BoxedAsyncAcceptor<dyn AsyncSharedConnection>;

pub struct DynAsyncSharedAcceptor<T: AsyncAcceptor>(T);

impl<T: AsyncAcceptor> AsyncAcceptor for DynAsyncSharedAcceptor<T> where T::Connection: 'static + AsyncSharedConnection {
	type Connection = BoxedAsyncSharedConnection;
	
		fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_accept(cx) {
			Poll::Ready(f)  => Poll::Ready(Box::pin(async { f.await.map::<Self::Connection, _>(|v| Box::pin(v)) })),
			Poll::Pending   => Poll::Pending
		}
	}
}

pub trait AsyncSharedConnection: Send + Sync {
	fn poll_open(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<io::Result<StreamId>>;
	
	fn poll_opened(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<io::Result<Option<StreamId>>>;
	
	fn poll_read<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [u8]) -> Poll<io::Result<Read<'a>>>;
	
	fn poll_read_vectored<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [&'a mut [u8]]) -> Poll<io::Result<Read<'a>>>;
	
	fn poll_write_headers(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, headers: &[Header]) -> Poll<io::Result<()>>;
	
	fn poll_write_body(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &[u8]) -> Poll<io::Result<()>>;
	
	fn poll_flush(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>>;
	
	fn poll_close(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>>;
}

pub trait AsyncSharedConnectionExt: AsyncSharedConnection {
	fn open(&self) -> AsyncSharedConnectionOpen<Self> {
		AsyncSharedConnectionOpen(self)
	}
	
	fn opened(&self) -> AsyncSharedConnectionOpened<Self> {
		AsyncSharedConnectionOpened(self)
	}
	
	fn read<'a>(&'a self, id: StreamId, buf: &'a mut [u8]) -> AsyncSharedConnectionRead<'a, Self> {
		AsyncSharedConnectionRead(self, id, buf)
	}
	
	fn read_vectored<'a>(&'a self, id: StreamId, buf: &'a mut [&'a mut [u8]]) -> AsyncSharedConnectionReadVectored<'a, Self> {
		AsyncSharedConnectionReadVectored(self, id, buf)
	}
	
	fn write_headers<'a>(&'a self, id: StreamId, headers: &'a [Header]) -> AsyncSharedConnectionWriteHeaders<'a, Self> {
		AsyncSharedConnectionWriteHeaders(self, id, headers)
	}
	
	fn write_body<'a>(&'a self, id: StreamId, buf: &'a [u8]) -> AsyncSharedConnectionWriteBody<'a, Self> {
		AsyncSharedConnectionWriteBody(self, id, buf)
	}
	
	fn flush(&self, id: StreamId) -> AsyncSharedConnectionFlush<Self> {
		AsyncSharedConnectionFlush(self, id)
	}
	
	fn close(&self, id: StreamId) -> AsyncSharedConnectionClose<Self> {
		AsyncSharedConnectionClose(self, id)
	}
}

impl<T: AsyncSharedConnection + ?Sized> AsyncSharedConnectionExt for T {}

pub struct AsyncSharedConnectionOpen<'a, T: AsyncSharedConnection + ?Sized>(&'a T);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionOpen<'a, T> {
	type Output = io::Result<StreamId>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_open(cx)
	}
}

pub struct AsyncSharedConnectionOpened<'a, T: AsyncSharedConnection + ?Sized>(&'a T);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionOpened<'a, T> {
	type Output = io::Result<Option<StreamId>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_opened(cx)
	}
}

pub struct AsyncSharedConnectionRead<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId, &'a mut [u8]);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionRead<'a, T> {
	type Output = io::Result<Read<'a>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, buf) = unsafe { Pin::into_inner_unchecked(std::mem::transmute::<_, Pin<&mut AsyncSharedConnectionRead<T>>>(self)) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_read(cx, *id, buf)
	}
}

pub struct AsyncSharedConnectionReadVectored<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId, &'a mut [&'a mut [u8]]);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionReadVectored<'a, T> {
	type Output = io::Result<Read<'a>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, buf) = unsafe { Pin::into_inner_unchecked(std::mem::transmute::<_, Pin<&mut AsyncSharedConnectionReadVectored<T>>>(self)) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_read_vectored(cx, *id, buf)
	}
}

pub struct AsyncSharedConnectionWriteHeaders<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId, &'a [Header]);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionWriteHeaders<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, headers) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_write_headers(cx, *id, headers)
	}
}

pub struct AsyncSharedConnectionWriteBody<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId, &'a [u8]);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionWriteBody<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id, buf) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_write_body(cx, *id, buf)
	}
}

pub struct AsyncSharedConnectionFlush<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionFlush<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_flush(cx, *id)
	}
}

pub struct AsyncSharedConnectionClose<'a, T: AsyncSharedConnection + ?Sized>(&'a T, StreamId);

impl<'a, T: AsyncSharedConnection + ?Sized> Future for AsyncSharedConnectionClose<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, id) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&**inner) }.poll_close(cx, *id)
	}
}

pub type BoxedAsyncSharedConnection = Pin<Box<dyn AsyncSharedConnection>>;

impl AsyncSharedConnection for BoxedAsyncSharedConnection {
	fn poll_open(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<io::Result<StreamId>> {
		self.as_ref().poll_open(cx)
	}
	
	fn poll_opened(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<io::Result<Option<StreamId>>> {
		self.as_ref().poll_opened(cx)
	}
	
	fn poll_read<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [u8]) -> Poll<io::Result<Read<'a>>> {
		self.as_ref().poll_read(cx, id, buf)
	}
	
	fn poll_read_vectored<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [&'a mut [u8]]) -> Poll<io::Result<Read<'a>>> {
		self.as_ref().poll_read_vectored(cx, id, buf)
	}
	
	fn poll_write_headers(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, headers: &[Header]) -> Poll<io::Result<()>> {
		self.as_ref().poll_write_headers(cx, id, headers)
	}
	
	fn poll_write_body(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &[u8]) -> Poll<io::Result<()>> {
		self.as_ref().poll_write_body(cx, id, buf)
	}
	
	fn poll_flush(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>> {
		self.as_ref().poll_flush(cx, id)
	}
	
	fn poll_close(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<io::Result<()>> {
		self.as_ref().poll_close(cx, id)
	}
}

// STREAM

pub trait AsyncStream: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Send + Unpin {
	fn poll_read_headers(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Vec<Header>>>;
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[Header]) -> Poll<io::Result<()>>;
}

pub trait AsyncStreamExt: AsyncStream {
	fn read_headers(&mut self) -> AsyncStreamReadHeaders<Self> {
		AsyncStreamReadHeaders(self)
	}
	
	fn write_headers<'a>(&'a mut self, headers: &'a [Header]) -> AsyncStreamWriteHeaders<'a, Self> {
		AsyncStreamWriteHeaders(self, headers)
	}
}

impl<T: AsyncStream + ?Sized> AsyncStreamExt for T {}

pub struct AsyncStreamReadHeaders<'a, T: AsyncStream + ?Sized>(&'a mut T);

impl<'a, T: AsyncStream + ?Sized> Future for AsyncStreamReadHeaders<'a, T> {
	type Output = io::Result<Vec<Header>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_read_headers(cx)
	}
}

pub struct AsyncStreamWriteHeaders<'a, T: AsyncStream + ?Sized>(&'a mut T, &'a [Header]);

impl<'a, T: AsyncStream + ?Sized> Future for AsyncStreamWriteHeaders<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, headers) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_headers(cx, headers)
	}
}

pub type BoxedAsyncStream = Pin<Box<dyn AsyncStream>>;

impl AsyncStream for BoxedAsyncStream {
	fn poll_read_headers<'a>(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Vec<Header>>> {
		self.as_mut().poll_read_headers(cx)
	}
	
	fn poll_write_headers(mut self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[Header]) -> Poll<io::Result<()>> {
		self.as_mut().poll_write_headers(cx, headers)
	}
}