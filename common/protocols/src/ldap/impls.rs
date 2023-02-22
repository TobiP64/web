// MIT License
//
// Copyright (c) 2019-2023  Tobias Pfeiffer
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

use std::future::Future;
use {
	super::{*, ber::*},
	crate::*,
	std::{io, net, pin::Pin, task::{Poll, Context}, sync::Arc}
};

const DEFAULT_BUFFER_LEN: usize = 0x1000;

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
	inner: T,
	buf:   Vec<u8>
}

impl<T: io::Read + io::Write> Connection<T> {
	pub fn new(inner: T) -> Self {
		let mut buf = Vec::with_capacity(DEFAULT_BUFFER_LEN);
		unsafe { buf.set_len(DEFAULT_BUFFER_LEN) };
		Self { inner, buf }
	}
}

impl<T: io::Read + io::Write> traits::Connection for Connection<T> {
	fn send_msg(&mut self, msg: &Message) -> io::Result<()> {
		self.buf.clear();
		write_msg(msg, &mut self.buf);
		self.inner.write_all(&self.buf)
	}
	
	fn recv_msg(&mut self) -> io::Result<Message> {
		self.buf.clear();
		let (_, len) = self.inner.read_ber_to_buf(&mut self.buf)?;
		read_msg(&self.buf[..len])
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
	inner:    T,
	send_buf: Vec<u8>,
	send_off: usize,
	recv_buf: Vec<u8>,
	recv_off: usize
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite> AsyncConnection<T> {
	pub fn new(inner: T) -> Self {
		Self {
			inner,
			send_buf: Vec::with_capacity(DEFAULT_BUFFER_LEN),
			send_off: 0,
			recv_buf: {
				let mut buf = Vec::with_capacity(DEFAULT_BUFFER_LEN);
				unsafe { buf.set_len(2); }
				buf
			},
			recv_off: 0
		}
	}
}

impl<T: futures_lite::io::AsyncRead + futures_lite::io::AsyncWrite + Send> traits::AsyncConnection for AsyncConnection<T> {
	fn poll_send_msg(self: Pin<&mut Self>, cx: &mut Context<'_>, msg: &Message) -> Poll<io::Result<()>> {
		let Self { inner, send_buf, send_off, .. } = unsafe { Pin::into_inner_unchecked(self) };
		poll_send(cx, msg, unsafe { Pin::new_unchecked(inner) }, send_buf, send_off)
	}
	
	fn poll_recv_msg(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Message>> {
		let Self { inner, recv_buf, recv_off, .. } = unsafe { Pin::into_inner_unchecked(self) };
		poll_recv(cx, unsafe { Pin::new_unchecked(inner) }, recv_buf, recv_off)
	}
}

#[cfg(feature = "smol")]
pub struct SharedClientConnection<T: traits::AsyncConnection>(crate::utils::Scheduler<T, MessageId, Message, Message, io::Error>);

#[cfg(feature = "smol")]
impl<T: traits::AsyncConnection> SharedClientConnection<T> {
	pub fn new(inner: T) -> Self {
		Self(crate::utils::Scheduler::new(inner))
	}
}

#[cfg(feature = "smol")]
impl<T: traits::AsyncConnection + Send + Sync> traits::AsyncSharedClientConnection for SharedClientConnection<T> {
	fn send<'a>(self: Pin<&'a Self>, msg: &'a Message) -> Pin<Box<dyn Future<Output = io::Result<Message>> + Send + 'a>> {
		Box::pin(async move { unsafe { Pin::into_inner_unchecked(self) }.0
			.dispatch(msg.message_id, msg.clone(), true).await
			.transpose()
			.expect("expecting a reply") })
	}
}

fn poll_send<T: futures_lite::io::AsyncWrite>(
	cx:    &mut Context<'_>,
	msg:   &Message,
	inner: Pin<&mut T>,
	buf:   &mut Vec<u8>,
	off:   &mut usize
) -> Poll<io::Result<()>> {
	if buf.is_empty() {
		write_msg(msg, buf);
	}
	
	loop {
		match unsafe { std::mem::transmute_copy::<_, Pin<&mut T>>(&inner) }.poll_write(cx, &buf[*off..]) {
			Poll::Pending => return Poll::Pending,
			Poll::Ready(Ok(n)) => {
				*off += n;
				
				if *off == buf.len() {
					buf.clear();
					*off = 0;
					return Poll::Ready(Ok(()));
				}
			}
			Poll::Ready(Err(e)) => {
				buf.clear();
				*off = 0;
				return Poll::Ready(Err(e));
			}
		}
	}
}

#[allow(clippy::bad_bit_mask)]
fn poll_recv<T: futures_lite::io::AsyncRead>(
	cx:    &mut Context<'_>,
	inner: Pin<&mut T>,
	buf:   &mut Vec<u8>,
	off:   &mut usize
) -> Poll<io::Result<Message>> {
	let inner = unsafe { Pin::into_inner_unchecked(inner) };
	
	loop {
		match unsafe { Pin::new_unchecked(&mut*inner) }.poll_read(cx, &mut buf[*off..]) {
			Poll::Pending       => return Poll::Pending,
			Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
			Poll::Ready(Ok(n))  => {
				*off += n;
				match *off {
					0..=2 => {
						if *off == buf.len() {
							let len = buf[1] & 0x7F;
							
							if buf[1] & 0x80 == 1 && len > 8 {
								return Poll::Ready(Err(io::Error::new(
									io::ErrorKind::InvalidData, "integer too long")))
							}
							
							buf.reserve(len as usize);
							unsafe { buf.set_len(buf.len() + len as usize); }
						}
					}
					v if buf[1] & 0x80 == 1 && buf.len() <= (buf[1] & 0x7F) as usize => {
						if v == buf.len() {
							let mut bytes = [0u8; 8];
							bytes[10 - buf.len()..].copy_from_slice(&buf[2..]);
							let len = u64::from_be_bytes(bytes) as _;
							buf.reserve(len);
							unsafe { buf.set_len(buf.len() + len); }
						}
					}
					v if v == buf.len() => {
						let data_off = match  buf[1] & 0x80 {
							0 => 2,
							_ => 2 + (buf[1] & 0x7F)
						};
						
						let msg = read_msg(&buf[data_off as usize..]);
						*off = 0;
						unsafe { buf.set_len(2); }
						return Poll::Ready(msg);
					}
					_ => ()
				}
			}
		}
	}
}