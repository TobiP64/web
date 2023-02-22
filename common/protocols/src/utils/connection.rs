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

use std::{io, pin::Pin, future::Future, task::{Context, Poll}};

pub trait Connector {
	type Connection;

	fn connect(&self) -> io::Result<Self::Connection>;
}

pub trait Acceptor {
	type Connection;

	fn accept(&mut self) -> io::Result<Self::Connection>;
}

pub struct Incoming<'a, T: Acceptor>(&'a mut T);

impl<'a, T: Acceptor> Iterator for Incoming<'a, T> {
	type Item = io::Result<T::Connection>;

	fn next(&mut self) -> Option<Self::Item> {
		Some(self.0.accept())
	}
}

pub struct IntoIncoming<T: Acceptor>(T);

impl<T: Acceptor> Iterator for IntoIncoming<T> {
	type Item = io::Result<T::Connection>;

	fn next(&mut self) -> Option<Self::Item> {
		Some(self.0.accept())
	}
}

pub trait AsyncConnector: Send + Sync {
	type Connection: 'static + Send;
	//type Future: DerefMut<Target: Future<Output = io::Result<Self::Connection>> + Send + 'static>;

	//fn connect(&self) -> Pin<Self::Future>;
	fn connect<'a>(&'a self) -> Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'a>>;
}

impl<C: 'static + Send, /*, F: DerefMut<Target: Future<Output = io::Result<C>> + Send + 'static>*/> AsyncConnector for Pin<Box<dyn AsyncConnector<Connection = C/*, Future = F*/>>> {
	type Connection = C;
	//type Future     = F;

	fn connect<'a>(&'a self) -> Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'a>> {
		(**self).connect()
	}
}

pub type BoxedAsyncConnector<T> = Pin<Box<dyn AsyncConnector<Connection = Pin<Box<T>>/*, Future = Box<dyn Future<Output = io::Result<Pin<Box<T>>>>>*/>>>;

pub trait AsyncAcceptor: Send {
	type Connection: 'static + Send;
	//type Future: DerefMut<Target: Future<Output = io::Result<Self::Connection>> + Send + 'static>;

	//fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Future>;
	fn poll_accept(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Pin<Box<dyn Future<Output = io::Result<Self::Connection>> + Send + 'static>>>;
}

pub type BoxedAsyncAcceptor<T> = Pin<Box<dyn AsyncAcceptor<Connection = Pin<Box<T>>>>>;

pub trait AsyncAcceptorExt: AsyncAcceptor {
	fn accept(&mut self) -> AsyncAcceptorAccept<Self> {
		AsyncAcceptorAccept(self)
	}

	fn incoming(&mut self) -> AsyncAcceptorIncoming<Self> {
		AsyncAcceptorIncoming(self)
	}

	/*fn into_incoming(self) -> AsyncAcceptorIntoIncoming<Self> where Self: Sized {
		AsyncAcceptorIntoIncoming(self)
	}*/
}

impl<T: AsyncAcceptor> AsyncAcceptorExt for T {}

pub struct AsyncAcceptorAccept<'a, T: AsyncAcceptor + ?Sized>(&'a mut T);

impl<'a, T: AsyncAcceptor + ?Sized> Future for AsyncAcceptorAccept<'a, T> {
	type Output = Pin<Box<dyn Future<Output = io::Result<T::Connection>> + Send>>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut**v) }.poll_accept(cx)
	}
}

pub struct AsyncAcceptorIncoming<'a, T: AsyncAcceptor + ?Sized>(&'a mut T);

impl<'a, T: AsyncAcceptor + ?Sized> futures_lite::Stream for AsyncAcceptorIncoming<'a, T> {
	type Item = Pin<Box<dyn Future<Output = io::Result<T::Connection>> + Send>>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		unsafe { self.map_unchecked_mut(|Self(v)| &mut**v) }.poll_accept(cx).map(Some)
	}
}

/*pub struct AsyncAcceptorIntoIncoming<T: AsyncAcceptor>(T);

impl<T: AsyncAcceptor> futures_lite::Stream for AsyncAcceptorIntoIncoming<T> {
	type Item = Pin<Box<dyn Future<Output = io::Result<T::Connection>> + 'static + Send>>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		unsafe { self.map_unchecked_mut(|Self(v)| v) }.poll_accept(cx).map(Some)
	}
}*/