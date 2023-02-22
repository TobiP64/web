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

//! Hyper Text Transfer Protocol version 1.1
//!
//! [RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616/)

use {
	super::{traits::{self, StreamId, Read}, *},
	crate::*,
	std::{io, net, pin::Pin, task::{Context, Poll}, convert::TryFrom, sync::Arc, future::Future},
	futures_lite::io::*
};

pub const DEFAULT_PORT:     u16 = 80;
pub const DEFAULT_PORT_TLS: u16 = 443;

const DEFAULT_BUF_LEN: usize = 256;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
	Ready,
	ReadRequestHeaders { pseudo: bool, len: Option<usize> },
	ReadRequestBody(usize),
	WriteRequestHeaders { len: Option<usize> },
	WriteRequestBody(usize),
	ReadResponseHeaders { pseudo: bool, len: Option<usize> },
	ReadResponseBody(usize),
	WriteResponseHeaders { len: Option<usize> },
	WriteResponseBody(usize),
	Closed
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AsyncState {
	Ready,
	ReadRequestHeaders { pseudo: bool, len: Option<usize> },
	ReadRequestBody(usize),
	WriteRequestHeaders { rem: usize, len: Option<usize> },
	WriteRequestNewLine { rem: usize, len: usize },
	WriteRequestBody(usize),
	ReadResponseHeaders { pseudo: bool, len: Option<usize> },
	ReadResponseBody(usize),
	WriteResponseHeaders { rem: usize, len: Option<usize> },
	WriteResponseNewLine { rem: usize, len: usize },
	WriteResponseBody(usize),
	Closed
}

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
	type Connection = Connection<T::Connection>;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		self.inner.connect().map(Connection::new)
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
	type Connection = Connection<T::Connection>;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.inner.accept().map(Connection::new)
	}
}

pub struct Connection<T: io::BufRead + io::Write> {
	inner:  T,
	stream: usize,
	state:  State,
	buf:    Vec<u8>
}

impl<T: io::BufRead + io::Write> Connection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner, stream: 0, state: State::Ready, buf: Vec::with_capacity(DEFAULT_BUF_LEN) }
	}
}

impl<T: io::BufRead + io::Write> traits::Connection for Connection<T> {
	fn open(&mut self) -> Result<StreamId> {
		if self.state != State::Ready {
			panic!("invalid state");
		}
		
		self.stream += 1;
		self.state = State::WriteRequestHeaders { len: None };
		Ok(self.stream)
	}
	
	fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<Option<(StreamId, Read<'a>)>> {
		let mut headers = Vec::new();
		
		match &mut self.state {
			State::Ready => {
				if self.inner.fill_buf()?.is_empty() {
					return Ok(None);
				}
				
				self.stream += 1;
				self.state = State::ReadRequestHeaders { pseudo: false, len: None };
				Ok(Some((self.stream, Read::Opened)))
			}
			State::ReadRequestHeaders { .. } | State::ReadResponseHeaders { .. } => loop {
				self.buf.clear();
				self.inner.read_until(b'\n', &mut self.buf)?;
				let s = to_trimmed_utf8(&self.buf)?;
				
				match (&mut self.state, s.is_empty()) {
					(State::ReadRequestHeaders { pseudo: pseudo @ false, len }, false) => {
						let (method, s) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						let method = method.parse()
							.map_err(header_error)?;
						
						if method == Method::Get || method == Method::Post {
							*len = Some(0);
						}
						
						headers.push(Header::Method(method));
						
						let (path, proto) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						headers.push(Header::Path(path.to_string()));
						
						if !matches!(proto, "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1") {
							return Err(io::Error::new(io::ErrorKind::Other, "Invalid protocol"));
						}
						
						*pseudo = true;
					}
					(State::ReadResponseHeaders { pseudo: pseudo @ false, .. }, false) => {
						let (proto, s) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						if !matches!(proto, "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1") {
							return Err(io::Error::new(io::ErrorKind::Other, "Invalid protocol"))
						}
						
						let (status, _) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						headers.push(Header::Status(s.trim()
							.parse::<usize>()
							.map_err(header_error)?
							.try_into()
							.map_err(header_error)?));
						*pseudo = true;
					}
					(State::ReadRequestHeaders { pseudo: true, len }, false)
					| (State::ReadResponseHeaders { pseudo: true, len }, false) => {
						let (key, val) = s.split_once(':').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						let header = Header::parse_name_value(key, val);
						
						if let Header::ContentLength(v) = &header {
							*len = Some(*v);
						}
						
						headers.push(header);
					}
					(State::ReadRequestHeaders { pseudo: false, .. }, true)
					| (State::ReadResponseHeaders { pseudo: false, .. }, true) =>
						return Err(io::Error::new(io::ErrorKind::Other, "stream is empty")),
					(State::ReadRequestHeaders { pseudo: true, len }, true) => {
						self.state = State::ReadRequestBody(len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?);
						return Ok(Some((self.stream, Read::Headers(headers))));
					}
					(State::ReadResponseHeaders { pseudo: true, len }, true) => {
						self.state = State::ReadResponseBody(len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?);
						return Ok(Some((self.stream, Read::Headers(headers))));
					}
					_ => unreachable!()
				}
			}
			State::ReadRequestBody(0) => {
				self.state = State::WriteResponseHeaders { len: None };
				Ok(Some((self.stream, Read::Closed)))
			}
			State::ReadResponseBody(0) => {
				self.state = State::Ready;
				Ok(Some((self.stream, Read::Closed)))
			}
			State::ReadResponseBody(rem) | State::ReadRequestBody(rem) => {
				let __buf_len__ = buf.len();
				let read = self.inner.read(&mut buf[..__buf_len__.min(*rem)])?;
				*rem -= read;
				Ok(Some((self.stream, Read::Body(&buf[..read]))))
			}
			_ => panic!("invalid state")
		}
	}
	
	fn write_headers(&mut self, id: StreamId, headers: &[Header]) -> Result<()> {
		if self.stream != id {
			panic!("invalid stream id");
		}
		
		match &mut self.state {
			State::WriteRequestHeaders { len } | State::WriteResponseHeaders { len } => {
				for header in headers {
					match header {
						Header::Method(Method::Get) | Header::Method(Method::Head) => *len = Some(0),
						Header::ContentLength(v) => *len = Some(*v),
						_ => ()
					}
					
					match header {
						Header::Method(v) => write!(&mut self.inner, "{}", v),
						Header::Path(v)   => write!(&mut self.inner, " {} HTTP/1.1\r\n", v),
						Header::Status(v) => write!(&mut self.inner, "HTTP/1.1 {} {}\r\n", *v as u32, v),
						header            => write!(&mut self.inner, "{}: {}\r\n", header.name_v1(), header),
					}?;
				}
				
				Ok(())
			}
			_ => panic!("invalid state")
		}
	}
	
	fn write_body(&mut self, id: StreamId, buf: &[u8]) -> Result<()> {
		if self.stream != id {
			panic!("invalid stream id");
		}
		
		loop {
			match &mut self.state {
				State::WriteResponseHeaders { len } => {
					self.inner.write_all(b"\r\n")?;
					self.state = State::WriteResponseBody(len.ok_or_else(||
						io::Error::new(io::ErrorKind::Other, "Content-Length header not present"))?);
				}
				State::WriteRequestHeaders { len } => {
					self.inner.write_all(b"\r\n")?;
					self.state = State::WriteRequestBody(len.ok_or_else(||
						io::Error::new(io::ErrorKind::Other, "Content-Length header not present"))?);
				}
				State::WriteRequestBody(0) => {
					self.state = State::ReadResponseHeaders { pseudo: false, len: None };
					return Ok(());
				}
				State::WriteResponseBody(0) => {
					self.state = State::Ready;
					return Ok(());
				}
				State::WriteRequestBody(rem) | State::WriteResponseBody(rem) => {
					self.inner.write_all(buf)?;
					*rem -= buf.len();
					return Ok(());
				}
				_ => panic!("invalid state")
			}
		}
	}
	
	fn close(&mut self, id: StreamId) -> Result<()> {
		if self.state != State::Ready {
			self.write_body(id, &[])?;
		}
		
		self.inner.flush()?;
		Ok(())
	}
}

#[cfg(feature = "smol")]
pub fn connector_async<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync>(addr: T) -> AsyncConnector<buffered::AsyncConnector<tcp::AsyncConnector<T>>> {
	AsyncConnector::new(buffered::AsyncConnector::new(tcp::AsyncConnector::new(addr)))
}

#[cfg(feature = "smol")]
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
	type Connection = AsyncConnection<T::Connection>;
	//type Future     = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		Box::pin(async move { f.await.map(AsyncConnection::new) })
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
	type Connection = AsyncConnection<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_accept(cx) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async { f.await.map(AsyncConnection::new) }))
		}
	}
}

pub struct AsyncConnection<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> {
	inner:  T,
	stream: usize,
	state:  AsyncState,
	buf:    Vec<u8>
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> AsyncConnection<T> {
	pub fn new(inner: T) -> Self {
		Self { inner, stream: 0, state: AsyncState::Ready, buf: Vec::with_capacity(DEFAULT_BUF_LEN) }
	}
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite + Send + 'static> traits::AsyncConnection for AsyncConnection<T> {
	fn poll_open(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<StreamId>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		
		if self_.state != AsyncState::Ready {
			panic!("invalid state");
		}
		
		self_.stream += 1;
		self_.state = AsyncState::WriteRequestHeaders { rem: 0, len: None };
		Poll::Ready(Ok(self_.stream))
	}
	
	fn poll_read<'a>(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &'a mut [u8]) -> Poll<Result<Option<(StreamId, Read<'a>)>>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		let __buf_len__ = buf.len();
		let mut headers = Vec::new();
		
		match &mut self_.state {
			AsyncState::Ready => match unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_fill_buf(cx) {
				Poll::Ready(Ok([])) => Poll::Ready(Ok(None)),
				Poll::Ready(Ok(v))  => {
					self_.stream += 1;
					self_.state = AsyncState::ReadRequestHeaders { pseudo: false, len: None };
					Poll::Ready(Ok(Some((self_.stream, Read::Opened))))
				}
				Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
				Poll::Pending       => Poll::Pending
			}
			AsyncState::ReadRequestHeaders { pseudo, len } | AsyncState::ReadResponseHeaders { pseudo, len } => loop {
				self_.buf.clear();
				match read_until_internal(unsafe { Pin::new_unchecked(&mut self_.inner) }, cx, b'\n', &mut self_.buf) {
					Poll::Ready(Ok(())) => (),
					Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
					Poll::Pending       => return Poll::Pending
				}
				
				let s = to_trimmed_utf8(&self_.buf)?;
				
				match (&mut self_.state, s.is_empty()) {
					(AsyncState::ReadRequestHeaders { pseudo: pseudo @ false, len }, false) => {
						let (method, s) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						let method = method.parse()
							.map_err(header_error)?;
						
						if method == Method::Get || method == Method::Post {
							*len = Some(0);
						}
						
						headers.push(Header::Method(method));
						
						let (path, proto) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						headers.push(Header::Path(path.to_string()));
						
						if !matches!(proto, "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1") {
							return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Invalid protocol")))
						}
						
						*pseudo = true;
					}
					(AsyncState::ReadResponseHeaders { pseudo: pseudo @ false, .. }, false) => {
						let (proto, s) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						if !matches!(proto, "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1") {
							return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Invalid protocol")))
						}
						
						let (status, _) = s.split_once(' ').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						
						headers.push(Header::Status(s.trim()
							.parse::<usize>()
							.map_err(header_error)?
							.try_into()
							.map_err(header_error)?));
						*pseudo = true;
					}
					(AsyncState::ReadRequestHeaders { pseudo: true, len }, false)
					| (AsyncState::ReadResponseHeaders { pseudo: true, len }, false) => {
						let (key, val) = s.split_once(':').ok_or_else(|| Error::new(
							ErrorKind::InvalidData, "failed to parse HTTP header"))?;
						let header = Header::parse_name_value(key.trim(), val.trim());
						
						if let Header::ContentLength(v) = &header {
							*len = Some(*v);
						}
						
						headers.push(header);
					}
					(AsyncState::ReadRequestHeaders { pseudo: false, .. }, true)
					| (AsyncState::ReadResponseHeaders { pseudo: false, .. }, true) =>
						return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "stream is empty"))),
					(AsyncState::ReadRequestHeaders { pseudo: true, len }, true) => {
						self_.state = AsyncState::ReadRequestBody(len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?);
						return Poll::Ready(Ok(Some((self_.stream, Read::Headers(headers)))));
					}
					(AsyncState::ReadResponseHeaders { pseudo: true, len }, true) => {
						self_.state = AsyncState::ReadResponseBody(len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?);
						return Poll::Ready(Ok(Some((self_.stream, Read::Headers(headers)))));
					}
					_ => unreachable!()
				}
			}
			AsyncState::ReadRequestBody(0) => {
				self_.state = AsyncState::WriteResponseHeaders { rem: 0, len: None };
				Poll::Ready(Ok(Some((self_.stream, Read::Closed))))
			}
			AsyncState::ReadResponseBody(0) => {
				self_.state = AsyncState::Ready;
				Poll::Ready(Ok(Some((self_.stream, Read::Closed))))
			}
			AsyncState::ReadResponseBody(rem) | AsyncState::ReadRequestBody(rem) => {
				match unsafe { Pin::new_unchecked(&mut self_.inner) }
					.poll_read(cx, &mut buf[..__buf_len__.min(*rem)]) {
					Poll::Ready(Ok(read)) => {
						*rem -= read;
						Poll::Ready(Ok(Some((self_.stream, Read::Body(&buf[..read])))))
					}
					Poll::Ready(Err(e))   => Poll::Ready(Err(e)),
					Poll::Pending         => Poll::Pending
				}
			}
			_ => panic!("invalid state")
		}
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId, headers: &[Header]) -> Poll<Result<()>> {
		use std::io::Write;
		
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		
		if self_.stream != id {
			panic!("invalid stream id");
		}
		
		loop {
			match &mut self_.state {
				AsyncState::WriteRequestHeaders { rem, len } | AsyncState::WriteResponseHeaders { rem, len } if *rem == 0 => {
					self_.buf.clear();
					
					for header in headers {
						match header {
							Header::Method(Method::Get) | Header::Method(Method::Head) => *len = Some(0),
							Header::ContentLength(v) => *len = Some(*v),
							_ => ()
						}
						
						match header {
							Header::Method(v) => write!(&mut self_.buf, "{}", v),
							Header::Path(v)   => write!(&mut self_.buf, " {} HTTP/1.1\r\n", v),
							Header::Status(v) => write!(&mut self_.buf, "HTTP/1.1 {} {}\r\n", *v as u32, v),
							header            => write!(&mut self_.buf, "{}: {}\r\n", header.name_v1(), header),
						}?;
					}
					
					*rem = self_.buf.len();
				}
				/*AsyncState::WriteRequestHeaders { rem: 0, len, .. }  => {
					self_.state = AsyncState::WriteRequestBody(len.ok_or_else(|| io::Error::new(
						io::ErrorKind::Other, "Content-Length header not present"))?);
					return Poll::Ready(Ok(()));
				}
				AsyncState::WriteResponseHeaders { rem: 0, len, .. } => {
					self_.state = AsyncState::WriteRequestBody(len.ok_or_else(|| io::Error::new(
						io::ErrorKind::Other, "Content-Length header not present"))?);
					return Poll::Ready(Ok(()));
				}*/
				AsyncState::WriteRequestHeaders { rem, .. } | AsyncState::WriteResponseHeaders { rem, .. } =>
					match unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_write(cx, &self_.buf[self_.buf.len() - *rem..]) {
						Poll::Ready(Ok(n)) if n == *rem => {
							*rem = 0;
							return Poll::Ready(Ok(()));
						}
						Poll::Ready(Ok(n))  => *rem -= n,
						Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
						Poll::Pending       => return Poll::Pending
					},
				_ => panic!("invalid state")
			}
		}
	}
	
	fn poll_write_body(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId, buf: &[u8]) -> Poll<Result<()>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		
		if self_.stream != id {
			panic!("invalid stream id");
		}
		
		loop {
			match &mut self_.state {
				AsyncState::WriteResponseHeaders { rem: 0, len } => {
					self_.state = AsyncState::WriteResponseNewLine {
						rem: 2,
						len: len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?
					};
				}
				AsyncState::WriteRequestHeaders { rem: 0, len } => {
					self_.state = AsyncState::WriteRequestNewLine {
						rem: 2,
						len: len.ok_or_else(|| io::Error::new(
							io::ErrorKind::Other, "Content-Length header not present"))?
					};
				}
				AsyncState::WriteResponseNewLine { rem: 0, len } => self_.state = AsyncState::WriteResponseBody(*len),
				AsyncState::WriteRequestNewLine  { rem: 0, len } => self_.state = AsyncState::WriteRequestBody(*len),
				AsyncState::WriteResponseNewLine { rem, len } | AsyncState::WriteRequestNewLine  { rem, len } => {
					match unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_write(cx, &b"\r\n"[2 - *rem..]) {
						Poll::Ready(Ok(n))  => *rem -= n,
						Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
						Poll::Pending       => return Poll::Pending
					}
				}
				AsyncState::WriteRequestBody(0) => {
					self_.state = AsyncState::ReadResponseHeaders { pseudo: false, len: None };
					return Poll::Ready(Ok(()));
				}
				AsyncState::WriteResponseBody(0) => {
					self_.state = AsyncState::Ready;
					return Poll::Ready(Ok(()));
				}
				AsyncState::WriteRequestBody(rem) | AsyncState::WriteResponseBody(rem) => {
					match unsafe { Pin::new_unchecked(&mut self_.inner) }
						.poll_write(cx, &buf[buf.len() - *rem..])
					{
						Poll::Ready(Ok(n))  => *rem -= n,
						Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
						Poll::Pending       => return Poll::Pending
					}
				}
				_ => panic!("invalid state")
			};
		}
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<Result<()>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		
		if self_.stream != id {
			panic!("invalid stream id")
		}
		
		unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_flush(cx)
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<Result<()>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		
		if self_.state != AsyncState::Ready {
			match unsafe { Pin::new_unchecked(&mut*self_) }.poll_write_body(cx, id, &[]) {
				Poll::Ready(Ok(())) => (),
				v => return v
			}
		}
		
		unsafe { Pin::new_unchecked(self_) }.poll_flush(cx, id)
	}
}

pub struct AsyncSharedConnector<T: utils::AsyncConnector> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite {
	inner: AsyncConnector<T>
}

impl<T: utils::AsyncConnector> AsyncSharedConnector<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite  {
	pub fn new(inner: AsyncConnector<T>) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncConnector> utils::AsyncConnector for AsyncSharedConnector<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite  {
	type Connection = AsyncSharedConnection<T::Connection>;
	//type Future     = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		let f = self.inner.connect();
		Box::pin(async move { f.await.map(AsyncSharedConnection::new) })
	}
}

pub struct AsyncSharedAcceptor<T: utils::AsyncAcceptor> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite  {
	inner: AsyncAcceptor<T>
}

impl<T: utils::AsyncAcceptor> AsyncSharedAcceptor<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite  {
	pub fn new(inner: AsyncAcceptor<T>) -> Self {
		Self { inner }
	}
}

impl<T: utils::AsyncAcceptor> utils::AsyncAcceptor for AsyncSharedAcceptor<T> where T::Connection: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite  {
	type Connection = AsyncSharedConnection<T::Connection>;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		match unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_accept(cx) {
			Poll::Pending  => Poll::Pending,
			Poll::Ready(f) => Poll::Ready(Box::pin(async { f.await.map(AsyncSharedConnection::new) }))
		}
	}
}

pub struct AsyncSharedConnection<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> {
	inner: std::sync::Mutex<AsyncConnection<T>>,
	wait:  concurrent_queue::ConcurrentQueue<std::task::Waker>
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite> AsyncSharedConnection<T> {
	pub fn new(inner: AsyncConnection<T>) -> Self {
		Self { inner: std::sync::Mutex::new(inner), wait: concurrent_queue::ConcurrentQueue::unbounded() }
	}
}

impl<T: futures_lite::io::AsyncBufRead + futures_lite::io::AsyncWrite + Send + Sync + 'static> traits::AsyncSharedConnection for AsyncSharedConnection<T> {
	fn poll_open(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<Result<StreamId>> {
		let Self { inner, wait } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		
		if inner.state != AsyncState::Ready {
			std::mem::drop(wait.push(cx.waker().clone()));
			Poll::Pending
		} else {
			traits::AsyncConnection::poll_open(unsafe { Pin::new_unchecked(&mut*inner) }, cx)
		}
	}
	
	fn poll_opened(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<Result<Option<StreamId>>> {
		let Self { inner, wait } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		
		if inner.state != AsyncState::Ready {
			std::mem::drop(wait.push(cx.waker().clone()));
			Poll::Pending
		} else {
			match traits::AsyncConnection::poll_read(unsafe { Pin::new_unchecked(&mut*inner) }, cx, &mut []) {
				Poll::Pending                             => Poll::Pending,
				Poll::Ready(Ok(Some((id, Read::Opened)))) => Poll::Ready(Ok(Some(id))),
				Poll::Ready(Ok(Some(_)))                  => unreachable!(),
				Poll::Ready(Ok(None))                     => Poll::Ready(Ok(None)),
				Poll::Ready(Err(e))                       => Poll::Ready(Err(e))
			}
		}
	}
	
	fn poll_read<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [u8]) -> Poll<Result<Read<'a>>> {
		let Self { inner, wait } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		match traits::AsyncConnection::poll_read(unsafe { Pin::new_unchecked(&mut*inner) }, cx, buf) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Ok(Some((id2, v)))) if id2 == id => {
				if let Read::Closed = &v {
					while let Ok(waker) = wait.pop() {
						waker.wake();
					}
				}
				
				Poll::Ready(Ok(v))
			}
			Poll::Ready(Ok(Some(_))) => Poll::Ready(Err(io::Error::new(
				io::ErrorKind::Other, "got invalid stream id from inner stream"))),
			Poll::Ready(Ok(None))    => Poll::Ready(Err(io::Error::from(
				io::ErrorKind::ConnectionRefused))),
			Poll::Ready(Err(e))      => Poll::Ready(Err(e))
		}
	}
	
	fn poll_read_vectored<'a>(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &'a mut [&'a mut [u8]]) -> Poll<Result<Read<'a>>> {
		let Self { inner, wait } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		match traits::AsyncConnection::poll_read_vectored(unsafe { Pin::new_unchecked(&mut*inner) }, cx, buf) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Ok(Some((id2, v)))) if id2 == id => {
				if let Read::Closed = &v {
					while let Ok(waker) = wait.pop() {
						waker.wake();
					}
				}
				
				Poll::Ready(Ok(v))
			}
			Poll::Ready(Ok(Some(_))) => Poll::Ready(Err(io::Error::new(
				io::ErrorKind::Other, "got invalid stream id from inner stream"))),
			Poll::Ready(Ok(None))    => Poll::Ready(Err(io::Error::from(
				io::ErrorKind::ConnectionRefused))),
			Poll::Ready(Err(e))      => Poll::Ready(Err(e))
		}
	}
	
	fn poll_write_headers(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, headers: &[Header]) -> Poll<Result<()>> {
		let Self { inner, .. } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		traits::AsyncConnection::poll_write_headers(unsafe { Pin::new_unchecked(&mut*inner) }, cx, id, headers)
	}
	
	fn poll_write_body(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId, buf: &[u8]) -> Poll<Result<()>> {
		let Self { inner, .. } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		traits::AsyncConnection::poll_write_body(unsafe { Pin::new_unchecked(&mut*inner) }, cx, id, buf)
	}
	
	fn poll_flush(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<Result<()>> {
		let Self { inner, .. } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		traits::AsyncConnection::poll_flush(unsafe { Pin::new_unchecked(&mut*inner) }, cx, id)
	}
	
	fn poll_close(self: Pin<&Self>, cx: &mut Context<'_>, id: StreamId) -> Poll<Result<()>> {
		let Self { inner, .. } = unsafe { Pin::into_inner_unchecked(self) };
		let mut inner = inner.try_lock().expect("synchronization error");
		traits::AsyncConnection::poll_close(unsafe { Pin::new_unchecked(&mut*inner) }, cx, id)
	}
}

fn header_error<T>(_: T) -> Error {
	Error::new(ErrorKind::InvalidData, "failed to parse HTTP header")
}

fn to_trimmed_utf8(buf: &[u8]) -> Result<&str> {
	Ok(std::str::from_utf8(buf)
		.map_err(|_| Error::new(ErrorKind::InvalidData, "invalid UTF-8"))?
		.trim())
}

fn parse_status(s: &str) -> Result<Header> {
	if let Some(s) = s.strip_prefix("HTTP/1.1")
		.or_else(|| s.strip_prefix("HTTP/1.0"))
	{
		if let Some((status, _)) = s.trim().split_once(' ') {
			let status = status.trim().parse::<usize>().map_err(header_error)?;
			Ok(Header::Status(super::Status::try_from(status).map_err(header_error)?))
		} else {
			Err(header_error(()))
		}
	} else if let Some((s, _)) = s.split_once(' ') {
		Ok(Header::Path(s.to_string()))
	} else {
		Err(header_error(()))
	}
}

fn read_until_internal<R: AsyncBufRead + ?Sized>(
	mut reader: Pin<&mut R>,
	cx:         &mut Context<'_>,
	byte:       u8,
	buf:        &mut Vec<u8>
) -> Poll<Result<()>> {
	loop {
			let available = match reader.as_mut().poll_fill_buf(cx) {
				Poll::Ready(Ok(t)) => t,
				Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
				Poll::Pending => return Poll::Pending,
			};
			
			let (done, used) = if let Some(i) = memchr::memchr(byte, available) {
				buf.extend_from_slice(&available[..=i]);
				(true, i + 1)
			} else {
				buf.extend_from_slice(available);
				(false, available.len())
			};
		
		reader.as_mut().consume(used);
		
		if done || used == 0 {
			return Poll::Ready(Ok(()));
		}
	}
}

fn write_all_internal(
	mut write: Pin<&mut (impl AsyncWrite + Unpin + ?Sized)>,
	cx:        &mut Context<'_>,
	written:   &mut usize,
	buf:       &[u8]
) -> Poll<Result<()>> {
	while *written < buf.len() {
		match write.as_mut().poll_write(cx, &buf[*written..]) {
			Poll::Pending       => return Poll::Pending,
			Poll::Ready(Ok(0))  => return Poll::Ready(Err(ErrorKind::WriteZero.into())),
			Poll::Ready(Ok(n))  => *written += n,
			Poll::Ready(Err(e)) => return Poll::Ready(Err(e))
		}
	}
	
	Poll::Ready(Ok(()))
}

#[cfg(all(test, feature = "assert_matches"))]
mod tests {
	use crate::http::*;
	
	#[test]
	fn connect() {
		let (client, server) = crate::utils::pipe::Pipe::new_buffered();
		let (mut client, mut server) = (super::Connection::new(client), super::Connection::new(server));
		
		let request_headers = [
			Header::Method(Method::Get),
			Header::Path("/".to_string()),
			Header::UserAgent("cargo test".to_string())
		];
		
		let response_headers = [
			Header::Status(Status::Ok),
			Header::Server("cargo test".to_string())
		];
		
		let mut client_stream = client.stream().unwrap();
		assert_matches!(client_stream.write_headers(&request_headers), Ok(()));
		assert_matches!(client_stream.flush(), Ok(()));
		
		let server_stream = server.accept();
		assert!(matches!(server_stream, Ok(Some(_))));
		let mut server_stream = server_stream.unwrap().unwrap();
		
		let mut headers_read = Vec::new();
		assert_matches!(server_stream.read_all_headers(&mut headers_read), Ok(()));
		assert_matches!(server_stream.read(&mut [0]), Ok(0));
		assert_eq!(headers_read, request_headers);
		
		assert_matches!(server_stream.write_headers(&response_headers), Ok(()));
		assert_matches!(server_stream.write_all(b"test body"), Ok(()));
		assert_matches!(server_stream.flush(), Ok(()));
		
		let mut headers_read = Vec::new();
		assert_matches!(client_stream.read_all_headers(&mut headers_read), Ok(()));
		assert_eq!(headers_read, response_headers);
		
		let mut buf = [0u8; 9];
		assert_matches!(client_stream.read_exact(&mut buf), Ok(()));
		assert_matches!(client_stream.read(&mut [0]), Ok(0));
		assert_eq!(&buf, b"test body");
	}
}