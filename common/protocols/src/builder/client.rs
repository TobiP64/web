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

use {crate::{*, utils::Connector}, std::{io, net, sync::Arc}};

pub struct Builder;

impl Builder {
	pub fn addr<T: net::ToSocketAddrs>(self, addr: T) -> Tcp<T> {
		Tcp { addr }
	}
	
	pub fn quic<T: net::ToSocketAddrs>(self, addr: T) -> Quic<T> {
		Quic { addr }
	}
}

pub struct Tcp<T: net::ToSocketAddrs> {
	addr: T
}

impl<T: net::ToSocketAddrs> Tcp<T> {
	pub fn connector(self) -> tcp::Connector<T> {
		tcp::Connector::new(self.addr)
	}
	
	pub fn connect(self) -> io::Result<net::TcpStream> {
		self.connector().connect()
	}
	
	pub fn tls(self, name: tls::ServerName) -> Tls<T> {
		Tls {
			inner: self,
			name,
			cfg:   Arc::new(tls::ClientConfig::builder()
				.with_safe_defaults()
				.with_root_certificates(tls::RootCertStore::empty())
				.with_no_client_auth())
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

pub struct Dns<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Dns<T> {
	pub fn connector(self) -> dns::Connector<buffered::Connector<tcp::Connector<T>>> {
		dns::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<dns::Connection<buffered::BufStream<net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Http<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Http<T> {
	pub fn connector(self) -> http::v1::Connector<buffered::Connector<tcp::Connector<T>>> {
		http::v1::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<http::v1::Connection<buffered::BufStream<net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Ldap<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Ldap<T> {
	pub fn connector(self) -> ldap::Connector<tcp::Connector<T>> {
		ldap::Connector::new(self.inner.connector())
	}
	
	pub fn connect(self) -> io::Result<ldap::Connection<net::TcpStream>> {
		self.connector().connect()
	}
}

pub struct Rtsp<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Rtsp<T> {
	pub fn connector(self) -> rtsp::Connector<buffered::Connector<tcp::Connector<T>>> {
		rtsp::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<rtsp::Connection<buffered::BufStream<net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Smtp<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Smtp<T> {
	pub fn connector(self) -> smtp::Connector<buffered::Connector<tcp::Connector<T>>> {
		smtp::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<smtp::ClientConnection<buffered::BufStream<net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Ws<T: net::ToSocketAddrs> {
	inner: Tcp<T>
}

impl<T: net::ToSocketAddrs> Ws<T> {
	pub fn connector(self) -> ws::Connector<tcp::Connector<T>> {
		ws::Connector::new(self.inner.connector())
	}
	
	pub fn connect(self) -> io::Result<ws::Connection<net::TcpStream>> {
		self.connector().connect()
	}
}

pub struct Tls<T: net::ToSocketAddrs> {
	inner: Tcp<T>,
	name:  tls::ServerName,
	cfg:   Arc<tls::ClientConfig>
}

impl<T: net::ToSocketAddrs> Tls<T> {
	pub fn with_config(mut self, cfg: Arc<tls::ClientConfig>) -> Self {
		self.cfg = cfg;
		self
	}
	
	pub fn connector(self) -> tls::Connector<tcp::Connector<T>> {
		tls::Connector::new(self.inner.connector(), self.name, self.cfg)
	}
	
	pub fn connect(self) -> io::Result<tls::StreamOwned<tls::ClientConnection, net::TcpStream>> {
		self.connector().connect()
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

pub struct Dot<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Dot<T> {
	pub fn connector(self) -> dns::Connector<tls::Connector<tcp::Connector<T>>> {
		dns::Connector::new(self.inner.connector())
	}
	
	pub fn connect(self) -> io::Result<dns::Connection<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Https<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Https<T> {
	pub fn connector(self) -> http::v1::Connector<buffered::Connector<tls::Connector<tcp::Connector<T>>>> {
		http::v1::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<http::v1::Connection<buffered::BufStream<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>>> {
		self.connector().connect()
	}
}

pub struct Ldaps<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Ldaps<T> {
	pub fn connector(self) -> ldap::Connector<tls::Connector<tcp::Connector<T>>> {
		ldap::Connector::new(self.inner.connector())
	}
	
	pub fn connect(self) -> io::Result<ldap::Connection<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Rtsps<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Rtsps<T> {
	pub fn connector(self) -> rtsp::Connector<buffered::Connector<tls::Connector<tcp::Connector<T>>>> {
		rtsp::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<rtsp::Connection<buffered::BufStream<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>>> {
		self.connector().connect()
	}
}

pub struct Smtps<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Smtps<T> {
	pub fn connector(self) -> smtp::Connector<buffered::Connector<tls::Connector<tcp::Connector<T>>>> {
		smtp::Connector::new(buffered::Connector::new(self.inner.connector()))
	}
	
	pub fn connect(self) -> io::Result<smtp::ClientConnection<buffered::BufStream<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>>> {
		self.connector().connect()
	}
}

pub struct Wss<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Wss<T> {
	pub fn connector(self) -> ws::Connector<tls::Connector<tcp::Connector<T>>> {
		ws::Connector::new(self.inner.connector())
	}
	
	pub fn connect(self) -> io::Result<ws::Connection<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Alpn<T: net::ToSocketAddrs> {
	inner: Tls<T>
}

impl<T: net::ToSocketAddrs> Alpn<T> {
	pub fn connector(self) -> tls::AlpnConnector<tcp::Connector<T>> {
		tls::AlpnConnector::new(tls::Connector::new(tcp::Connector::new(self.inner.inner.addr), self.inner.name, self.inner.cfg))
	}
	
	pub fn connect(self) -> io::Result<tls::AlpnClientConnection<tls::StreamOwned<tls::ClientConnection, net::TcpStream>>> {
		self.connector().connect()
	}
}

pub struct Quic<T: net::ToSocketAddrs> {
	addr: T
}
