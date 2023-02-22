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
//unwrap()
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

use std::{
	cell::UnsafeCell,
	collections::LinkedList,
	sync::Arc,
	pin::Pin,
	task::{Poll, Waker, Context}
};

pub trait Stream<T, I, O, E> {
	fn poll_send(self: Pin<&mut Self>, cx: &mut Context<'_>, token: T, msg: &I) -> Poll<Result<(), E>>;
	
	fn poll_recv(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(T, O), E>>;
}

/// Error that is emitted when an synchronization error occurs (i.e. invalid state, lock poisoning)
pub struct SyncError(pub &'static str);

#[allow(clippy::from_over_into)]
impl Into<std::io::Error> for SyncError {
	fn into(self) -> std::io::Error {
		std::io::Error::new(std::io::ErrorKind::Other, self.0)
	}
}

#[derive(Debug)]
enum RequestState<I, O, E> {
	NoReplyRequestActive(I),
	NoReplyRequestWake(I, Waker),
	RequestActive(I),
	RequestWake(I, Waker),
	PendingActive,
	PendingWake(Waker),
	ResultOkNoReply,
	ResultOk(O),
	ResultErr(E)
}

#[allow(clippy::type_complexity)]
#[derive(Debug)]
struct SchedulerQueue<T, I, O, E> {
	wake:  Option<Waker>,
	queue: LinkedList<(T, Arc<__UnsafeCell__<RequestState<I, O, E>>>)>
}

#[derive(Debug)]
pub struct Scheduler<S: Stream<T, I, O, E>, T: Eq + Copy + std::fmt::Display, I, O, E> where SyncError: Into<E> {
	stream: __UnsafeCell__<S>,
	queue:  smol::lock::Mutex<SchedulerQueue<T, I, O, E>>
}

impl<S: Stream<T, I, O, E>, T: Eq + Copy + std::fmt::Display, I, O, E> Scheduler<S, T, I, O, E> where SyncError: Into<E> {
	pub fn new(stream: S) -> Self {
		Self {
			queue:   smol::lock::Mutex::new(SchedulerQueue {
				wake:  None,
				queue: LinkedList::new()
			}),
			stream: __UnsafeCell__::new(stream),
		}
	}
	
	pub async fn set_stream(&self, stream: S) {
		// spin until the queue is empty, so it is safe to replace the stream
		loop {
			let queue = self.queue.lock().await;
			
			if queue.queue.is_empty() {
				unsafe { *self.stream.get() = stream; }
				return;
			}
		}
	}
	
	pub async fn dispatch(&self, token: T, request: I, reply: bool) -> Result<Option<O>, E> {
		use RequestState::*;
		
		let mut queue = self.queue.lock().await;
		
		let result = if queue.queue.is_empty() {
			queue.queue.push_back((token, Arc::new(__UnsafeCell__::new(match reply {
				true  => RequestActive(request),
				false => NoReplyRequestActive(request)
			}))));
			
			self.run(queue).await
		} else {
			let mut request = Some(request);
			let state = smol::future::poll_fn(|cx| Poll::Ready(match reply {
				true  => RequestWake(request.take().unwrap(), cx.waker().clone()),
				false => NoReplyRequestWake(request.take().unwrap(), cx.waker().clone())
			})).await;
			
			let state = Arc::new(__UnsafeCell__::new(state));
			queue.queue.push_back((token, state.clone()));
			
			if let Some(waker) = queue.wake.take() {
				waker.wake();
			}
			
			std::mem::drop(queue);
			let mut ready = false;
			smol::future::poll_fn(|_| if ready {
				Poll::Ready(())
			} else {
				ready = true;
				Poll::Pending
			}).await;
			
			loop {
				match unsafe { __UnsafeCell_get_deref__(&*state) } {
					NoReplyRequestActive(_) | RequestActive(_) | PendingActive => break self.run(self.queue.lock().await).await,
					NoReplyRequestWake(..) | PendingWake(..) | RequestWake(..) => {
						ready = false;
						smol::future::poll_fn(|_| if ready {
							Poll::Ready(())
						} else {
							ready = true;
							Poll::Pending
						}).await
					}
					_ => break Arc::try_unwrap(state)
						.map_err(|_| SyncError("failed to lock unwrap state").into())?
						.into_inner()
				}
			}
		};
		
		match result {
			ResultOkNoReply => Ok(None),
			ResultOk(buf) => Ok(Some(buf)),
			ResultErr(e)  => Err(e),
			_             => Err(SyncError("invalid query state after wakeup").into())
		}
	}
	
	async fn run<'a>(&'a self, mut queue: smol::lock::MutexGuard<'a, SchedulerQueue<T, I, O, E>>) -> RequestState<I, O, E> {
		use RequestState::*;
		
		enum StreamResult<T, O, E> {
			Send,
			Recv(Result<(T, O), E>)
		}
		
		debug_assert!(!queue.queue.is_empty(), "cannot acquire stream when queue is empty");
		let stream = unsafe { Pin::new_unchecked(__UnsafeCell_get_deref_mut__(&self.stream)) };
		let mut result = StreamResult::Send;
		
		let (queue, state_) = 'main: loop {
			match result {
				StreamResult::Recv(Ok((recv_token, buf))) => {
					let mut cursor = queue.queue.cursor_front_mut();
					
					loop {
						match cursor.current() {
							None => break,
							Some((token, _)) if *token == recv_token => break,
							Some(_) => cursor.move_next()
						}
					}
					
					match cursor.remove_current() {
						Some((_, state)) => match unsafe { &mut*state.get() } {
							PendingWake(waker) => {
								let waker = unsafe { std::ptr::read(waker) };
								unsafe { std::ptr::write(state.get(), ResultOk(buf)) }
								waker.wake();
							}
							PendingActive => break (queue, ResultOk(buf)),
							_ => log::warn!("request #{}: received response for already completed query", recv_token)
						}
						None => log::warn!("request #{}: received response for unregistered query", recv_token)
					}
				}
				StreamResult::Recv(Err(e)) => break (queue, ResultErr(e)),
				StreamResult::Send => {
					let mut cursor = queue.queue.cursor_front_mut();
					
					while let Some((token, state)) = cursor.current() {
						let token = *token;
						
						match unsafe { __UnsafeCell_get_deref_mut__(&*state) } {
							RequestActive(buf) => match futures_lite::future::poll_fn(|cx|
								unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }
									.poll_send(cx, token, buf)).await
							{
								Ok(()) => {
									std::mem::drop(unsafe { std::ptr::read(buf) });
									unsafe { std::ptr::write(state.get(), PendingActive) }
								}
								Err(e) => {
									cursor.remove_current();
									break 'main (queue, ResultErr(e))
								}
							}
							NoReplyRequestActive(buf) => {
								let new_state = match futures_lite::future::poll_fn(|cx|
									unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }
										.poll_send(cx, token, buf)).await
								{
									Ok(()) => ResultOkNoReply,
									Err(e) => ResultErr(e)
								};
								
								cursor.remove_current();
								break 'main (queue, new_state);
							}
							RequestWake(buf, waker) => match futures_lite::future::poll_fn(|cx|
								unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }
									.poll_send(cx, token, buf)).await
							{
								Ok(()) => {
									let waker = unsafe { std::ptr::read(waker) };
									unsafe { std::ptr::write(state.get(), PendingWake(waker)) }
								}
								Err(e) => {
									let waker = unsafe { std::ptr::read(waker) };
									unsafe { std::ptr::write(state.get(), ResultErr(e)) }
									cursor.remove_current();
									waker.wake();
								}
							}
							NoReplyRequestWake(buf, waker) => {
								let new_state = match futures_lite::future::poll_fn(|cx|
									unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }
										.poll_send(cx, token, buf)).await
								{
									Ok(()) => ResultOkNoReply,
									Err(e) => ResultErr(e)
								};
								
								let waker = unsafe { std::ptr::read(waker) };
								unsafe { std::ptr::write(state.get(), new_state) }
								cursor.remove_current();
								waker.wake();
							}
							_ => ()
						}
						
						cursor.move_next();
					}
				}
			}
			
			let mut future_state = Some(queue);
			
			result = smol::future::poll_fn(|cx| {
				if let Some(mut queue) = future_state.take() {
					// first call, set waker and return
					queue.wake = Some(cx.waker().clone());
					unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }.poll_recv(cx).map(StreamResult::Recv)
				} else {
					// call after wake up, check if it was caused by the stream or a newly created
					// query
					match unsafe { std::mem::transmute_copy::<_, Pin<&mut S>>(&stream) }.poll_recv(cx) {
						Poll::Ready(v) => Poll::Ready(StreamResult::Recv(v)),
						Poll::Pending  => Poll::Ready(StreamResult::Send)
					}
				}
			}).await;
			
			queue = self.queue.lock().await;
		};
		
		for (_, state) in queue.queue.iter() {
			let state = unsafe { &mut*state.get() };
			
			match state {
				NoReplyRequestWake(buf, waker) => {
					let buf = unsafe { std::ptr::read(buf) };
					let waker = unsafe { std::ptr::read(waker) };
					unsafe { std::ptr::write(state, NoReplyRequestActive(buf)) };
					waker.wake();
				}
				RequestWake(buf, waker) => {
					let buf = unsafe { std::ptr::read(buf) };
					let waker = unsafe { std::ptr::read(waker) };
					unsafe { std::ptr::write(state, RequestActive(buf)) };
					waker.wake();
				}
				PendingWake(waker) => {
					let waker = unsafe { std::ptr::read(waker) };
					unsafe { std::ptr::write(state, PendingActive) };
					waker.wake();
				}
				_ => continue
			}
			
			return state_;
		}
		
		state_
	}
}

#[derive(Debug)]
struct __UnsafeCell__<T>(UnsafeCell<T>);

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for __UnsafeCell__<T> {}
unsafe impl<T> Sync for __UnsafeCell__<T> {}

impl<T> __UnsafeCell__<T> {
	fn new(v: T) -> Self {
		Self(UnsafeCell::new(v))
	}
	
	fn into_inner(self) -> T {
		self.0.into_inner()
	}
}

impl<T> std::ops::Deref for __UnsafeCell__<T> {
	type Target = UnsafeCell<T>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> std::ops::DerefMut for __UnsafeCell__<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[allow(non_snake_case)]
unsafe fn __UnsafeCell_get_deref__<'a, T>(v: &__UnsafeCell__<T>) -> &'a T {
	&*v.get()
}

#[allow(non_snake_case)]
unsafe fn __UnsafeCell_get_deref_mut__<'a, T>(v: &__UnsafeCell__<T>) -> &'a mut T {
	&mut*v.get()
}