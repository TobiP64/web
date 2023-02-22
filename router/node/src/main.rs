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
#![allow(clippy::option_map_unit_fn, unused_variables, clippy::uninit_vec, dead_code)]

extern crate kranus_protocols as net;

use std::{borrow::Cow, collections::HashMap, io::Write, str::FromStr, sync::*, time::Duration};
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "notify")]
use notify::{RecursiveMode, Watcher};
use global::watcher;

pub use kranus_router_node::*;

mod ctx;
mod global;
mod builtins;
mod utils;

type ModuleInitFn = for<'a> extern "Rust" fn(
	log: &'static dyn log::Log,
	lvl: log::LevelFilter,
	ctx: Arc<dyn Context>,
	trt: otel_mrt::Runtime,
	cfg: &'a mut (dyn erased_serde::Deserializer + Send + Sync)
) -> DynFuture<'a, Result<()>>;

const ENV_CONFIG:          &str = "KRANUS_ROUTER_CONFIG";
const ENV_WORKING_DIR:     &str = "KRANUS_ROUTER_WORKING_DIR";
const ENV_LOG_LEVEL:       &str = "KRANUS_ROUTER_LOG_LEVEL";
const DEFAULT_CONFIG_PATH: &str = "./config.toml";
const CFG_INCLUDE_KEY:     &str = "include";
const MODULE_INFO_CONST:   &str = "NET_SERVICES_PLUGIN_INFO\0";
const MODULE_INIT_FN:      &str = "net_services_plugin_init\0";
const HEADER_SERVER:       &str = "net-services";
const BANNER:              &str = r#"
             _                              _
            | |                            (_)
  _ __   ___| |_ ______ ___  ___ _ ____   ___  ___ ___  ___
 | '_ \ / _ \ __|______/ __|/ _ \ '__\ \ / / |/ __/ _ \/ __|
 | | | |  __/ |_       \__ \  __/ |   \ V /| | (_|  __/\__ \
 |_| |_|\___|\__|      |___/\___|_|    \_/ |_|\___\___||___/
     by Tobias Pfeiffer <tobias.pfeiffer@3dee74.net>
"#;

const HELP: &str = r#"
Usage: net-services [config file|config dir] [options...]

Options:
 -h, --help
 -i, --info
 -c, --config, --include <config file|module file|dir>
 -a, --abort
 -d, --dry-run, --check

Environment Varaibles:
 KRANUS_ROUTER_WORKER_THREADS
 KRANUS_ROUTER_CONFIG
 KRANUS_ROUTER_WORKING_DIR
 KRANUS_ROUTER_LOG_LEVEL
 OTEL_EXPORTER_OTLP_ENDPOINT
 OTEL_EXPORTER_OTLP_PROTOCOL
 OTEL_EXPORTER_OTLP_INSECURE
 OTEL_EXPORTER_OTLP_COMPRESSION
 OTEL_METRIC_EXPORT_INTERVAL
 OTEL_EXPORTER_OTLP_TIMEOUT
 OTEL_RESOURCE_ATTRIBUTES
 OTEL_SERVICE_NAME
 OTEL_ATTRIBUTE_VALUE_LENGTH_LIMIT
 OTEL_ATTRIBUTE_COUNT_LIMIT
 OTEL_SPAN_ATTRIBUTE_VALUE_LENGTH_LIMIT
 OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT
 OTEL_SPAN_EVENT_COUNT_LIMIT
 OTEL_SPAN_LINK_COUNT_LIMIT
 OTEL_EVENT_ATTRIBUTE_COUNT_LIMIT
 OTEL_LINK_ATTRIBUTE_COUNT_LIMIT
 OTEL_METRICS_EXEMPLAR_FILTER
"#;

const INFO: &str = concat!(
	"\nVersion:      ", env!("CARGO_PKG_VERSION"),
	"\nAuthors:      ", env!("CARGO_PKG_AUTHORS"),
	"\nName:         ", env!("CARGO_PKG_NAME"),
	"\nRepository:   ", env!("CARGO_PKG_REPOSITORY"),
	"\nLicense:      ", env!("CARGO_PKG_LICENSE"),
	"\n"
);

fn main() {
	println!("{}", BANNER);
	let threads = std::env::var(ENV_WORKER_THREADS).ok().and_then(|v| v.parse().ok());
	log::set_max_level(log::LevelFilter::Debug);
	log::set_logger(stdout_log::get()).unwrap();
	async_executor::run(threads, run);
}

async fn run() {
	log::info!("init: commencing initialization sequence ...");
	let t = std::time::Instant::now();

    // apply ENV_WORKING_DIR

	if let Some(dir) = std::env::var_os(ENV_WORKING_DIR) {
		let display = std::path::Path::new(&dir).display();
		match std::env::set_current_dir(&dir) {
			Ok(()) => log::info!("init: changed working dir to {} (from env var `NET_SERVICES_WORKING_DIR`)", display),
			Err(e) => {
				log::error!("init: failed to change working dir to {} (from env var `NET_SERVICES_WORKING_DIR`): {}", display, e);
				std::process::exit(1);
			}
		}
	}

    // apply ENV_LOG_LEVEL

	if let Ok(level) = std::env::var(ENV_LOG_LEVEL) {
		match log::LevelFilter::from_str(&level) {
			Ok(v) => {
				log::set_max_level(v);
				log::info!("init: changed log level to {} (from env var `NET_SERVICES_LOG_LEVEL`)", v);
			}
			Err(_) => log::error!("init: invalid log level (from env var `NET_SERVICES_LOG_LEVEL`): {}", level)
		}
	}

    // locate config

	let mut args = std::env::args().skip(1).collect::<Vec<_>>().into_iter();
	let root_cfg = args.next()
		.filter(|s| !s.starts_with('-'))
		.map(|s| (s, "argument".to_string()))
		.or_else(|| std::env::var_os(ENV_CONFIG)
			.and_then(|s| s.into_string().ok())
			.map(|s| (s, "env var `NET_SERVICES_CONFIG`".to_string())))
		.unwrap_or_else(|| (DEFAULT_CONFIG_PATH.to_string(), format!("default path `{}`", DEFAULT_CONFIG_PATH)));

    // parse args

	let mut abort        = false;
	let mut dry_run      = false;
	let mut loaded       = 0;
	let mut errors       = 0;
	let mut errors_total = 0;
	let mut includes     = vec![root_cfg];
	let mut merged_cfg   = HashMap::new();
	let mut module_paths = Vec::new();
	let mut modules      = HashMap::new();
	let mut files        = HashMap::new();
	let (tx, rx)         = mpsc::channel();
	#[cfg(feature = "hot-reload")]
	let mut watcher = match notify::watcher(tx, watcher::DEFAULT_WATCHER_DELAY) {
		Ok(v) => v,
		Err(e) => {
			log::error!("init: failed to init fs watcher: {}", e);
			std::process::exit(1);
		}
	};

	while let Some(arg) = args.next() {
		let keys = match arg.strip_prefix('-') {
			Some("h") | Some("-help") => {
				println!("{}", HELP);
				std::process::exit(0);
			}
			Some("i") | Some("-info") => {
				println!("{}", INFO);
				std::process::exit(0);
			}
			Some("-include") | Some("c") | Some("-config") => {
				let path = match args.next() {
					Some(v) => v,
					None => {
						log::error!("init: expected path after `--include` and `--config`");
						std::process::exit(1);
					}
				};

				includes.push((path, "argument".to_string()));
				continue;
			}
			Some("a") | Some("-abort") => {
				abort = true;
				continue;
			}
			Some("d") | Some("-dry-run") | Some("-check") => {
				dry_run = true;
				continue;
			}
			Some(v) => {
				log::warn!("global: invalid argument: `{}`", v);
				continue;
			}
			None => arg
		};

		let val = match args.next() {
			Some(v) => v,
			None    => {
				log::error!("init: expected value after config key");
				std::process::exit(1);
			}
		};

		let mut map = &mut merged_cfg;
		let mut keys = keys.split('.').peekable();

		while let Some(key) = keys.next() {
			match keys.peek() {
				None => {
					match map.entry(key.to_string()) {
						std::collections::hash_map::Entry::Vacant(entry) => {
							entry.insert(serde_dyn_repr::Value::String(val));
						}
						std::collections::hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
							serde_dyn_repr::Value::Seq(v) => v.push(serde_dyn_repr::Value::String(val)),
							_ => {
								let old = entry.insert(serde_dyn_repr::Value::Seq(vec![serde_dyn_repr::Value::String(val)]));

								match entry.get_mut() {
									serde_dyn_repr::Value::Seq(v) => v.push(old),
									_ => unreachable!()
								}
							}
						}
					}

					break;
				}
				Some(_) => {
					map.insert(key.to_string(), serde_dyn_repr::Value::Map(HashMap::new()));

					match map.get_mut(key) {
						Some(serde_dyn_repr::Value::Map(v)) => map = v,
						_ => unreachable!()
					}
				}
			}
		}
	}

    // parse config

	while let Some((path, reference)) = includes.pop() {
		let metadata = match std::fs::metadata(&path) {
			Ok(v) => v,
			Err(e) => {
				log::error!("init: failed to read `{}` (referenced by {}): {}", path, reference, e);
				errors += 1;
				continue;
			}
		};

		#[cfg(feature = "hot-reload")]
		if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
			log::error!("init: failed to watch `{}` (referenced by {}): {}", path, reference, e);
			errors += 1;
			continue;
		}

		if metadata.is_dir() {
			let dir = match std::fs::read_dir(&path) {
				Ok(v)  => v,
				Err(e) => {
					log::error!("init: failed to read directory `{}` (referenced by {}): {}", path, reference, e);
					errors += 1;
					continue;
				}
			};

			for entry in dir {
				let entry = match entry {
					Ok(v)  => v,
					Err(e) => {
						log::error!("init: failed to read directory `{}` (referenced by {}): {}", path, reference, e);
						errors += 1;
						continue;
					}
				};

				if let Some(path) = entry.path().to_str() {
					includes.push((path.to_string(), reference.clone()));
				}
			}

			continue;
		}

		let (_, extension) = path.rsplit_once('.').unwrap_or(("", ""));

		match extension {
			"so" | "dll" => {
				module_paths.push((PluginPath::SharedLib(path), reference));
				continue;
			}
			"wasm" => {
				module_paths.push((PluginPath::Wasm(path), reference));
				continue;
			}
			_ => ()
		}

		let cfg = match std::fs::read_to_string(&path) {
			Ok(v)  => v,
			Err(e) => {
				log::error!("init: failed to read config `{}` (referenced by {}): {}", path, reference, e);
				errors += 1;
				continue;
			}
		};

		let cfg = match extension {
			"yml" | "yaml" => serde_yaml::from_str::<HashMap<String, serde_dyn_repr::Value>>(&cfg)
				.map_err(|v| v.to_string()),
			_ => toml::from_str::<HashMap<String, serde_dyn_repr::Value>>(&cfg)
				.map_err(|v| v.to_string())
		};

		let cfg = match cfg {
			Ok(v) => v,
			Err(e) => {
				log::error!("init: failed to parse config `{}` (referenced by {}): {}", path, reference, e);
				errors += 1;
				continue;
			}
		};

		let mut wrapped_cfg = serde_dyn_repr::Value::Map(cfg);

		if let Err(e) = substitute_env_vars(&mut wrapped_cfg) {
			log::error!("init: failed to substitute environment variables in config `{}` (referenced by {}): {}", path, reference, e);
			errors += 1;
			continue;
		}

		let cfg = match wrapped_cfg {
			serde_dyn_repr::Value::Map(v) => v,
			_ => unreachable!()
		};

		if let Some(serde_dyn_repr::Value::Seq(seq)) = cfg.get(CFG_INCLUDE_KEY) {
			for include in seq {
				match include {
					serde_dyn_repr::Value::String(include) => includes.push((include.clone(), format!("`{}`", path))),
					_ => {
						log::error!("init: failed to parse config `{}` (referenced by {}): `include` must be an array of strings", path, reference);
						errors += 1;
						continue;
					}
				}
			}
		}

		loaded += 1;
		merge_maps(&mut merged_cfg, cfg);
		log::info!("init: loaded config `{}` (referenced by {})", path, reference);
	}

	let cfg = merged_cfg;

	log::info!("init: loaded {} configs (skipped {} due to errors)", loaded, errors);

	errors_total += errors;
	let (mut loaded, mut errors) = (0, 0);

    // load modules

	for (path, reference) in module_paths {
		match path {
			PluginPath::SharedLib(path) => {
				let lib = match unsafe { libloading::Library::new(&path) } {
					Ok(v)  => v,
					Err(e) => {
						log::error!("init: failed to load module `{}` (referenced by {}): {}", path, reference, e);
						errors += 1;
						continue;
					}
				};

				loaded += 1;
				log::info!("init: loaded module `{}` (referenced by {})", path, reference);
				modules.insert(path, (reference, Arc::new(Plugin::SharedLib(lib))));
			}
			#[cfg(feature = "wasm-runtime")]
			PluginPath::Wasm(path) => {
				let wasm = match std::fs::read(path) {
					Ok(v)  => v,
					Err(e) => {
						log::error!("init: failed to load module `{}` (referenced by {}): {}", path, reference, e);
						errors += 1;
						continue;
					}
				};

				let instance = match wasmer_runtime::instantiate(&wasm, wasmer_runtime::imports! {}) {
					Ok(v) => v,
					Err(e) => {
						log::error!("init: failed to load module `{}` (referenced by {}): {}", path, reference, e);
						errors += 1;
						continue;
					}
				};

				loaded += 1;
				modules.insert(path, (reference, Arc::new(Plugin::Wasm(instance))));
				log::info!("global: loaded module `{}` (referenced by {})", path, reference);
			}
			#[cfg(not(feature = "wasm-runtime"))]
			PluginPath::Wasm(path) => {
				log::error!("global: failed to load module `{}` (referenced by {}): no WASM runtime (feature `wasm-runtime` is not enabled)", path, reference);
				errors += 1;
			}
		}
	}

	log::info!("init: loaded {} plugins (skipped {} due to errors)", loaded, errors);

	let changes_file = global::init(&mut <dyn erased_serde::Deserializer>::erase(serde_dyn_repr::Value::Map(cfg.clone()))).await;
	let ctx = Arc::new(ctx::ContextWrapper::new(cfg.clone(), files, changes_file));
	log::info!("init: initialized global context");

	if dry_run {
		log::info!("init: dry run flag set, exiting ...");
		std::process::exit(0);
	}

	errors_total += errors;
	let (loaded, errors) = (AtomicUsize::new(0), AtomicUsize::new(0));
	kranus_router_node::set_context(ctx.clone());

	smol::stream::StreamExt::for_each(net::utils::zip(modules.into_iter()
		.map(|(path, (reference, module))| Box::pin(async {
			let (path, reference, module) = (path, reference, module);
			(match module.as_ref() {
				Plugin::SharedLib(lib) => {
					let info = match unsafe { lib.get::<&'static &'static str>(MODULE_INFO_CONST.as_bytes()) } {
						Ok(v)  => *v,
						Err(e) => {
							errors.fetch_add(1, Ordering::SeqCst);
							log::error!("init: failed to initialize plugin `{}` (referenced by {}): {}", path, reference, e);
							return;
						}
					};

					let init_fn = match unsafe { lib.get::<ModuleInitFn>(MODULE_INIT_FN.as_bytes()) } {
						Ok(v)  => v,
						Err(e) => {
							errors.fetch_add(1, Ordering::SeqCst);
							log::error!("init: failed to initialize plugin `{}` (referenced by {}): {}", path, reference, e);
							return;
						}
					};

					// TODO if e.is::<Vec<Error>>(), count errors
					if let Err(e) = (init_fn)(log::logger(), log::max_level(), ctx.clone(), otel_rt.clone(),
											  &mut <dyn erased_serde::Deserializer>::erase(serde_dyn_repr::Value::Map(cfg.clone()))).await {
						errors.fetch_add(1, Ordering::SeqCst);
						log::error!("init: failed to initialize plugin `{}` (referenced by {}): {}", path, reference, e.display());
						return;
					}

					loaded.fetch_add(1, Ordering::SeqCst);
					log::info!("init: initialized plugin `{}` (referenced by {})", path, reference);
				}
				#[cfg(feature = "wasm-runtime")]
				Plugin::Wasm(instance) => {
					//instance.call(MODULE_INIT_FN, &[]);
					todo!()
				}
			}) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
		}))
		.chain(std::iter::once_with(|| Box::pin(async {
			if let Err(e) = builtins::run(&mut <dyn erased_serde::Deserializer>::erase(serde_dyn_repr::Value::Map(cfg.clone()))).await {
				log::error!("init: failed to initialize plugin `builtins`: {:?}", e);
				errors.fetch_add(1, Ordering::SeqCst);
			} else {
				log::info!("init: initialized plugin `builtins`");
				loaded.fetch_add(1, Ordering::SeqCst);
			}
		})))), |_| ()).await;

	errors_total += errors.load(Ordering::SeqCst);
	log::info!("init: initialized {} plugins (skipped {} due to errors)", loaded.load(Ordering::SeqCst), errors.load(Ordering::SeqCst));

	if abort && errors_total > 0 {
		log::error!("init: {} errors during initialization, shutting down", errors_total);
		std::process::exit(1);
	}

	log::info!("init: initialization completed with {} errors, server online ({:.3}s)", errors_total, t.elapsed().as_secs_f32());

	std::mem::drop(args);

	#[cfg(feature = "hot-reload")]
	if let Err(e) = std::thread::Builder::new()
		.name("watcher-main".to_string())
		.spawn(move || smol::block_on(watcher::run(ctx, rx, watcher))) {
		log::error!("watcher: failed to spawn thread");
	}
}

enum PluginPath {
	SharedLib(String),
	Wasm(String)
}

enum Plugin {
	SharedLib(libloading::Library),
	#[cfg(feature = "wasm-runtime")]
	Wasm(wasmer_runtime::Instance)
}

fn substitute_env_vars(val: &mut serde_dyn_repr::Value) -> std::result::Result<(), std::env::VarError> {
	use serde_dyn_repr::Value::*;

	match val {
		String(str) => {
			let mut i = 0;
			while let Some(start) = str[i..].find("${") {

				let end = match str[start + 2..].find('}') {
					Some(v) => v,
					None => break
				};

				let env_var = std::env::var(&str[start + 2..end])?;

				str.drain(start..=end);
				str.insert_str(start, &env_var);
				i = start + env_var.len();
			}

			Ok(())
		}
		Seq(seq)   => seq.iter_mut().try_for_each(substitute_env_vars),
		Map(map)   => map.iter_mut().try_for_each(|(_, v)| substitute_env_vars(v)),
		Enum(_, ref mut v) => substitute_env_vars(v),
		_          => Ok(())
	}
}

fn merge_maps(dst: &mut HashMap<String, serde_dyn_repr::Value>, src: HashMap<String, serde_dyn_repr::Value>) {
	for (key, val) in src {
		let mut dst = match dst.entry(key) {
			std::collections::hash_map::Entry::Occupied(v) => v,
			std::collections::hash_map::Entry::Vacant(v)   => {
				v.insert(val);
				continue;
			}
		};

		match (dst.get_mut(), val) {
			(serde_dyn_repr::Value::Map(dst), serde_dyn_repr::Value::Map(src)) => merge_maps(dst, src),
			(serde_dyn_repr::Value::Seq(dst), serde_dyn_repr::Value::Seq(src)) => merge_seqs(dst, src),
			(dst, src) => *dst = src
		}
	}
}

fn merge_seqs(dst: &mut Vec<serde_dyn_repr::Value>, src: Vec<serde_dyn_repr::Value>) {
	*dst = src; // TODO
}