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
	super::{traits, Message},
	crate::*,
	std::{io, net, pin::Pin, task::{Poll, Context}, sync::Arc, future::Future}
};

pub fn connector<T: net::ToSocketAddrs>(addr: T) -> Connector<tcp::Connector<T>> {
	Connector::new(tcp::Connector::new(addr))
}

pub fn connector_tls<T: net::ToSocketAddrs>(addr: T, name: tls::ServerName, tls: Arc<tls::ClientConfig>) -> Connector<tls::Connector<tcp::Connector<T>>> {
	Connector::new(tls::Connector::new(tcp::Connector::new(addr), name, tls))
}

pub fn acceptor<T: net::ToSocketAddrs>(addr: T) -> io::Result<Acceptor<tcp::Acceptor>> {
	Ok(Acceptor::new(tcp::Acceptor::new(addr)?))
}

pub fn acceptor_tls<T: net::ToSocketAddrs>(addr: T, tls: Arc<tls::ServerConfig>) -> io::Result<Acceptor<tls::Acceptor<tcp::Acceptor>>> {
	Ok(Acceptor::new(tls::Acceptor::new(tcp::Acceptor::new(addr)?, tls)))
}

pub struct Connector<T: utils::Connector> where T::Connection: io::Read + io::Write {
	inner: T
}

impl<T: utils::Connector> Connector<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::Connector> utils::Connector for Connector<T> where T::Connection: io::Read + io::Write {
	type Connection = Connection<T::Connection>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.inner.connect().map(Connection::new)
	}
}

pub struct Acceptor<T: utils::Acceptor> where T::Connection: io::Read + io::Write {
	inner: T
}

impl<T: utils::Acceptor> Acceptor<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T) -> Self {
		Acceptor { inner }
	}
}

impl<T: utils::Acceptor> utils::Acceptor for Acceptor<T> where T::Connection: io::Read + io::Write {
	type Connection = Connection<T::Connection>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.inner.accept().map(Connection::new)
	}
}

pub struct Connection<T: io::Read + io::Write> {
	inner: T
}

impl<T: io::Read + io::Write> Connection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: io::Read + io::Write> traits::Connection for Connection<T> {
	fn send_msg(&mut self, message: &Message) -> io::Result<()> {
		todo!()
	}
	
	fn recv_msg(&mut self) -> io::Result<Message> {
		todo!()
	}
}

#[cfg(feature = "smol")]
pub fn connector_async<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(addr: T) -> AsyncConnector<tcp::AsyncConnector<T>> {
	AsyncConnector::new(tcp::AsyncConnector::new(addr))
}

#[cfg(feature = "smol")]
pub fn connector_tls_async<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(addr: T, host: tls::r#async::webpki::DNSName, tls: Arc<tls::r#async::rustls::ClientConfig>) -> AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>> {
	AsyncConnector::new(tls::AsyncConnector::new(tcp::AsyncConnector::new(addr), host, tls))
}

#[cfg(feature = "smol")]
pub async fn acceptor_async<T: smol::net::AsyncToSocketAddrs>(addr: T) -> io::Result<AsyncAcceptor<tcp::AsyncAcceptor>> {
	Ok(AsyncAcceptor::new(tcp::AsyncAcceptor::new(addr).await?))
}

#[cfg(feature = "smol")]
pub async fn acceptor_tls_async<T: smol::net::AsyncToSocketAddrs>(addr: T, tls: Arc<tls::r#async::rustls::ServerConfig>) -> io::Result<AsyncAcceptor<tls::AsyncAcceptor<tcp::AsyncAcceptor>>> {
	Ok(AsyncAcceptor::new(tls::AsyncAcceptor::new(tcp::AsyncAcceptor::new(addr).await?, tls)))
}

pub struct AsyncConnector<T: utils::AsyncConnector> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	inner: T
}

impl<T: utils::AsyncConnector> AsyncConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncConnector> utils::AsyncConnector for AsyncConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncConnection<T::Connection>;
	//type Future     = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		Box::pin(async move { f.await.map(AsyncConnection::new) })
	}
}

pub struct AsyncAcceptor<T: utils::AsyncAcceptor> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	inner: T
}

impl<T: utils::AsyncAcceptor> AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncAcceptor> utils::AsyncAcceptor for AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncConnection<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_accept(cx) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async { f.await.map(AsyncConnection::new) }))
		}
	}
}

pub struct AsyncConnection<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> {
	inner: T
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> AsyncConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Send> traits::AsyncConnection for AsyncConnection<T> {
	fn poll_send_msg(self: Pin<&mut Self>, cx: &mut Context<'_>, message: &Message) -> Poll<io::Result<()>> {
		todo!()
	}
	
	fn poll_recv_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Message>> {
		todo!()
	}
}