// MIT License
//
// Copyright (c) 2021 Tobias Pfeiffer
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
	super::{traits, Command},
	crate::*,
	std::{io, net, pin::Pin, task::{Poll, Context}, sync::Arc, future::Future}
};

pub fn connector<T: net::ToSocketAddrs>(addr: T) -> Connector<buffered::Connector<tcp::Connector<T>>> {
	Connector::new(buffered::Connector::new(tcp::Connector::new(addr)))
}

pub fn connector_tls<T: net::ToSocketAddrs>(addr: T, name: tls::ServerName, tls: Arc<tls::ClientConfig>) -> Connector<buffered::Connector<tls::Connector<tcp::Connector<T>>>> {
	Connector::new(buffered::Connector::new(tls::Connector::new(tcp::Connector::new(addr), name, tls)))
}

pub fn acceptor<T: net::ToSocketAddrs>(addr: T) -> io::Result<Acceptor<buffered::Acceptor<tcp::Acceptor>>> {
	Ok(Acceptor::new(buffered::Acceptor::new(tcp::Acceptor::new(addr)?)))
}

pub fn acceptor_tls<T: net::ToSocketAddrs>(addr: T, tls: Arc<tls::ServerConfig>) -> io::Result<Acceptor<buffered::Acceptor<tls::Acceptor<tcp::Acceptor>>>> {
	Ok(Acceptor::new(buffered::Acceptor::new(tls::Acceptor::new(tcp::Acceptor::new(addr)?, tls))))
}

pub struct Connector<T: utils::Connector> where T::Connection: io::BufRead + io::Write {
	inner: T
}

impl<T: utils::Connector> Connector<T> where T::Connection: io::BufRead + io::Write {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::Connector> utils::Connector for Connector<T> where T::Connection: io::BufRead + io::Write {
	type Connection = ClientConnection<T::Connection>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.inner.connect().map(ClientConnection::new)
	}
}

pub struct ClientConnection<T: io::BufRead + io::Write> {
	inner: T
}

impl<T: io::BufRead + io::Write> ClientConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: io::BufRead + io::Write> traits::ClientConnection for ClientConnection<T> {
	fn write_command(&mut self, command: &Command) -> io::Result<()> {
		todo!()
	}
	
	fn read_response(&mut self) -> io::Result<String> {
		todo!()
	}
}

pub struct Acceptor<T: utils::Acceptor> where T::Connection: io::BufRead + io::Write {
	inner: T
}

impl<T: utils::Acceptor> Acceptor<T> where T::Connection: io::BufRead + io::Write {
	pub fn new(inner: T) -> Self {
		Acceptor { inner }
	}
}

impl<T: utils::Acceptor> utils::Acceptor for Acceptor<T> where T::Connection: io::BufRead + io::Write {
	type Connection = ServerConnection<T::Connection>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.inner.accept().map(ServerConnection::new)
	}
}

pub struct ServerConnection<T: io::BufRead + io::Write> {
	inner: T
}

impl<T: io::BufRead + io::Write> ServerConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: io::BufRead + io::Write> traits::ServerConnection for ServerConnection<T> {
	fn read_command(&mut self) -> io::Result<Command> {
		todo!()
	}
	
	fn write_response(&mut self, msg: &str) -> io::Result<()> {
		todo!()
	}
}

#[cfg(feature = "smol")]
pub fn connector_async<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(addr: T) -> AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
	AsyncConnector::new(buffered::AsyncConnector::new(tcp::AsyncConnector::new(addr)))
}

#[cfg(all(feature = "smol"))]
pub fn connector_tls_async<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(addr: T, host: tls::r#async::webpki::DNSName, tls: Arc<tls::r#async::rustls::ClientConfig>) -> AsyncConnector<buffered::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>>> {
	AsyncConnector::new(buffered::AsyncConnector::new(tls::AsyncConnector::new(tcp::AsyncConnector::new(addr), host, tls)))
}

#[cfg(feature = "smol")]
pub async fn acceptor_async<T: smol::net::AsyncToSocketAddrs>(addr: T) -> io::Result<AsyncAcceptor<buffered::AsyncAcceptor<tcp::AsyncAcceptor>>> {
	Ok(AsyncAcceptor::new(buffered::AsyncAcceptor::new(tcp::AsyncAcceptor::new(addr).await?)))
}

#[cfg(feature = "smol")]
pub async fn acceptor_tls_async<T: smol::net::AsyncToSocketAddrs>(addr: T, tls: Arc<tls::r#async::rustls::ServerConfig>) -> io::Result<AsyncAcceptor<buffered::AsyncAcceptor<tls::AsyncAcceptor<tcp::AsyncAcceptor>>>> {
	Ok(AsyncAcceptor::new(buffered::AsyncAcceptor::new(tls::AsyncAcceptor::new(tcp::AsyncAcceptor::new(addr).await?, tls))))
}

pub struct AsyncConnector<T: utils::AsyncConnector> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	inner: T
}

impl<T: utils::AsyncConnector> AsyncConnector<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncConnector> utils::AsyncConnector for AsyncConnector<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncClientConnection<T::Connection>;
	//type Future     = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		Box::pin(async move { f.await.map(AsyncClientConnection::new) })
	}
}

pub struct AsyncClientConnection<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> {
	inner: T
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> AsyncClientConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite + Send> traits::AsyncClientConnection for AsyncClientConnection<T> {
	fn poll_write_command(self: Pin<&mut Self>, cx: &mut Context<'_>, command: &Command) -> Poll<io::Result<()>> {
		todo!()
	}
	
	fn poll_read_response(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<String>> {
		todo!()
	}
}

pub struct AsyncAcceptor<T: utils::AsyncAcceptor> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	inner: T
}

impl<T: utils::AsyncAcceptor> AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncAcceptor> utils::AsyncAcceptor for AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	type Connection = AsyncServerConnection<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_accept(cx) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async { f.await.map(AsyncServerConnection::new) }))
		}
	}
}

pub struct AsyncServerConnection<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> {
	inner: T
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> AsyncServerConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite + Send> traits::AsyncServerConnection for AsyncServerConnection<T> {
	fn poll_read_command(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Command<'static>>> {
		todo!()
	}
	
	fn poll_write_response(self: Pin<&mut Self>, cx: &mut Context<'_>, msg: &str) -> Poll<io::Result<()>> {
		todo!()
	}
}