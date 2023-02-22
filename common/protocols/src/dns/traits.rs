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
	super::Message,
	crate::utils::connection::*,
	std::{io, future::Future, pin::Pin, task::{Context, Poll}}
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

impl<T: Acceptor> Acceptor for DynAcceptor<T> where T::Connection: 'static + Connection {
	type Connection = BoxedConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.0.accept().map(|v| {
			let v: BoxedConnection = Box::new(v);
			v
		})
	}
}

pub trait Connection {
	fn send_msg(&mut self, message: &Message) -> io::Result<()>;
	
	fn recv_msg(&mut self) -> io::Result<Message>;
}

pub type BoxedConnection = Box<dyn Connection>;

impl Connection for BoxedConnection {
	fn send_msg(&mut self, message: &Message) -> io::Result<()> {
		self.as_mut().send_msg(message)
	}
	
	fn recv_msg(&mut self) -> io::Result<Message> {
		self.as_mut().recv_msg()
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
	fn poll_send_msg(self: Pin<&mut Self>, cx: &mut Context<'_>, message: &Message) -> Poll<io::Result<()>>;
	
	fn poll_recv_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Message>>;
}

pub trait AsyncConnectionExt: AsyncConnection {
	fn send_msg<'a>(&'a mut self, message: &'a Message) -> AsyncConnectionSend<'a, Self> {
		AsyncConnectionSend(self, message)
	}
	
	fn recv_msg(&mut self) -> AsyncConnectionRecv<Self> {
		AsyncConnectionRecv(self)
	}
}

impl<T: AsyncConnection> AsyncConnectionExt for T {}

pub struct AsyncConnectionSend<'a, T: AsyncConnection + ?Sized>(&'a mut T, &'a Message);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionSend<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, msg) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_send_msg(cx, msg)
	}
}

pub struct AsyncConnectionRecv<'a, T: AsyncConnection + ?Sized>(&'a mut T);

impl<'a, T: AsyncConnection + ?Sized> Future for AsyncConnectionRecv<'a, T> {
	type Output = io::Result<Message>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_recv_msg(cx)
	}
}

pub type BoxedAsyncConnection = Pin<Box<dyn AsyncConnection>>;

impl AsyncConnection for BoxedAsyncConnection {
	fn poll_send_msg(mut self: Pin<&mut Self>, cx: &mut Context<'_>, message: &Message) -> Poll<io::Result<()>> {
		self.as_mut().poll_send_msg(cx, message)
	}
	
	fn poll_recv_msg(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Message>> {
		self.as_mut().poll_recv_msg(cx)
	}
}