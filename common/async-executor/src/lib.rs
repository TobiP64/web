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

#![warn(clippy::all)]
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "num_cpus"), feature(available_parallelism))]
#![cfg_attr(not(feature = "conquer-once"), feature(once_cell))]

use std::{
	sync::{Mutex, Condvar, Arc, TryLockError},
	collections::VecDeque,
	task::{Wake, Waker},
	pin::Pin,
	future::Future,
	thread::JoinHandle
};

#[cfg(not(feature = "conquer-once"))]
const DEFAULT_WORKER_COUNT: usize = 2;
const NO_EXECUTOR:          &str  = "executor was not initialized";
#[cfg(not(feature = "conquer-once"))]
static EXECUTOR: std::lazy::SyncOnceCell<Executor> = std::lazy::SyncOnceCell::new();
#[cfg(feature = "conquer-once")]
static EXECUTOR: conquer_once::OnceCell<Executor> = conquer_once::OnceCell::uninit();

pub fn run<F: Future<Output = ()> + Send + 'static>(threads: Option<usize>, main: fn() -> F) {
	EXECUTOR.get_or_init(Executor::default);
	spawn(main());
	
	(0..threads.unwrap_or_else(|| {
		#[cfg(feature = "num_cpus")]
			{
				num_cpus::get()
			}
		
		#[cfg(not(feature = "num_cpus"))]
			{
				std::thread::available_parallelism()
					.map_or(DEFAULT_WORKER_COUNT, |v| v.get())
			}
	}))
		.map(|i| std::thread::Builder::new()
			.name(format!("worker-{}", i))
			.spawn(worker))
		.collect::<std::result::Result<Vec<_>, _>>()
		.expect("failed to spawn worker threads")
		.into_iter()
		.try_for_each(JoinHandle::join)
		.expect("failed to join worker threads");
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
	spawn_dyn(Box::pin(future));
}

pub fn spawn_dyn(future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
	Arc::new(Task(Mutex::new(future))).wake();
}

#[derive(Default)]
struct Executor {
	queue: Mutex<VecDeque<Arc<Task>>>,
	wait:  Condvar
}

pub fn worker() {
	let executor = EXECUTOR.get().expect(NO_EXECUTOR);
	
	loop {
		let mut queue = executor.queue.lock().expect("failed to lock queue");
		let task = loop {
			match queue.pop_front() {
				Some(task) => break task,
				None => queue = executor.wait.wait(queue).expect("failed to lock queue")
			}
		};
		
		std::mem::drop(queue);
		
		let r = std::panic::catch_unwind(|| {
			match task.0.try_lock() {
				Ok(mut v) => return Ok(v.as_mut().poll(&mut std::task::Context::from_waker(
					&Waker::from(task.clone())))),
				Err(TryLockError::WouldBlock)  => log::warn!("task already running"),
				Err(TryLockError::Poisoned(_)) => log::warn!("task poisoned")
			}
			
			Err(())
		});
		
		match r {
			Err(e) => match e.downcast_ref::<String>() {
				Some(s) => log::error!("task panicked: {}", s),
				None    => log::error!("task panicked")
			},
			Ok(Err(_)) => (),
			Ok(_) => ()
		}
	}
}

struct Task(Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>);

impl Wake for Task {
	fn wake(self: Arc<Self>) {
		let executor = EXECUTOR.get().expect(NO_EXECUTOR);
		executor.queue.lock().expect("failed to lock queue").push_back(self);
		executor.wait.notify_one();
	}
}