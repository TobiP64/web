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
	super::*,
	crate::{interfaces::*, utils::{send_response, trie::*}},
	std::{path::*, time::UNIX_EPOCH, sync::{Arc, mpsc, atomic::*}, time::Duration},
	net::http::{self, traits::AsyncStreamExt},
	smol::{io::{AsyncReadExt, AsyncWriteExt}, stream::StreamExt},
	async_recursion::async_recursion,
	notify::{Watcher, DebouncedEvent}
};

pub(super) async fn run(name: &str, cfg: Config) -> Result<()> {
	let backend = Arc::new(FsBackend {
		name:      name.to_string(),
		resources: TrieNode::default()
	});
	
	FsBackend::add_resource(
		&backend.resources,
		name,
		cfg.dir.trim_end_matches('/'),
		&mut PathBuf::from(&cfg.dir),
		cfg.http.unwrap().load
	).await;
	
	#[cfg(feature = "hot-reload")]
	if cfg.reload {
		let moved_dir = cfg.dir;
		let backend = backend.clone();
		std::thread::Builder::new()
			.name(format!("watcher-{}", &backend.name))
			.spawn(move || smol::future::block_on(
				async move { backend.watch(&moved_dir).await; }))?;
	}
	
	let id = crate::component_id(name);
	crate::add_component::<HttpStreamHandler>(id, Box::new(__Arc_StorageBackend__(backend)));
	Ok(())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub dir:       String,
	#[serde(default)]
	pub preload:   bool,
	#[serde(default)]
	pub reload:    bool,
	#[serde(default)]
	pub writable:  bool,
	#[serde(default)]
	pub recursive: bool,
	pub filter:    ConfigFilter,
	pub http:      Option<ConfigHttp>,
	pub smtp:      Option<ConfigSmtp>,
	pub dns:       Option<ConfigDns>
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFilter {
	pub allow_list: Vec<StringMatcher>,
	pub deny_list:  Vec<StringMatcher>
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigHttp {
	pub etag: Option<ConfigHttpEtag>,
	#[serde(default)]
	pub date: bool,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum ConfigHttpEtag {
	#[serde(alias = "sha3")]
	#[serde(alias = "SHA3")]
	#[serde(alias = "sha-3")]
	#[serde(alias = "SHA-3")]
	Sha3,
	#[serde(alias = "xxh3")]
	#[serde(alias = "XXH3")]
	#[serde(alias = "xxh-3")]
	#[serde(alias = "XXH-3")]
	Xxh3
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSmtp {

}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigDns {

}

struct FsBackend {
	name:      String,
	resources: TrieNode<StorageBackendResource>
}

impl FsBackend {
	#[async_recursion]
	async fn add_resource(resources: &TrieNode<StorageBackendResource>, backend: &str, root: &str, path: &mut PathBuf, load: bool) {
		let metadata = match smol::fs::metadata(&path).await {
			Ok(v) => v,
			Err(e) => {
				log::error!("backend `{}` resource `{}`: failed to read metadata: {}", backend, path.display(), e);
				return;
			}
		};
		
		let last_read     = UNIX_EPOCH.elapsed().unwrap().as_millis();
		let last_write    = metadata.modified().map_or(last_read, |t| t.duration_since(UNIX_EPOCH).unwrap().as_millis());
		let path_relative = "/".to_string() + path.strip_prefix(root).unwrap().to_str().unwrap();
		
		if metadata.is_dir() {
			let __tmp__ = path_relative.ends_with('/');
			let path_relative = path_relative + if __tmp__ { "" } else { "/" };
			
			resources.insert(&path_relative, StorageBackendResource {
				state:      State::Dir,
				path:       path.clone().into_boxed_path(),
				dirty:      AtomicBool::new(false),
				last_read:  AtomicU64::new(last_read as _),
				last_write: AtomicU64::new(last_write as _),
				headers:    Vec::new()
			}).await;
			
			let mut entries = match smol::fs::read_dir(&path).await {
				Ok(v) => v,
				Err(e) => {
					log::error!("backend `{}` resource `{}`: failed to read dir: {}", backend, path.display(), e);
					return;
				}
			};
			
			while let Some(entry) = entries.next().await {
				let entry = match entry {
					Ok(v) => v,
					Err(e) => {
						log::error!("backend `{}` resource `{}`: failed to read dir: {}", backend, path.display(), e);
						continue;
					}
				};
				
				path.push(entry.file_name());
				Self::add_resource(resources, backend, root, &mut*path, load).await;
				path.pop();
			}
		} else if metadata.is_file() {
			resources.insert(&path_relative, StorageBackendResource {
				state:      if load {
					match smol::fs::read(&path).await {
						Ok(v) => State::Loaded(v.into_boxed_slice()),
						Err(e) => {
							log::error!("backend `{}` resource `{}`: failed to read file: {}", backend, path.display(), e);
							return;
						}
					}
				} else {
					State::NotLoaded
				},
				path:       path.clone().into_boxed_path(),
				dirty:      AtomicBool::new(false),
				last_read:  AtomicU64::new(last_read as _),
				last_write: AtomicU64::new(last_write as _),
				headers:    Vec::new()
			}).await;
		} else {
			log::error!("backend `{}` resource `{}`: unknown type", backend, path.display());
			return;
		}
		
		log::info!("backend `{}` resource `{}`: {}", backend, path.display(), if load { "loaded" } else { "added" });
	}
	
	#[cfg(feature = "hot-reload")]
	async fn watch(&self, dir: &str) {
		let (tx, rx) = mpsc::channel();
		let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
			Ok(v) => v,
			Err(e) => {
				log::error!("backend `{}` watcher: {}", &self.name, e);
				return;
			}
		};
		
		if let Err(e) = watcher.watch(dir, notify::RecursiveMode::Recursive) {
			log::error!("backend `{}` watcher: {}", &self.name, e);
			return;
		}
		
		log::info!("backend `{}` watcher: started", &self.name);
		let mut prefix = std::env::current_dir().unwrap();
		prefix.push(dir);
		
		fn prepare_path(path: &Path, prefix: &Path) -> String {
			let path = path.strip_prefix(&prefix).unwrap();
			let mut buf = PathBuf::with_capacity(1 + path.to_str().unwrap().len());
			buf.push("/");
			buf.push(path);
			buf.to_string_lossy().into_owned()
		}
		
		loop {
			match rx.recv() {
				Ok(DebouncedEvent::Create(fs_path)) => {
					let path = prepare_path(&fs_path, &prefix);
					let (parent, relative_path, _idx) = match self.resources.child(&path).await {
						Ok(_)  => continue,
						Err(v) => v
					};
					
					let last_read = UNIX_EPOCH.elapsed().unwrap().as_millis();
					parent.insert(relative_path, StorageBackendResource {
						state:      match smol::fs::read(&fs_path).await {
							Ok(v) => State::Loaded(v.into_boxed_slice()),
							Err(e) => {
								log::error!("backend `{}` resource `{}` (`{}`): failed to read file: {}",
									&self.name, &path, fs_path.display(), e);
								continue;
							}
						},
						path:       fs_path.into_boxed_path(),
						dirty:      AtomicBool::new(false),
						last_read:  AtomicU64::new(last_read as _),
						last_write: AtomicU64::new(last_read as _),
						headers:    Vec::new()
					}).await;
					
					log::info!("backend `{}` resource `{}`: created", &self.name, &path);
				}
				Ok(DebouncedEvent::Write(fs_path)) => {
					let path = prepare_path(&fs_path, &prefix);
					let node = match self.resources.get(&path).await {
						Some(v) => v,
						None    => continue // resource was deleted or filtered
					};
					let mut data = node.data.write().await;
					let data = match &mut*data {
						Some(v) => v,
						None    => continue // resource was deleted or filtered
					};
					
					if data.dirty.load(Ordering::Relaxed) || data.state == State::NotLoaded {
						log::warn!("backend `{}` resource `{}`: resource is out of sync", &self.name, &path);
						return;
					}
					
					match smol::fs::read(&fs_path).await {
						Ok(v)  => {
							log::info!("backend `{}` resource `{}`: reloaded", &self.name, &path);
							data.state = State::Loaded(v.into_boxed_slice())
						}
						Err(e) => log::error!("backend `{}` resource `{}`: failed to reload file: {}", &self.name, &path, e)
					}
				}
				Ok(DebouncedEvent::Remove(path)) => {
					let path = prepare_path(&path, &prefix);
					let node = match self.resources.get(&path).await {
						Some(v) => v,
						None => continue // resource was deleted or filtered
					};
					let data = node.data.write().await;
					let data = match &*data {
						Some(v) => v,
						None => continue // resource was deleted or filtered
					};
					
					if data.dirty.load(Ordering::Relaxed) {
						log::warn!("backend `{}` resource `{}`: resource is out of sync", &self.name, &path);
						return;
					}
					
					self.resources.delete(&path).await;
					log::info!("backend `{}` resource `{}`: deleted", &self.name, &path);
				}
				Ok(DebouncedEvent::Error(e, path)) => log::error!("backend `{}` watcher (`{}`): {}", &self.name, path.unwrap_or_default().display(), e),
				Err(e)             => log::error!("backend `{}` watcher: {}", &self.name, e),
				_ => ()
			}
		}
	}
}

#[allow(non_camel_case_types)]
struct __Arc_StorageBackend__(Arc<FsBackend>);

impl std::ops::Deref for __Arc_StorageBackend__ {
	type Target = FsBackend;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl StreamHandler<dyn http::traits::AsyncStream> for __Arc_StorageBackend__ {
	fn accept<'a>(&'a self, stream: &'static mut dyn http::traits::AsyncStream) -> DynFuture<'a, Result<()>> {
		Box::pin(async move {
			let headers = stream.read_headers().await?;
			let mut body = Vec::new();
			stream.read_to_end(&mut body).await?;
			
			let (mut method, mut path, mut length, mut range) = (None, None, None, None);
			
			for header in headers {
				match header {
					http::Header::Method(v)        => method = Some(v),
					http::Header::Path(v)          => path   = Some(v),
					http::Header::ContentLength(v) => length = Some(v),
					http::Header::Range(v)         => range  = Some(v),
					_ => ()
				}
			}
			
			let (path, method) = match (path, method) {
				(Some(path), Some(method)) => (path, method),
				_ => return send_response(stream, http::Status::BadRequest).await
			};
			let i = path.find('?').unwrap_or_else(|| path.len());
			
			match (self.resources.child(&path[..i]).await, method) {
				(Ok(node), method @ (http::Method::Get | http::Method::Head)) => {
					let data = node.data.read().await;
					let resource = data.as_ref().unwrap();
					resource.last_read.store(UNIX_EPOCH.elapsed().unwrap().as_millis() as _, Ordering::Relaxed);
					
					#[allow(unused_assignments)]
					let mut data_owned = None;
					let data = match &resource.state {
						State::Dir          => return send_response(stream, http::Status::NotFound).await,
						State::Loaded(data) => &*data,
						State::NotLoaded    => match smol::fs::read(&*resource.path).await {
							Ok(v)  => {
								log::info!("backend `{}` resource `{}`: loaded (on demand)", &self.name, &path);
								data_owned = Some(v);
								data_owned.as_ref().unwrap().as_slice()
							}
							Err(e) => {
								log::error!("backend `{}` resource `{}`: failed to load file: {}", &self.name, &path, e);
								return send_response(stream, http::Status::InternalServerError).await;
							}
						}
					};
					
					match range {
						Some(http::Ranges { unit: http::Unit::Bytes, ranges }) => match ranges.as_slice() {
							[] => send_response(stream, http::Status::RangeNotSatisfiable).await,
							[range] => {
								let (start, end) = match range {
									http::Range { start: Some(start), end: None }      => (*start, data.len() - 1),
									http::Range { start: None,        end: Some(end) } => (0, *end),
									http::Range { start: Some(start), end: Some(end) } => (*start, *end),
									http::Range { start: None,        end: None }      => (0, 0)
								};
								
								if start > data.len() - 1 || end > data.len() - 1 {
									send_response(stream, http::Status::RangeNotSatisfiable).await
								} else {
									stream.write_headers(&[
										http::Header::Status(http::Status::PartialContent),
										http::Header::Server(HEADER_SERVER.to_string()),
										http::Header::ContentLength(end - start + 1),
										http::Header::ContentRange(http::ContentRange {
											unit:  http::Unit::Bytes,
											range: Some((start, end)),
											size:  Some(data.len())
										})
									]).await?;
									stream.write_all(if method == http::Method::Head { &[] } else { &data[start..=end] }).await?;
									Ok(())
								}
							}
							_ranges => send_response(stream, http::Status::NotImplemented).await
						},
						Some(_) => send_response(stream, http::Status::RangeNotSatisfiable).await,
						None => {
							stream.write_headers(&[
								http::Header::Status(http::Status::Ok),
								http::Header::Server(HEADER_SERVER.to_string()),
								http::Header::ContentLength(data.len())
							]).await?;
							stream.write_all(if method == http::Method::Head { &[] } else { data }).await?;
							Ok(())
						}
					}
				}
				(Ok(node), http::Method::Put) => {
					let length = match length {
						Some(v) => v,
						None => return send_response(stream, http::Status::LengthRequired).await
					};
					
					let mut buf = vec![0u8; length].into_boxed_slice();
					stream.read_exact(&mut buf).await?;
					
					let mut data = node.data.write().await;
					let resource = data.as_mut().unwrap();
					
					if let State::Dir = &resource.state {
						return send_response(stream, http::Status::MethodNotAllowed).await;
					}
					
					resource.state = State::Loaded(buf);
					resource.dirty.store(true, Ordering::Relaxed);
					resource.last_write.store(UNIX_EPOCH.elapsed().unwrap().as_millis() as _, Ordering::Relaxed);
					log::info!("backend `{}` resource `{}`: updated", &self.name, path);
					send_response(stream, http::Status::NoContent).await
				}
				(Ok(node), http::Method::Delete) => {
					let r = {
						let data = node.data.read().await;
						let resource = data.as_ref().unwrap();
						
						match resource.state {
							State::Dir => smol::fs::remove_dir_all(&*resource.path).await,
							_ => smol::fs::remove_file(&*resource.path).await
						}
					};
					
					(*node).delete().await;
					
					// TODO delete if file is not on disk
					
					match r {
						Ok(()) => {
							log::info!("backend `{}` resource `{}`: deleted", &self.name, path);
							send_response(stream, http::Status::NoContent).await
						}
						Err(e) => {
							log::error!("backend `{}` resource `{}`: failed to delete: {}", &self.name, path, e);
							send_response(stream, http::Status::InternalServerError).await
						}
					}
				}
				(Err((parent, rpath, _)), http::headers::Method::Put) => {
					let length = match length {
						Some(v) => v,
						None => return send_response(stream, http::Status::LengthRequired).await
					};
					
					let mut dir = parent.clone();
					
					while !dir.path.ends_with('/') {
						dir = match dir.parent() {
							Some(v) => v,
							None => return send_response(stream, http::Status::NotFound).await
						};
					}
					
					let mut buf = vec![0u8; length].into_boxed_slice();
					stream.read_exact(&mut buf).await?;
					
					#[cfg(feature = "http-etag-sha3")]
						{
							use sha3::Digest;
							let hasher = sha3::Sha3_512::new();
							hasher.update(&buf);
							let hash = hasher.finalize().as_slice();
							base64::encode(hash);
						}
					
					#[cfg(feature = "http-etag-xxh3")]
						{
							let hash = xxhash_rust::xxh3::xxh3_128(&buf).to_ne_bytes();
							base64::encode(hash);
						}
					
					let time = UNIX_EPOCH.elapsed().unwrap().as_millis() as _;
					parent.insert(rpath, StorageBackendResource {
						state:     State::Loaded(buf),
						path:       {
							let mut __tmp__ = dir.data.read().await
								.as_ref().unwrap()
								.path
								.to_path_buf();
							__tmp__.push(path.strip_prefix(&*dir.path).unwrap());
							__tmp__.into_boxed_path()
						},
						dirty:      AtomicBool::new(true),
						last_read:  AtomicU64::new(time),
						last_write: AtomicU64::new(time),
						headers:    Vec::new()
					}).await;
					
					log::info!("backend `{}` resource `{}`: created", &self.name, path);
					send_response(stream, http::Status::Created).await
				}
				(Ok(_),  _) => send_response(stream, http::Status::MethodNotAllowed).await,
				(Err(_), _) => send_response(stream, http::Status::NotFound).await
			}
		})
	}
}

#[derive(Debug)]
struct StorageBackendResource {
	state:      State,
	path:       Box<Path>,
	dirty:      AtomicBool,
	last_read:  AtomicU64,
	last_write: AtomicU64,
	headers:    Vec<http::Header>
}

#[derive(Eq, PartialEq)]
enum State {
	Dir,
	NotLoaded,
	Loaded(Box<[u8]>)
}

impl std::fmt::Debug for State {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Dir       => "Dir",
			Self::NotLoaded => "NotLoaded",
			Self::Loaded(_) => "Loaded"
		})
	}
}