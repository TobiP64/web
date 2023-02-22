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

use std::{io, net};

#[cfg(feature = "smol")]
use {
	std::{pin::Pin, task::{Context, Poll}, future::Future, sync::Arc},
	futures_lite::FutureExt
};

pub struct Connector<T: net::ToSocketAddrs> {
	addr: T
}

impl<T: net::ToSocketAddrs> Connector<T> {
	pub fn new(addr: T) -> Self {
		Self { addr }
	}
}

impl<T: net::ToSocketAddrs> crate::utils::Connector for Connector<T> {
	type Connection = net::TcpStream;
	
	fn connect(&self) -> io::Result<Self::Connection> {
		net::TcpStream::connect(&self.addr)
	}
}

pub struct Acceptor {
	inner: net::TcpListener
}

impl Acceptor {
	pub fn new(addr: impl net::ToSocketAddrs) -> io::Result<Self> {
		Ok(Self { inner: net::TcpListener::bind(addr)? })
	}
}

impl crate::utils::Acceptor for Acceptor {
	type Connection = net::TcpStream;
	
	fn accept(&mut self) -> io::Result<Self::Connection> {
		self.inner.accept().map(|(v, _)| v)
	}
}

#[cfg(feature = "smol")]
pub struct AsyncConnector<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> {
	addr: Arc<T>
}

#[cfg(feature = "smol")]
impl<T: smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> AsyncConnector<T> {
	pub fn new(addr: T) -> Self {
		Self { addr: Arc::new(addr) }
	}
}

#[cfg(feature = "smol")]
impl<T: 'static + smol::net::AsyncToSocketAddrs<Iter: Send> + Send + Sync> crate::utils::AsyncConnector for AsyncConnector<T> {
	type Connection = smol::net::TcpStream;
	//type Future     = Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>;
	
	fn connect(&self) -> Pin<Box<dyn std::future::Future<Output = io::Result<Self::Connection>> + Send + 'static>> {
		let addr = self.addr.clone();
		Box::pin(async move { smol::net::TcpStream::connect(&*addr).await })
	}
}

#[cfg(feature = "smol")]
pub struct AsyncAcceptor {
	inner: smol::net::TcpListener,
	state: Option<smol::future::Boxed<io::Result<(smol::net::TcpStream, net::SocketAddr)>>>
}

#[cfg(feature = "smol")]
impl AsyncAcceptor {
	pub async fn new(addr: impl smol::net::AsyncToSocketAddrs) -> io::Result<Self> {
		Ok(Self { inner: smol::net::TcpListener::bind(addr).await?, state: None })
	}
}

#[cfg(feature = "smol")]
impl crate::utils::AsyncAcceptor for AsyncAcceptor {
	type Connection = smol::net::TcpStream;
	
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>> {
		let Self { inner, state } = unsafe { Pin::into_inner_unchecked(std::mem::transmute::<_, Pin<&'static mut Self>>(self)) };
		match state.get_or_insert_with(|| Box::pin(inner.accept())).poll(cx) {
			Poll::Pending           => Poll::Pending,
			Poll::Ready(Ok((v, _))) => {
				*state = None;
				Poll::Ready(Box::pin(async move { Ok(v) }))
			},
			Poll::Ready(Err(e))     => {
				*state = None;
				Poll::Ready(Box::pin(async move { Err(e) }))
			}
		}
	}
}