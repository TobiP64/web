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

use std::io;
use std::path::PathBuf;
use serde::Deserialize;
use super::*;

pub mod console;
pub mod telemetry;
#[cfg(feature = "notify")]
pub mod watcher;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub global: Global
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Global {
	#[serde(default)]
	pub log_level:          Option<GlobalLogLevel>,
	pub working_dir:        Option<String>,
	pub changes_file:       Option<String>,
	#[serde(default)]
	pub disable_hot_reload: bool,
	pub worker_threads:     Option<usize>,
	pub max_queue_len:      Option<usize>,
	#[serde(default)]
	pub console:            global::console::Config,
	#[serde(default)]
	pub telemetry:          global::telemetry::Config,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlobalLogLevel { Off, Error, Warn, Info, Debug, Trace }

impl GlobalLogLevel {
	pub fn to_filter(self) -> log::LevelFilter {
		match self {
			Self::Off   => log::LevelFilter::Off,
			Self::Error => log::LevelFilter::Error,
			Self::Warn  => log::LevelFilter::Warn,
			Self::Info  => log::LevelFilter::Info,
			Self::Debug => log::LevelFilter::Debug,
			Self::Trace => log::LevelFilter::Trace
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ConfigEnum<T> {
	Bool(bool),
	Config(T)
}

pub async fn init<'a>(cfg: &'a mut (dyn dyn_serde::Deserializer<'a> + Send + Sync)) -> PathBuf {
	let cfg = match Config::deserialize(cfg) {
		Ok(v)  => v.global,
		Err(e) => {
			log::error!("global: failed to deserialize config: {}", e);
			std::process::exit(1);
		}
	};

	if let Some(level) = cfg.log_level {
		log::set_max_level(level.to_filter());
		log::info!("global: changed log level to {:?} (from config)", level);
	}

	if let Some(dir) = &cfg.working_dir {
		match std::env::set_current_dir(&dir) {
			Ok(()) => log::info!("global: working dir changed to {} (from config)", dir),
			Err(e) => {
				log::error!("global: failed to change working dir to {} (from config): {}", dir, e);
				std::process::exit(1);
			}
		}
	}

	if cfg.telemetry.enabled {
		otel_mrt::init_global(otel_mrt::Config {
			endpoint:                cfg.telemetry.endpoint.clone()
				.map_or(Cow::Borrowed(otel_mrt::DEFAULT_ENDPOINT), Cow::Owned),
			protocol:                cfg.telemetry.protocol
				.map_or(otel_mrt::DEFAULT_PROTOCOL, Into::into),
			tls_config:              None, // TODO
			interval:                cfg.telemetry.interval
				.unwrap_or(otel_mrt::DEFAULT_INTERVAL),
			timeout:                 cfg.telemetry.timeout
				.unwrap_or(otel_mrt::DEFAULT_TIMEOUT),
			instrumentation_library: None,
			..                       Default::default()
		}, Some(Box::new(|task| async_executor::spawn_dyn(task))));
	} else {
		otel_mrt::init_global(otel_mrt::Config::disabled(), None);
	}

	let otel_rt = otel_mrt::runtime();
	log::info!("global: initialized OpenTelemetry runtime (otel_mrt v{})", otel_mrt::VERSION);

	if cfg.console.enabled {
		let r = std::thread::Builder::new()
			.name("console".to_string())
			.stack_size(0x1000)
			.spawn(move || console::run(cfg.console));

		match r {
			Ok(_) => log::info!("builtin module `console` successfully initialized"),
			Err(e) => log::error!("builtin module `console` failed to initialize: {}", e),
		}
	}

	let changes_file = match cfg.changes_file {
		Some(v) => std::path::PathBuf::from(v),
		None => {
			let mut path = std::env::temp_dir();
			path.push("NET_SERVICES_CHANGES.yaml");
			path
		}
	};

	match smol::fs::File::open(&changes_file).await {
		Ok(_)  => log::info!("global: using config changes file {}", changes_file.display()),
		Err(e) if e.kind() != io::ErrorKind::NotFound => log::error!("global: failed to open config changes file {}: {}", changes_file.display(), e),
		Err(e) => match smol::fs::File::create(&changes_file).await {
			Ok(_)  => log::info!("global: created config changes file {}", changes_file.display()),
			Err(e) => log::error!("global: failed to create config changes file {}: {}", changes_file.display(), e)
		}
	}

	changes_file
}