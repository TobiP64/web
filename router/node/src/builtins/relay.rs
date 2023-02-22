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

use std::io;
use {
	super::*,
	crate::{interfaces::*, utils::*},
	std::time::Duration,
	net::{
		http::{self, traits::{AsyncSharedConnectionExt, AsyncStreamExt}},
		utils::connection::*,
	},
	smol::{io::{AsyncReadExt, AsyncWriteExt}}
};

const LOCALHOST:        &str = "localhost";
const DEFAULT_BUF_SIZE: usize = 0x1000;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
	#[serde(flatten)]
	pub socket:         super::ConfigSocket,
	pub buf_len:        Option<usize>,
	#[serde(default = "usize_zero")]
	pub retries:        usize,
	pub retry_interval: Option<Duration>,
	pub retry_backoff:  Option<Duration>
}

pub(super) async fn run(name: &str, cfg: Config) -> Result<()> {
	let id      = crate::component_id(name);
	let name    = name.to_string();
	let buf_len = cfg.buf_len.unwrap_or(DEFAULT_BUF_SIZE);
	
	fn endpoint(tcp: &ConfigSocketTcp, default_port: u16) -> String {
		format!(
			"{}:{}",
			tcp.host.as_deref().unwrap_or(LOCALHOST),
			tcp.port.unwrap_or(default_port)
		)
	}
	
	match cfg.socket {
		ConfigSocket { pipe: None, tcp: Some(tcp), udp: None, tls: None, http1: Some(http), .. } =>
			crate::add_component::<HttpStreamHandler>(id, Box::new(ModuleShared::new(
				name, buf_len, net::http::traits::DynAsyncSharedConnector::new(
					net::http::v1::AsyncSharedConnector::new(
						net::http::v1::AsyncConnector::new(
							net::buffered::AsyncConnector::new(
								net::tcp::AsyncConnector::new(
									endpoint(&tcp, net::http::v1::DEFAULT_PORT_TLS))))))).await?)),
		ConfigSocket { pipe: None, tcp: Some(tcp), udp: None, .. } =>
			crate::add_component::<ByteStreamHandler>(id, Box::new(Module::new(
				name, buf_len, net::tcp::AsyncConnector::new(
					endpoint(&tcp, 1024))))),
		_ => return Err("invalid config".into())
	};
	
	Ok(())
}

struct Module<T: AsyncConnector> {
	name:      String,
	buf_len:   usize,
	connector: T
}

impl<T: AsyncConnector> Module<T> {
	fn new(name: String, buf_len: usize, connector: T) -> Self {
		Self { name, buf_len, connector }
	}
}

impl<T: AsyncConnector<Connection = smol::net::TcpStream>> StreamHandler<dyn AsyncByteStream> for Module<T> {
	fn accept<'a>(&'a self, _stream: &'static mut dyn AsyncByteStream) -> DynFuture<'a, Result<()>> {
		Box::pin(async move {
			let conn = self.connector.connect().await?;
			todo!();
		})
	}
}

struct ModuleShared<T: AsyncConnector> {
	name:       String,
	buf_len:    usize,
	connector:  T,
	connection: smol::lock::RwLock<T::Connection>
}

impl<T: AsyncConnector> ModuleShared<T> {
	async fn new(name: String, buf_len: usize, connector: T) -> io::Result<Self> {
		let connection = smol::lock::RwLock::new(connector.connect().await?);
		Ok(Self { name, buf_len, connector, connection })
	}
}

impl<T: AsyncConnector<Connection = http::traits::BoxedAsyncSharedConnection>> StreamHandler<dyn http::traits::AsyncStream> for ModuleShared<T> {
	fn accept<'a>(&'a self, stream_src: &'static mut dyn http::traits::AsyncStream) -> DynFuture<'a, Result<()>> {
		Box::pin(async move {
			let mut buf = Vec::with_capacity(self.buf_len);
			unsafe { buf.set_len(self.buf_len) }; // SAFE: len matches capacity
			let conn = self.connection.read().await;
			let id = conn.open().await?;
			let mut stream_dst = http::AsyncStream(&*conn, id);
			
			let mut headers = stream_src.read_headers().await?;
			stream_dst.write_headers(&headers).await?;
			
			let len = match headers.iter().find_map(http::Header::as_method) {
				None => return send_response(stream_src, http::Status::BadRequest).await,
				Some(http::Method::Get| http::Method::Head) => Some(0),
				_ => headers.iter().find_map(http::Header::as_content_length).copied()
			};
			
			match len {
				Some(mut len) => while len > 0 {
					let __len__ = buf.len().min(len);
					let read = stream_src.read(&mut buf[..__len__]).await?;
					stream_dst.write_all(&buf[..read]).await?;
					len -= read;
				},
				None => loop {
					let read = stream_src.read(&mut buf).await?;
					
					if read == 0 {
						break;
					}
					
					stream_dst.write_all(&buf[..read]).await?;
				}
			}
			
			headers.clear();
			stream_dst.flush().await?;
			let headers = stream_dst.read_headers().await?;
			stream_src.write_headers(&headers).await?;
			
			let len = match headers.iter().find_map(http::Header::as_status) {
				None => {
					log::warn!("backend `{}`: failed to transmit response: :status header missing", &self.name);
					return send_response(stream_src, http::Status::InternalServerError).await;
				}
				Some(http::Status::NoContent) => Some(0),
				_ => headers.iter().find_map(http::Header::as_content_length).copied()
			};
			
			match len {
				Some(mut len) => while len > 0 {
					let __len__ = buf.len().min(len);
					let read = stream_dst.read(&mut buf[..__len__]).await?;
					stream_src.write_all(&buf[..read]).await?;
					len -= read;
				},
				None => loop {
					let read = stream_dst.read(&mut buf).await?;
					
					if read == 0 {
						break;
					}
					
					stream_src.write_all(&buf[..read]).await?;
				}
			}
			
			stream_src.flush().await?;
			Ok(())
		})
	}
}