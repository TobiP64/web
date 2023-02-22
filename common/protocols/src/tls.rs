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

use {crate::*, std::{io, sync::Arc, pin::Pin, task::{Context, Poll}, future::Future}};

pub use rustls::*;
pub use async_rustls as r#async;

pub const DEFAULT_PORT: u16 = 443;

pub mod alpn {
	// https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml#alpn-protocol-ids
	pub const HTTP09:         &[u8] = b"http/0.9";
	pub const HTTP10:         &[u8] = b"http/1.0";
	pub const HTTP11:         &[u8] = b"http/1.1";
	pub const TURN:           &[u8] = b"stun.turn";
	pub const STUN:           &[u8] = b"stun.nat-discovery";
	pub const HTTP2_OVER_TLS: &[u8] = b"h2";
	pub const HTTP2_OVER_TCP: &[u8] = b"h2c";
	pub const HTTP3:          &[u8] = b"h3";
	pub const WEB_RTC:        &[u8] = b"webrtc";
	pub const IMAP:           &[u8] = b"imap";
	pub const POP3:           &[u8] = b"pop3";
	pub const DOT:            &[u8] = b"dot";
}

pub struct Connector<T: crate::utils::Connector> where T::Connection: io::Read + io::Write {
	inner: T,
	name:  ServerName,
	cfg:   Arc<ClientConfig>
}

impl<T: crate::utils::Connector> Connector<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T, name: ServerName, cfg: Arc<ClientConfig>) -> Self {
		Self { inner, name, cfg }
	}
}

impl<T: crate::utils::Connector> crate::utils::Connector for Connector<T> where T::Connection: io::Read + io::Write {
	type Connection = StreamOwned<ClientConnection, T::Connection>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		let stream = self.inner.connect()?;
		let conn = ClientConnection::new(self.cfg.clone(), self.name.clone())
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(StreamOwned::new(conn, stream))
	}
}

pub struct Acceptor<T: crate::utils::Acceptor> where T::Connection: io::Read + io::Write {
	inner: T,
	cfg:   Arc<ServerConfig>
}

impl<T: crate::utils::Acceptor> Acceptor<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: T, cfg: Arc<ServerConfig>) -> Self {
		Self { inner, cfg }
	}
}

impl<T: crate::utils::Acceptor> crate::utils::Acceptor for Acceptor<T> where T::Connection: io::Read + io::Write {
	type Connection = StreamOwned<ServerConnection, T::Connection>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		let stream = self.inner.accept()?;
		let conn = ServerConnection::new(self.cfg.clone())
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		Ok(StreamOwned::new(conn, stream))
	}
}

pub struct AsyncConnector<T: crate::utils::AsyncConnector> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	inner: T,
	host:  r#async::webpki::DNSName,
	cfg:   Arc<r#async::rustls::ClientConfig>
}

impl<T: crate::utils::AsyncConnector> AsyncConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	pub fn new(inner: T, host: r#async::webpki::DNSName, cfg: Arc<r#async::rustls::ClientConfig>) -> Self {
		Self { inner, host, cfg }
	}
}

impl<T: crate::utils::AsyncConnector> crate::utils::AsyncConnector for AsyncConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	type Connection = r#async::client::TlsStream<T::Connection>;
	//type Future = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		let cfg = self.cfg.clone();
		Box::pin(async move {
			let stream = f.await?;
			r#async::TlsConnector::from(cfg).connect(self.host.as_ref(), stream).await
		})
	}
}

pub struct AsyncAcceptor<T: crate::utils::AsyncAcceptor> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	inner:    T,
	acceptor: r#async::TlsAcceptor
}

impl<T: crate::utils::AsyncAcceptor> AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	pub fn new(inner: T, cfg: Arc<r#async::rustls::ServerConfig>) -> Self {
		Self { inner, acceptor: r#async::TlsAcceptor::from(cfg) }
	}
}

impl<T: crate::utils::AsyncAcceptor> crate::utils::AsyncAcceptor for AsyncAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	type Connection = r#async::server::TlsStream<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		let Self { inner, acceptor } = unsafe { Pin::into_inner_unchecked(self) };
		let acceptor = acceptor.clone();
		match unsafe { Pin::new_unchecked(inner) }.poll_accept(cx) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async move { acceptor.accept(f.await?).await }))
		}
	}
}

pub struct AlpnConnector<T: crate::utils::Connector> where T::Connection: io::Read + io::Write {
	inner: Connector<T>
}

impl<T: crate::utils::Connector> AlpnConnector<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: Connector<T>) -> Self {
		Self { inner }
	}
}

impl<T: crate::utils::Connector> crate::utils::Connector for AlpnConnector<T> where T::Connection: io::Read + io::Write {
	type Connection = AlpnClientConnection<StreamOwned<ClientConnection, T::Connection>>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		let mut stream = self.inner.connect()?;
		io::Write::write(&mut stream, &[])?; // do handshake
		Ok(match stream.conn.alpn_protocol() {
			Some(tls::alpn::DOT)    => AlpnClientConnection::Dns(dns::Connection::new(stream)),
			Some(tls::alpn::HTTP11) => AlpnClientConnection::Http(http::v1::Connection::new(buffered::BufStream::new(stream))),
			_                       => return Err(io::Error::new(io::ErrorKind::Other, "unsupported ALPN protocol"))
		})
	}
}

pub enum AlpnClientConnection<T: io::Read + io::Write> {
	Dns(dns::Connection<T>),
	Http(http::v1::Connection<buffered::BufStream<T>>),
	Ldap(ldap::Connection<T>),
	Rtsp(rtsp::Connection<buffered::BufStream<T>>),
	Smtp(smtp::ClientConnection<buffered::BufStream<T>>),
	Ws(ws::Connection<T>)
}

pub struct AlpnAcceptor<T: crate::utils::Acceptor> where T::Connection: io::Read + io::Write {
	inner: Acceptor<T>
}

impl<T: crate::utils::Acceptor> AlpnAcceptor<T> where T::Connection: io::Read + io::Write {
	pub fn new(inner: Acceptor<T>) -> Self {
		Self { inner }
	}
}

impl<T: crate::utils::Acceptor> crate::utils::Acceptor for AlpnAcceptor<T> where T::Connection: io::Read + io::Write {
	type Connection = AlpnServerConnection<StreamOwned<ServerConnection, T::Connection>>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		let mut stream = self.inner.accept()?;
		io::Write::write(&mut stream, &[])?; // do handshake
		Ok(match stream.conn.alpn_protocol() {
			Some(tls::alpn::DOT)    => AlpnServerConnection::Dns(dns::Connection::new(stream)),
			Some(tls::alpn::HTTP11) => AlpnServerConnection::Http(http::v1::Connection::new(buffered::BufStream::new(stream))),
			_                       => return Err(io::Error::new(io::ErrorKind::Other, "unsupported ALPN protocol"))
		})
	}
}

pub enum AlpnServerConnection<T: io::Read + io::Write> {
	Dns(dns::Connection<T>),
	Http(http::v1::Connection<buffered::BufStream<T>>),
	Rtsp(rtsp::Connection<buffered::BufStream<T>>),
	Smtp(smtp::ServerConnection<buffered::BufStream<T>>),
	Ws(ws::Connection<T>)
}

pub struct AsyncAlpnConnector<T: crate::utils::AsyncConnector> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	inner: AsyncConnector<T>
}

impl<T: crate::utils::AsyncConnector> AsyncAlpnConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	pub fn new(inner: AsyncConnector<T>) -> Self {
		Self { inner }
	}
}

impl<T: crate::utils::AsyncConnector> crate::utils::AsyncConnector for AsyncAlpnConnector<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	type Connection = AsyncAlpnClientConnection<r#async::client::TlsStream<T::Connection>>;
	//type Future = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect(&self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>> {
		todo!()
	}
}

pub enum AsyncAlpnClientConnection<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> {
	Dns(dns::AsyncConnection<T>),
	Http(http::v1::AsyncConnection<buffered::AsyncBufStream<T>>),
	Ldap(ldap::AsyncConnection<T>),
	Rtsp(rtsp::AsyncConnection<buffered::AsyncBufStream<T>>),
	Smtp(smtp::AsyncClientConnection<buffered::AsyncBufStream<T>>),
	Ws(ws::AsyncConnection<T>)
}

pub struct AsyncAlpnAcceptor<T: crate::utils::AsyncAcceptor> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	inner: AsyncAcceptor<T>
}

impl<T: crate::utils::AsyncAcceptor> AsyncAlpnAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	pub fn new(inner: AsyncAcceptor<T>) -> Self {
		Self { inner }
	}
}

impl<T: crate::utils::AsyncAcceptor> crate::utils::AsyncAcceptor for AsyncAlpnAcceptor<T> where T::Connection: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Unpin {
	type Connection = AsyncAlpnServerConnection<r#async::server::TlsStream<T::Connection>>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		todo!()
	}
}

pub enum AsyncAlpnServerConnection<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> {
	Dns(dns::AsyncConnection<T>),
	Http(http::v1::AsyncConnection<buffered::AsyncBufStream<T>>),
	Ldap(ldap::AsyncConnection<T>),
	Rtsp(rtsp::AsyncConnection<buffered::AsyncBufStream<T>>),
	Smtp(smtp::AsyncServerConnection<buffered::AsyncBufStream<T>>),
	Ws(ws::AsyncConnection<T>)
}