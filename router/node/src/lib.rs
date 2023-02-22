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

//! Library for net-services plugins, each module must have an entry point that looks like this:
//!
//! ```
//! #[no_mangle]
//! pub extern "Rust" fn main<'a>(
//!     cfg: &'a mut (dyn net_services::dyn_serde::Deserializer + Send + Sync)
//! ) -> net_services::DynFuture<'a, net_services::Result<()>> {
//!     Box::pin(async move {
//!         // ...
//!         Ok(())
//!     })
//! }
//! ```

use std::{future::Future, pin::Pin, sync::Arc, any::Any};

pub use {
	log,
	dyn_error::*,
	erased_serde as dyn_serde,
	otel_mrt as otel,
	kranus_protocols as net
};

macro_rules! plugin {
    ( ( $ident:ident: $ty:ty as $( $interface:ty )+ ),* ) => {
		#[no_mangle]
		pub extern "Rust" fn main<'a>(cfg: &'a mut (dyn $crate::dyn_serde::Deserializer + Send + Sync)) -> $crate::DynFuture<'a, $crate::Result<()>> {
			Box::pin(async move {
				(
					{
						let component = $ty::new(&mut*cfg).await?;
						let id = net_services::component_id(stringify!($ident));

						$(
							$crate::add_component(id, Arc::new(Box::new(component) as $interface)).await;
						)*
					}
				)*

				Ok(())
			})
		}
	};
}

//#![feature(const_type_id)]
//#[no_mangle]
//pub static CHECK_ID: std::any::TypeId = std::any::TypeId::of::<()>();

#[no_mangle]
pub static NET_SERVICES_PLUGIN_INFO_STR: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS"));

#[no_mangle]
pub static NET_SERVICES_PLUGIN_INFO: &&str = &NET_SERVICES_PLUGIN_INFO_STR;

//#![feature(once_cell)]
//static CONTEXT: std::lazy::SyncOnceCell<Arc<dyn Context>> = std::lazy::SyncOnceCell::new();
static CONTEXT_DATA:   std::sync::atomic::AtomicPtr<()> = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
static CONTEXT_VTABLE: std::sync::atomic::AtomicPtr<()> = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct ComponentRef<T>(Arc<std::sync::atomic::AtomicPtr<T>>);

impl<T> std::ops::Deref for ComponentRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let ptr = self.0.load(std::sync::atomic::Ordering::SeqCst);

		if ptr.is_null() {
			panic!("component not present");
		}

		unsafe { &*ptr }
	}
}

impl<T> Clone for ComponentRef<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

// can't use the std version because it's unstable
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TraitObject {
	pub data:   *mut (),
	pub vtable: *mut (),
}

extern "Rust" {
	fn main<'a>(cfg: &'a (dyn erased_serde::Deserializer + Send + Sync)) -> DynFuture<'a, Result<()>>;
}

#[no_mangle]
pub extern "Rust" fn net_services_plugin_init<'a>(
	log: &'static dyn log::Log,
	lvl: log::LevelFilter,
	ctx: Arc<dyn Context>,
	trt: otel::Runtime,
	cfg: &'a (dyn dyn_serde::Deserializer + Send + Sync)
) -> DynFuture<'a, Result<()>> {
	log::set_logger(log).unwrap();
	log::set_max_level(lvl);
	//CONTEXT.set(ctx);
	set_context(ctx);
	unsafe { otel_mrt::set_global(trt) };
	log::debug!("init: plugin `{}` context initialized", env!("CARGO_PKG_NAME"));
	unsafe { main(cfg) }
}

pub fn set_context(ctx: Arc<dyn Context>) {
	let object = unsafe { std::mem::transmute::<_, TraitObject>(Arc::as_ptr(&ctx)) };
	std::mem::forget(ctx); // SAFE: the context lives as long as the module is loaded
	CONTEXT_DATA.store(object.data, std::sync::atomic::Ordering::Relaxed);
	CONTEXT_VTABLE.store(object.vtable, std::sync::atomic::Ordering::Relaxed);
}

fn context() -> &'static dyn Context {
	//CONTEXT.get().expect("context was not initialized")

	unsafe { std::mem::transmute(TraitObject {
		data:   CONTEXT_DATA.load(std::sync::atomic::Ordering::Relaxed),
		vtable: CONTEXT_VTABLE.load(std::sync::atomic::Ordering::Relaxed)
	}) }
}

pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
	context().spawn_dyn(Box::pin(f))
}

pub fn component_id(name: &str) -> u128 {
	context().component_id(name)
}

pub fn get_component<T: Any + Send + Sync>(id: u128) -> ComponentRef<T> {
	let ptr = context().component_dyn_get(id, get_interface_id::<T>());
	ComponentRef(unsafe { std::mem::transmute(ptr) })
}

pub fn add_component<T: Any + Send + Sync>(id: u128, interface: T) {
	let v = Box::leak(Box::new(interface));
	context().component_dyn_add(id, get_interface_id::<T>(), v as *mut T as _);
}

pub fn del_component<T: Any + Send + Sync>(id: u128) -> bool {
	context().component_dyn_del(id, get_interface_id::<T>())
}

fn get_interface_id<T: ?Sized>() -> u64 {
	use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
	let mut hasher = DefaultHasher::new();
	std::any::type_name::<T>().hash(&mut hasher);
	hasher.finish()
}

pub trait Context: Send + Sync + 'static + std::fmt::Debug + std::fmt::Display {
	fn spawn_dyn(&self, f: DynFuture<'static, ()>);

	fn component_id(&self, name: &str) -> u128;

	fn component_dyn_get(&self, id: u128, interface: u64) -> Arc<std::sync::atomic::AtomicPtr<()>>;

	fn component_dyn_add(&self, id: u128, interface: u64, ptr: *mut ());

	fn component_dyn_del(&self, id: u128, interface: u64) -> bool;
}

pub struct ErrorWithMessage<E: std::fmt::Display>(&'static str, E);

impl<E: std::fmt::Display> std::fmt::Display for ErrorWithMessage<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.0)?;
		f.write_str(": ")?;
		self.1.fmt(f)
	}
}

pub trait WithMessage<T, E: std::fmt::Display> {
	fn with_msg(self, msg: &'static str) -> std::result::Result<T, ErrorWithMessage<E>>;
}

impl<T, E: std::fmt::Display> WithMessage<T, E> for std::result::Result<T, E> {
	fn with_msg(self, msg: &'static str) -> std::result::Result<T, ErrorWithMessage<E>> {
		self.map_err(|e| ErrorWithMessage(msg, e))
	}
}

impl<T> WithMessage<T, &'static str> for std::result::Result<T, ()> {
	fn with_msg(self, msg: &'static str) -> std::result::Result<T, ErrorWithMessage<&'static str>> {
		self.map_err(|_| ErrorWithMessage(msg, ""))
	}
}

impl<T> WithMessage<T, String> for std::option::Option<T> {
	fn with_msg(self, msg: &'static str) -> std::result::Result<T, ErrorWithMessage<String>> {
		self.ok_or(format!("{} was {:?}", std::any::type_name::<Self>(), None::<()>)).with_msg(msg)
	}
}

/// Traits and types for interfacing with built-in modules
pub mod interfaces {
	use {super::*, net::http, std::any::TypeId};

	pub trait AsyncByteStream: smol::io::AsyncRead + smol::io::AsyncWrite + Send {}

	impl<T: smol::io::AsyncRead + smol::io::AsyncWrite + Send> AsyncByteStream for T {}

	pub trait GenericStream {
		fn has(&self, ty: TypeId) -> bool;

		fn downcast_ref(&self, ty: TypeId) -> Option<*const ()>;

		fn downcast_mut(&mut self, ty: TypeId) -> Option<*mut ()>;
	}

	pub trait GenericStreamExt {
		fn has<T: 'static>(&self) -> bool;

		fn downcast_ref<T: 'static>(&self) -> Option<&T>;

		fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T>;
	}

	impl<S: GenericStream> GenericStreamExt for S {
		fn has<T: 'static>(&self) -> bool {
			GenericStream::has(self, TypeId::of::<T>())
		}

		fn downcast_ref<T: 'static>(&self) -> Option<&T> {
			GenericStream::downcast_ref(self, TypeId::of::<T>())
				.map(|ptr| unsafe { &* (ptr as *const T) })
		}

		fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
			GenericStream::downcast_mut(self, TypeId::of::<T>())
				.map(|ptr| unsafe { &mut *(ptr as *mut T) })
		}
	}

	pub trait StreamHandler<T: ?Sized>: Send + Sync {
		/// The compiler decided that `stream` has to be `'static` at all costs.
		/// Due to that, `stream` must be used as if it had the `'a` lifetime,
		/// although it is `'static`. Not doing so is undefined behaviour.
		fn accept<'a>(&'a self, stream: &'static mut T) -> DynFuture<'a, Result<()>>;
	}

	pub trait StreamFilter<T: ?Sized>: Send + Sync {
		fn filter<'a>(&'a self, stream: &'static mut T) -> DynFuture<'a, Result<bool>>;
	}

	pub type GenericStreamHandler = Box<dyn StreamHandler<dyn GenericStream>>;
	pub type HttpStreamHandler = Box<dyn StreamHandler<dyn http::traits::AsyncStream>>;
	pub type ByteStreamHandler = Box<dyn StreamHandler<dyn AsyncByteStream>>;
}