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
	super::Command,
	crate::utils::connection::*,
	std::{io, future::Future, task::{Context, Poll}, pin::Pin}
};

// SYNC

pub type BoxedConnector = Box<dyn Connector<Connection = BoxedClientConnection>>;

impl Connector for BoxedConnector {
	type Connection = BoxedClientConnection;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.as_ref().connect()
	}
}

pub struct DynConnector<T: Connector>(T);

impl<T: Connector> Connector for DynConnector<T>  where T::Connection: 'static + ClientConnection {
	type Connection = BoxedClientConnection;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.0.connect().map(|v| {
			let v: BoxedClientConnection = Box::new(v);
			v
		})
	}
}

pub trait ClientConnection {
	fn write_command(&mut self, command: &Command) -> io::Result<()>;
	
	fn read_response(&mut self) -> io::Result<String>;
}

pub type BoxedClientConnection = Box<dyn ClientConnection>;

impl ClientConnection for BoxedClientConnection {
	fn write_command(&mut self, command: &Command) -> io::Result<()> {
		self.as_mut().write_command(command)
	}
	
	fn read_response(&mut self) -> io::Result<String> {
		self.as_mut().read_response()
	}
}

pub type BoxedAcceptor = Box<dyn Acceptor<Connection = BoxedServerConnection>>;

impl Acceptor for BoxedAcceptor {
	type Connection = BoxedServerConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.as_mut().accept()
	}
}

pub struct DynAcceptor<T: Acceptor>(T);

impl<T: Acceptor> Acceptor for DynAcceptor<T> where T::Connection: 'static + ServerConnection  {
	type Connection = BoxedServerConnection;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.0.accept().map(|v| {
			let v: BoxedServerConnection = Box::new(v);
			v
		})
	}
}

pub trait ServerConnection {
	fn read_command(&mut self) -> io::Result<Command>;
	
	fn write_response(&mut self, msg: &str) -> io::Result<()>;
}

pub type BoxedServerConnection = Box<dyn ServerConnection>;

impl ServerConnection for BoxedServerConnection {
	fn read_command(&mut self) -> io::Result<Command> {
		self.as_mut().read_command()
	}
	
	fn write_response(&mut self, msg: &str) -> io::Result<()> {
		self.as_mut().write_response(msg)
	}
}

// ASYNC

pub type BoxedAsyncConnector = crate::utils::connection::BoxedAsyncConnector<dyn AsyncClientConnection>;

pub struct DynAsyncConnector<T: AsyncConnector>(T);

impl<T: AsyncConnector> DynAsyncConnector<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

impl<T: AsyncConnector> AsyncConnector for DynAsyncConnector<T> where T::Connection: 'static + AsyncClientConnection {
	type Connection = BoxedAsyncClientConnection;
	//type Future     = Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.0.connect();
		Box::pin(async move { f.await.map(|v| { let v: BoxedAsyncClientConnection = Box::pin(v); v }) })
	}
}

pub trait AsyncClientConnection: Send {
	fn poll_write_command(self: Pin<&mut Self>, cx: &mut Context<'_>, command: &Command) -> Poll<io::Result<()>>;
	
	fn poll_read_response(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<String>>;
}

pub trait AsyncClientConnectionExt: AsyncClientConnection {
	fn write_command<'a>(&'a mut self, command: &'a Command) -> AsyncClientConnectionWriteCommand<'a, Self> {
		AsyncClientConnectionWriteCommand(self, command)
	}
	
	fn read_response(&mut self) -> AsyncClientConnectionReadResponse<Self> {
		AsyncClientConnectionReadResponse(self)
	}
}

impl<T: AsyncClientConnection + ?Sized> AsyncClientConnectionExt for T {}

pub struct AsyncClientConnectionWriteCommand<'a, T: AsyncClientConnection + ?Sized>(&'a mut T, &'a Command<'a>);

impl<'a, T: AsyncClientConnection + ?Sized> Future for AsyncClientConnectionWriteCommand<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, cmd) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_command(cx, cmd)
	}
}

pub struct AsyncClientConnectionReadResponse<'a, T: AsyncClientConnection + ?Sized>(&'a mut T);

impl<'a, T: AsyncClientConnection + ?Sized> Future for AsyncClientConnectionReadResponse<'a, T> {
	type Output = io::Result<String>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_read_response(cx)
	}
}

pub type BoxedAsyncClientConnection = Pin<Box<dyn AsyncClientConnection>>;

impl AsyncClientConnection for BoxedAsyncClientConnection {
	fn poll_write_command(mut self: Pin<&mut Self>, cx: &mut Context<'_>, command: &Command) -> Poll<io::Result<()>> {
		self.as_mut().poll_write_command(cx, command)
	}
	
	fn poll_read_response(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<String>> {
		self.as_mut().poll_read_response(cx)
	}
}

pub type BoxedAsyncAcceptor = crate::utils::connection::BoxedAsyncAcceptor<dyn AsyncServerConnection>;

pub struct DynAsyncAcceptor<T: AsyncAcceptor>(T);

impl<T: AsyncAcceptor> AsyncAcceptor for DynAsyncAcceptor<T> where T::Connection: 'static + AsyncServerConnection {
	type Connection = BoxedAsyncServerConnection;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_accept(cx) {
			Poll::Ready(f)  => Poll::Ready(Box::pin(async { f.await.map::<Self::Connection, _>(|v| Box::pin(v)) })),
			Poll::Pending   => Poll::Pending
		}
	}
}

pub trait AsyncServerConnection: Send {
	fn poll_read_command(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Command<'static>>>;
	
	fn poll_write_response(self: Pin<&mut Self>, cx: &mut Context<'_>, msg: &str) -> Poll<io::Result<()>>;
}

pub trait AsyncServerConnectionExt: AsyncServerConnection {
	fn read_command(&mut self) -> AsyncServerConnectionReadCommand<Self> {
		AsyncServerConnectionReadCommand(self)
	}
	
	fn write_response<'a>(&'a mut self, msg: &'a str) -> AsyncServerConnectionWriteResponse<'a, Self> {
		AsyncServerConnectionWriteResponse(self, msg)
	}
}

impl<T: AsyncServerConnection + ?Sized> AsyncServerConnectionExt for T {}

pub struct AsyncServerConnectionReadCommand<'a, T: AsyncServerConnection + ?Sized>(&'a mut T);

impl<'a, T: AsyncServerConnection + ?Sized> Future for AsyncServerConnectionReadCommand<'a, T> {
	type Output = io::Result<Command<'static>>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_read_command(cx)
	}
}

pub struct AsyncServerConnectionWriteResponse<'a, T: AsyncServerConnection + ?Sized>(&'a mut T, &'a str);

impl<'a, T: AsyncServerConnection + ?Sized> Future for AsyncServerConnectionWriteResponse<'a, T> {
	type Output = io::Result<()>;
	
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let Self(inner, msg) = unsafe { Pin::into_inner_unchecked(self) };
		unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_response(cx, msg)
	}
}

pub type BoxedAsyncServerConnection = Pin<Box<dyn AsyncServerConnection>>;

impl AsyncServerConnection for BoxedAsyncServerConnection {
	fn poll_read_command(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Command<'static>>> {
		self.as_mut().poll_read_command(cx)
	}
	
	fn poll_write_response(mut self: Pin<&mut Self>, cx: &mut Context<'_>, msg: &str) -> Poll<io::Result<()>> {
		self.as_mut().poll_write_response(cx, msg)
	}
}