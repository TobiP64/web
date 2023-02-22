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

#[cfg(feature = "smol")]
use {std::{future::Future, pin::Pin, task::{Context, Poll}}, futures_lite::Stream};

pub fn _82223_into_ok_or_err<T>(v: Result<T, T>) -> T {
	match v {
		Ok(v) => v,
		Err(v) => v,
	}
}

pub fn _88373_into_incoming(self_: std::net::TcpListener) -> IntoIncoming {
	IntoIncoming { listener: self_ }
}

#[cfg(feature = "smol")]
pub fn _88373_into_incoming_async(self_: smol::net::TcpListener) -> IntoIncomingAsync {
	IntoIncomingAsync { listener: self_, state: None }
}

#[derive(Debug)]
pub struct IntoIncoming {
	listener: std::net::TcpListener,
}

impl Iterator for IntoIncoming {
	type Item = std::io::Result<std::net::TcpStream>;
	
	fn next(&mut self) -> Option<std::io::Result<std::net::TcpStream>> {
		Some(self.listener.accept().map(|p| p.0))
	}
}

#[cfg(feature = "smol")]
type State = Pin<Box<dyn Future<Output = std::io::Result<(smol::net::TcpStream, std::net::SocketAddr)>>>>;

#[cfg(feature = "smol")]
pub struct IntoIncomingAsync {
	listener: smol::net::TcpListener,
	state:    Option<State>
}

#[cfg(feature = "smol")]
impl Stream for IntoIncomingAsync {
	type Item = std::io::Result<smol::net::TcpStream>;
	
	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let self_ = unsafe { self.get_unchecked_mut() };
		
		match self_.state
			.get_or_insert_with(|| unsafe { std::mem::transmute::<Pin<Box<dyn Future<Output = std::io::Result<(smol::net::TcpStream, std::net::SocketAddr)>>>>, _>(Box::pin(
				async { self_.listener.accept().await })) })
			.as_mut()
			.poll(cx)
		{
			Poll::Pending  => Poll::Pending,
			Poll::Ready(v) => Poll::Ready(Some(v.map(|v| v.0)))
		}
	}
}