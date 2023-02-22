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

use {
	crate::*,
	std::{
		any::Any,
		sync::{Arc, atomic::AtomicPtr},
		collections::{BTreeMap, btree_map::Entry},
		path::PathBuf
	},
	smol::lock::{Mutex, RwLock}
};

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(5);

pub enum File {
	Config(ConfigFile),
	Plugin(PluginFile),
	#[cfg(feature = "wasm-runtime")]
	Wasm(WasmFile)
}

pub struct ConfigFile {
	pub value:    HashMap<String, serde_dyn_repr::Value>,
	pub includes: Vec<Arc<File>>
}

pub struct PluginFile {
	pub info:   String,
	pub plugin: libloading::Library
}

#[cfg(feature = "wasm-runtime")]
pub struct WasmFile {
	pub info:   String,
	pub plugin: wasmer_runtime::Instance
}

pub enum Plugin {
	SharedLib(Arc<PluginFile>),
	#[cfg(feature = "wasm-runtime")]
	Wasm(Arc<WasmFile>)
}

#[allow(clippy::type_complexity)]
pub(crate)  struct ContextImpl {
	pub(crate) config:       HashMap<String, serde_dyn_repr::Value>,
	pub(crate) files:        HashMap<PathBuf, Arc<RwLock<File>>>,
	pub(crate) changes_file: Arc<PathBuf>,
	pub(crate) components:   BTreeMap<(u128, u64), Arc<AtomicPtr<()>>>
}

pub enum Component {
	Present(Arc<dyn Any + Send + Sync>),
	Waker(Vec<std::task::Waker>)
}

pub struct ContextWrapper(pub(crate) Mutex<ContextImpl>);

impl ContextWrapper {
	pub fn new(
		config:       HashMap<String, serde_dyn_repr::Value>,
		mut files:    HashMap<PathBuf, Arc<RwLock<File>>>,
		changes_file: PathBuf
	) -> Self {
		files.insert(changes_file.clone(), Arc::new(RwLock::new(File::Config(ConfigFile {
			value:    HashMap::new(),
			includes: Vec::new()
		}))));
		
		Self(Mutex::new(ContextImpl {
			config,
			files,
			changes_file: Arc::new(changes_file),
			components:    Default::default()
		}))
	}
	
	async fn save_cfg(&self) {
		let (path, buf) = {
			let inner = self.0.lock().await;
			let path = inner.changes_file.clone();
			let file = match inner.files.get(&*inner.changes_file) {
				Some(v) => v,
				None => panic!()
			};
			
			let file = file.read().await;
			let file = match &*file {
				File::Config(v) => v,
				_ => panic!()
			};
			
			let buf = match serde_yaml::to_string(&file.value) {
				Ok(v)  => v,
				Err(e) => {
					log::error!("failed to save config: {}", e);
					return;
				}
			};
			
			(path, buf)
		};
		
		if let Err(e) = smol::fs::write(&*path, buf).await {
			log::error!("failed to save config: {}", e);
		}
	}
}

impl Context for ContextWrapper {
	fn spawn_dyn(&self, f: DynFuture<'static, ()>) {
		async_executor::spawn_dyn(f)
	}
	
	/*fn config_get(&self, key: &[&str]) -> serde_dyn_repr::Value {
		todo!()
	}
	
	fn config_set(&self, key: &[&str], val: Option<serde_dyn_repr::Value>, save: bool) {
		todo!()
	}*/
	
	fn component_id(&self, name: &str) -> u128 {
		use std::hash::{Hash, Hasher};
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		name.hash(&mut hasher);
		hasher.finish() as _
	}
	
	fn component_dyn_get(&self, id: u128, interface: u64) -> Arc<AtomicPtr<()>> {
		smol::block_on(self.0.lock())
			.components
			.entry((id, interface))
			.or_insert_with(|| Arc::new(AtomicPtr::default()))
			.clone()
	}
	
	fn component_dyn_add(&self, id: u128, interface: u64, ptr: *mut ()) {
		match smol::block_on(self.0.lock())
			.components
			.entry((id, interface))
		{
			Entry::Occupied(v) => v.get().store(ptr, Ordering::SeqCst),
			Entry::Vacant(v)   => { v.insert(Arc::new(AtomicPtr::new(ptr))); }
		}
	}
	
	fn component_dyn_del(&self, id: u128, interface: u64) -> bool {
		smol::block_on(self.0.lock())
			.components
			.remove(&(id, interface))
			.is_none()
	}
}

impl std::fmt::Debug for ContextWrapper {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("Context")
			.finish()
	}
}

impl std::fmt::Display for ContextWrapper {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}