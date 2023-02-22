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

#![allow(dead_code, unused_variables)]

extern crate kranus_protocols as net;

use {
	std::{
		borrow::{Borrow, Cow},
		collections::{HashMap, HashSet},
		marker::PhantomData,
		ptr,
		time::{Duration, Instant},
		sync::{Arc, Mutex, atomic::{AtomicPtr, Ordering}},
		hash::{Hash, Hasher}
	},
	net::otlp::{metrics::*, tracing::*, logging::*}
};

pub use {
	metrics::*,
	tracing::*,
	logging::*,
	exporter::{Protocol, Compression},
	net::otlp::{common::*, metrics::AggregationTemporality}
};

pub mod metrics;
pub mod tracing;
pub mod logging;
mod exporter;

pub const VERSION:                 &str           = env!("CARGO_PKG_VERSION");
pub const DEFAULT_ENDPOINT:        &str           = "localhost:55690";
pub const DEFAULT_PROTOCOL:        Protocol       = Protocol::HttpProto;
pub const DEFAULT_INSECURE:        bool           = false;
pub const DEFAULT_INTERVAL:        Duration       = Duration::from_secs(10);
pub const DEFAULT_TIMEOUT:         Duration       = Duration::from_secs(10);
pub const DEFAULT_SAMPLER:         Sampler        = Sampler::ParentBasedAlwaysOn;
pub const DEFAULT_EXEMPLAR_FILTER: ExemplarFilter = ExemplarFilter::WithSampledTrace;
pub const DEFAULT_LENGTH_LIMIT:    usize          = usize::MAX;
pub const DEFAULT_COUNT_LIMIT:     usize          = 128;
pub const DEFAULT_BUF_LEN:         usize          = 256;

static RUNTIME: AtomicPtr<RuntimeInner> = AtomicPtr::new(ptr::null_mut());

pub type Attributes<'a> = &'a [(Cow<'static, str>, AnyValue)];
pub type Executor = dyn Fn(std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>) + Send + Sync + 'static;

pub fn runtime() -> Runtime {
	get_global().expect("OpenTelemetry runtime has not yet been initialized")
}

pub fn init_global(config: Config, executor: Option<Box<Executor>>) {
	unsafe { set_global(Runtime::new(config, executor)); }
}

/// # Safety
///
/// This is an internal function, that is expose solely for the purpose of initializing the runtime
/// in dynamically loaded libraries and should not be used for anything else.
pub unsafe fn set_global(runtime: Runtime) {
	let old = RUNTIME.swap(Arc::as_ptr(&runtime.0) as *mut _, Ordering::SeqCst);
	std::mem::forget(runtime);
	if !old.is_null() {
		std::mem::drop(Arc::from_raw(old));
	}
}

pub fn get_global() -> Option<Runtime> {
	let ptr = RUNTIME.load(Ordering::SeqCst);
	(!ptr.is_null()).then(|| unsafe { Runtime(Arc::from_raw(ptr)) })
}

#[derive(Clone)]
pub struct Config {
	pub disabled:                          bool,
	pub instrumentation_library:           Option<InstrumentationLibrary>,
	pub endpoint:                          Cow<'static, str>,
	pub protocol:                          Protocol,
	pub tls_config:                        Option<Arc<net::tls::r#async::rustls::ClientConfig>>,
	pub compression:                       Option<Compression>,
	pub interval:                          Duration,
	pub timeout:                           Duration,
	pub resource:                          Vec<KeyValue>,
	pub traces_sampler:                    Sampler,
	pub attribute_value_length_limit:      usize,
	pub attribute_count_limit:             usize,
	pub span_attribute_value_length_limit: usize,
	pub span_attribute_count_limit:        usize,
	pub span_event_count_limit:            usize,
	pub span_link_count_limit:             usize,
	pub span_event_attribute_count_limit:  usize,
	pub span_link_attribute_count_limit:   usize,
	pub metric_exemplar_filter:            ExemplarFilter
}

impl Config {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn disabled() -> Self {
		Self { disabled: true, ..Self::default() }
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			disabled:                          false,
			instrumentation_library:           None,
			endpoint:                          std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok()
				.map_or(Cow::Borrowed(DEFAULT_ENDPOINT), Cow::Owned),
			protocol:                          std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_PROTOCOL),
			tls_config:                        std::env::var("OTEL_EXPORTER_OTLP_INSECURE")
				.map_or(DEFAULT_INSECURE, |v| v == "true")
				.then(|| Arc::new(net::tls::r#async::rustls::ClientConfig::new())),
			compression:                       std::env::var("OTEL_EXPORTER_OTLP_COMPRESSION").ok()
				.and_then(|v| v.parse().ok()),
			interval:                          std::env::var("OTEL_METRIC_EXPORT_INTERVAL").ok()
				.and_then(|v| v.parse().ok())
				.map_or(DEFAULT_INTERVAL, Duration::from_millis),
			timeout:                           std::env::var("OTEL_EXPORTER_OTLP_TIMEOUT").ok()
				.and_then(|v| v.parse().ok())
				.map_or(DEFAULT_TIMEOUT, Duration::from_millis),
			resource:                          std::env::var("OTEL_RESOURCE_ATTRIBUTES").ok()
				.unwrap_or_default()
				.split(',')
				.map(|attr| {
					let (key, val) = attr.split_once('=').unwrap_or((attr, ""));
					KeyValue {
						key:   key.to_string(),
						value: AnyValue::String(val.to_string())
					}
				})
				.chain(std::env::var("OTEL_SERVICE_NAME").ok()
					.map(|val| KeyValue {
						key:   "service.name".to_string(),
						value: AnyValue::String(val)
				}))
				.collect(),
			traces_sampler:                    Sampler::from_env()
				.unwrap_or(DEFAULT_SAMPLER),
			attribute_value_length_limit:      std::env::var("OTEL_ATTRIBUTE_VALUE_LENGTH_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_LENGTH_LIMIT),
			attribute_count_limit:             std::env::var("OTEL_ATTRIBUTE_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			span_attribute_value_length_limit: std::env::var("OTEL_SPAN_ATTRIBUTE_VALUE_LENGTH_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_LENGTH_LIMIT),
			span_attribute_count_limit:        std::env::var("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			span_event_count_limit:            std::env::var("OTEL_SPAN_EVENT_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			span_link_count_limit:             std::env::var("OTEL_SPAN_LINK_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			span_event_attribute_count_limit:  std::env::var("OTEL_EVENT_ATTRIBUTE_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			span_link_attribute_count_limit:   std::env::var("OTEL_LINK_ATTRIBUTE_COUNT_LIMIT").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_COUNT_LIMIT),
			metric_exemplar_filter:            std::env::var("OTEL_METRICS_EXEMPLAR_FILTER").ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(DEFAULT_EXEMPLAR_FILTER),
		}
	}
}

#[derive(Clone)]
pub struct Runtime(Arc<RuntimeInner>);

struct RuntimeInner {
	config:   Config,
	executor: Box<Executor>,
	sync:     Mutex<RuntimeSync>,
}

struct RuntimeSync {
	instruments:    HashSet<Arc<InstrumentInner>>,
	spans:          Vec<tracing::Span>,
	logs:           Vec<LogRecord>,
	attribute_keys: HashMap<u64, Cow<'static, str>>,
	attributes_buf: Vec<(u64, AnyValue)>,
	memory_pool:    RuntimeAlloc
}

struct RuntimeAlloc {
	size: usize,
	pool: *mut (),
	free: *mut ()
}

unsafe impl Send for RuntimeAlloc {}
unsafe impl Sync for RuntimeAlloc {}

impl Runtime {
	pub fn new(config: Config, executor: Option<Box<Executor>>) -> Self {
		let self_ = Self(Arc::new(RuntimeInner {
			config,
			executor:   executor.unwrap_or_else(exporter::get_default_executor),
			sync:       Mutex::new(RuntimeSync {
				instruments:    HashSet::with_capacity(DEFAULT_BUF_LEN),
				spans:          Vec::with_capacity(DEFAULT_BUF_LEN),
				logs:           Vec::with_capacity(DEFAULT_BUF_LEN),
				attribute_keys: HashMap::with_capacity(DEFAULT_BUF_LEN),
				attributes_buf: Vec::with_capacity(DEFAULT_BUF_LEN),
				memory_pool:    RuntimeAlloc {
					size: 0,
					pool: std::ptr::null_mut(),
					free: std::ptr::null_mut()
				}
			})
		}));

		let runtime = self_.0.clone();
		(self_.0.executor)(Box::pin(async move { exporter::run(runtime).await; }));
		self_
	}

	pub fn config(&self) -> &Config {
		&self.0.config
	}

	pub fn instrument<T: DataPointValue>(&self, params: InstrumentParameters) -> Instrument<T> {
		Instrument::new(self, params)
	}

	pub fn bound_instrument<T: DataPointValue, A: IntoIterator<Item = (Cow<'static, str>, AnyValue)>>(
		&self, params: InstrumentParameters, attrs: A) -> BoundInstrument<T> {
		self.instrument(params).bind(attrs)
	}

	pub fn observable_instrument<T: DataPointValue>(&self, params: InstrumentParameters, callback: impl Fn(&Instrument<T>) + Send + Sync + 'static) {
		Instrument::new_observable(self, params, callback)
	}

	pub fn span(&self, attributes: impl IntoIterator<Item = (Cow<'static, str>, AnyValue)>, links: impl IntoIterator<Item = Link>) -> Span {
		unimplemented!()
	}

	pub fn log(&self, params: LogParams) {
		unimplemented!()
	}

	pub fn logger<T: log::Log>(&self, inner: T) -> Logger<T> {
		Logger::new(self, inner)
	}
}

impl std::fmt::Debug for Runtime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.finish_non_exhaustive()
	}
}

impl RuntimeSync {
	fn clear(&mut self) {
		unsafe {
			std::ptr::write(&mut self.logs, Vec::new());
			std::ptr::write(&mut self.spans, Vec::new());
			self.memory_pool.free = self.memory_pool.pool;
			*(self.memory_pool.free as *mut usize) = self.memory_pool.size;
		}
	}
}