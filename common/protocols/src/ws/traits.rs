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
	crate::utils::connection::*,
	std::{io, future::Future, pin::Pin, task::{Context, Poll}},
};

// SYNC

pub type BoxedConnector = Box<dyn Connector<Connection = BoxedConnection>>;

impl Connector for BoxedConnector {
	type Connection = BoxedConnection;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.as_ref().connect()
	}
}

pub struct DynConnector<T: Connector>(T);

impl<T: Connector> Connector for DynConnector<T> where T::Connection: 'static + Connection  {
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

impl<T: Acceptor> Acceptor for DynAcceptor<T> where T::Connection: 'static + Connection  {
	type Connection = BoxedConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.0.accept().map(|v| {
			let v: BoxedConnection = Box::new(v);
			v
		})
	}
}

pub trait Connection {
	fn read(&mut self, opcode: &mut u32, buf: &mut [u8]) -> io::Result<u64>;
	
	fn write(&mut self, opcode: u32, buf: &[u8]) -> io::Result<u64>;
}

pub type BoxedConnection = Box<dyn Connection>;

impl Connection for BoxedConnection {
	fn read(&mut self, opcode: &mut u32, buf: &mut [u8]) -> io::Result<u64> {
		self.as_mut().read(opcode, buf)
	}
	
	fn write(&mut self, opcode: u32, buf: &[u8]) -> io::Result<u64> {
		self.as_mut().write(opcode, buf)
	}
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

pub trait AsyncConnection: Send {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, opcode: &mut u32, buf: &mut [u8]) -> Poll<io::Result<u64>>;
	
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, opcode: u32, buf: &[u8]) -> Poll<io::Result<u64>>;
}

pub trait AsyncConnectionExt: AsyncConnection {
	fn read<'a>(&'a mut self, opcode: &'a mut u32, buf: &'a mut [u8]) -> AsyncConnectionRead<'a, Self> {
		AsyncConnectionRead(self, opcode, buf)
	}
	
	fn write<'a>(&'a mut self, opcode: u32, buf: &'a [u8]) -> AsyncConnectionWrite<'a, Self> {
		AsyncConnectionWrite(self, opcode, buf)
	}
}

impl<T: AsyncConnection + ?Sized> AsyncConnectionExt for T {}

pub struct AsyncConnectionRead<'a, T: AsyncConnection + ?Sized>(&'a mut T, &'a mut u32, &'a mut [u8]);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionRead<'a, T> {
	type Output = io::Result<u64>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, opcode, buf) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_read(cx, opcode, buf)
	}
}

pub struct AsyncConnectionWrite<'a, T: AsyncConnection + ?Sized>(&'a mut T, u32, &'a [u8]);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionWrite<'a, T> {
	type Output = io::Result<u64>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, opcode, buf) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write(cx, *opcode, buf)
	}
}

pub type BoxedAsyncConnection = Pin<Box<dyn AsyncConnection>>;

impl AsyncConnection for BoxedAsyncConnection {
	fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, opcode: &mut u32, buf: &mut [u8]) -> Poll<io::Result<u64>> {
		self.as_mut().poll_read(cx, opcode, buf)
	}
	
	fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, opcode: u32, buf: &[u8]) -> Poll<io::Result<u64>> {
		self.as_mut().poll_write(cx, opcode, buf)
	}
}