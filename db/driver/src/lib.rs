// MIT License
//
// Copyright (c) 2022 Tobias Pfeiffer
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

use std::borrow::Cow;
use std::sync::{Arc, RwLock};

pub use net::tls;

extern crate kranus_protocols as net;

pub mod cursor;
pub mod query;
pub mod terms;
pub mod wire;
pub mod topology;

type Result<T> = std::result::Result<T, Error>;

pub const DEFAULT_HOST: &str = "localost";
pub const DEFAULT_PORT: u16  = 443;

#[derive(Clone)]
pub struct ClientOptions {
	/// the host to connect to (default `localhost`)
	pub hostname: Cow<'static, str>,
	/// the port to connect on (default `443`)
	pub port:     u16,
	pub tls:      Arc<net::tls::ClientConfig>,
	/// connect to cluster nodes with one of these labels
	pub labels:   Vec<String>
}

impl Default for ClientOptions {
    fn default() -> Self {
		Self {
			hostname: Cow::Borrowed(DEFAULT_HOST),
			port:     DEFAULT_PORT,
			tls:      Arc::new(net::tls::ClientConfig::builder()
				.with_safe_defaults()
				.with_root_certificates(net::tls::RootCertStore::empty())
				.with_no_client_auth()),
			labels:   Vec::new(),
		}
    }
}

impl ClientOptions {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn connect(self) -> Result<Client> {
		Client::connect(self)
	}

	#[cfg(feature = "async")]
	pub async fn connect_async(self) -> Result<AsyncClient> {
		AsyncClient::connect(self).await
	}
}

struct ClientInner {
	options:  ClientOptions,
	topology: RwLock<()>
}

#[derive(Clone)]
pub struct Client(Arc<ClientInner>);

impl Client {
	pub fn connect(options: ClientOptions) -> Result<Self> {
		todo!()
	}
}

#[cfg(feature = "async")]
struct AsyncClientInner {
	options:  ClientOptions,
	topology: smol::lock::RwLock<()>
}

#[cfg(feature = "async")]
#[derive(Clone)]
pub struct AsyncClient(Arc<AsyncClientInner>);

#[cfg(feature = "async")]
impl AsyncClient {
	pub async fn connect(options: ClientOptions) -> Result<Self> {
		todo!()
	}
}

#[derive(Debug)]
pub enum Error {
	Sync(&'static str),
	Io(std::io::Error),
	Tls(net::tls::Error),
	Dns(net::webpki::InvalidDnsNameError),
	Server(String)
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Self::Io(e)
	}
}

impl<T> From<std::sync::PoisonError<T>> for Error {
	fn from(_: std::sync::PoisonError<T>) -> Self {
		Self::Sync("poison error")
	}
}

impl From<net::tls::Error> for Error {
	fn from(e: net::tls::Error) -> Self {
		Self::Tls(e)
	}
}

impl From<net::webpki::InvalidDnsNameError> for Error {
	fn from(e: net::webpki::InvalidDnsNameError) -> Self {
		Self::Dns(e)
	}
}
