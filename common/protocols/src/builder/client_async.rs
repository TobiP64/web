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

use {crate::{*, utils::AsyncConnector}, std::{io, sync::Arc}, smol::net};

pub struct Builder;

impl Builder {
	pub fn addr<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(self, addr: T) -> Tcp<T> {
		Tcp { addr }
	}
	
	pub fn quic<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(self, addr: T) -> Quic<T> {
		Quic { addr }
	}
}

pub struct Tcp<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	addr: T
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Tcp<T> {
	pub fn connector(self) -> tcp::AsyncConnector<T> {
		tcp::AsyncConnector::new(self.addr)
	}
	
	pub async fn connect(self) -> io::Result<net::TcpStream> {
		self.connector().connect().await
	}
	
	pub fn tls(self, name: tls::r#async::webpki::DNSName) -> Tls<T> {
		Tls {
			inner: self,
			name,
			cfg:   Arc::new(tls::r#async::rustls::ClientConfig::new())
		}
	}
	
	pub fn dns(self) -> Dns<T> {
		Dns { inner: self }
	}
	
	pub fn http(self) -> Http<T> {
		Http { inner: self }
	}
	
	pub fn ldap(self) -> Ldap<T> {
		Ldap { inner: self }
	}
	
	pub fn rtsp(self) -> Rtsp<T> {
		Rtsp { inner: self }
	}
	
	pub fn smtp(self) -> Smtp<T> {
		Smtp { inner: self }
	}
	
	pub fn ws(self) -> Ws<T> {
		Ws { inner: self }
	}
}

pub struct Dns<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Dns<T> {
	pub fn connector(self) -> dns::AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
		dns::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<dns::AsyncConnection<buffered::AsyncBufStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Http<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Http<T> {
	pub fn connector(self) -> http::v1::AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
		http::v1::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<http::v1::AsyncConnection<buffered::AsyncBufStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Ldap<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Ldap<T> {
	pub fn connector(self) -> ldap::AsyncConnector<tcp::AsyncConnector<T>> {
		ldap::AsyncConnector::new(self.inner.connector())
	}
	
	pub async fn connect(self) -> io::Result<ldap::AsyncConnection<net::TcpStream>> {
		self.connector().connect().await
	}
}

pub struct Rtsp<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Rtsp<T> {
	pub fn connector(self) -> rtsp::AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
		rtsp::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<rtsp::AsyncConnection<buffered::AsyncBufStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Smtp<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Smtp<T> {
	pub fn connector(self) -> smtp::AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
		smtp::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<smtp::AsyncClientConnection<buffered::AsyncBufStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Ws<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Ws<T> {
	pub fn connector(self) -> ws::AsyncConnector<tcp::AsyncConnector<T>> {
		ws::AsyncConnector::new(self.inner.connector())
	}
	
	pub async fn connect(self) -> io::Result<ws::AsyncConnection<net::TcpStream>> {
		self.connector().connect().await
	}
}

pub struct Tls<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tcp<T>,
	name:  async_rustls::webpki::DNSName,
	cfg:   Arc<tls::r#async::rustls::ClientConfig>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Tls<T> {
	pub fn with_config(mut self, cfg: Arc<tls::r#async::rustls::ClientConfig>) -> Self {
		self.cfg = cfg;
		self
	}
	
	pub fn connector(self) -> tls::AsyncConnector<tcp::AsyncConnector<T>> {
		tls::AsyncConnector::new(self.inner.connector(), self.name, self.cfg)
	}
	
	pub async fn connect(self) -> io::Result<tls::r#async::client::TlsStream<net::TcpStream>> {
		self.connector().connect().await
	}
	
	pub fn alpn(self) -> Alpn<T> {
		Alpn { inner: self }
	}
	
	pub fn dot(self) -> Dot<T> {
		Dot { inner: self }
	}
	
	pub fn http(self) -> Https<T> {
		Https { inner: self }
	}
	
	pub fn ldap(self) -> Ldaps<T> {
		Ldaps { inner: self }
	}
	
	pub fn rtsp(self) -> Rtsps<T> {
		Rtsps { inner: self }
	}
	
	pub fn smtp(self) -> Smtps<T> {
		Smtps { inner: self }
	}
	
	pub fn ws(self) -> Wss<T> {
		Wss { inner: self }
	}
}

pub struct Dot<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Dot<T> {
	pub fn connector(self) -> dns::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>> {
		dns::AsyncConnector::new(self.inner.connector())
	}
	
	pub async fn connect(self) -> io::Result<dns::AsyncConnection<tls::r#async::client::TlsStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Https<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Https<T> {
	pub fn connector(self) -> http::v1::AsyncConnector<buffered::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>>> {
		http::v1::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<http::v1::AsyncConnection<buffered::AsyncBufStream<tls::r#async::client::TlsStream<net::TcpStream>>>> {
		self.connector().connect().await
	}
}

pub struct Ldaps<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Ldaps<T> {
	pub fn connector(self) -> ldap::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>> {
		ldap::AsyncConnector::new(self.inner.connector())
	}
	
	pub async fn connect(self) -> io::Result<ldap::AsyncConnection<tls::r#async::client::TlsStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Rtsps<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Rtsps<T> {
	pub fn connector(self) -> rtsp::AsyncConnector<buffered::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>>> {
		rtsp::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<rtsp::AsyncConnection<buffered::AsyncBufStream<tls::r#async::client::TlsStream<net::TcpStream>>>> {
		self.connector().connect().await
	}
}

pub struct Smtps<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Smtps<T> {
	pub fn connector(self) -> smtp::AsyncConnector<buffered::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>>> {
		smtp::AsyncConnector::new(buffered::AsyncConnector::new(self.inner.connector()))
	}
	
	pub async fn connect(self) -> io::Result<smtp::AsyncClientConnection<buffered::AsyncBufStream<tls::r#async::client::TlsStream<net::TcpStream>>>> {
		self.connector().connect().await
	}
}

pub struct Wss<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Wss<T> {
	pub fn connector(self) -> ws::AsyncConnector<tls::AsyncConnector<tcp::AsyncConnector<T>>> {
		ws::AsyncConnector::new(self.inner.connector())
	}
	
	pub async fn connect(self) -> io::Result<ws::AsyncConnection<tls::r#async::client::TlsStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Alpn<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	inner: Tls<T>
}

impl<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> Alpn<T> {
	pub fn connector(self) -> tls::AsyncAlpnConnector<tcp::AsyncConnector<T>> {
		tls::AsyncAlpnConnector::new(tls::AsyncConnector::new(tcp::AsyncConnector::new(self.inner.inner.addr), self.inner.name, self.inner.cfg))
	}
	
	pub async fn connect(self) -> io::Result<tls::AsyncAlpnClientConnection<tls::r#async::client::TlsStream<net::TcpStream>>> {
		self.connector().connect().await
	}
}

pub struct Quic<T: 'static + net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	addr: T
}