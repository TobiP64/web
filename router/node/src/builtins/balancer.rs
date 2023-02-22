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
	crate::interfaces::*,
	net::http
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	#[serde(default)]
	pub backends: Vec<ConfigBackend>,
	#[serde(default)]
	pub method:   ConfigMethod
}

#[derive(Clone, Debug, Deserialize)]
pub enum ConfigBackendEnum {
	Name(String),
	Config(ConfigBackend)
}

impl From<ConfigBackendEnum> for ConfigBackend {
	fn from(v: ConfigBackendEnum) -> Self {
		match v {
			ConfigBackendEnum::Name(name) => Self { name, weight: 1.0 },
			ConfigBackendEnum::Config(cfg) => cfg
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "ConfigBackendEnum")]
#[serde(deny_unknown_fields)]
pub struct ConfigBackend {
	pub name: String,
	#[serde(default = "f32_one")]
	pub weight: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ConfigMethod {
	First,
	Nearest,
	RoundRobin
}

impl Default for ConfigMethod {
	fn default() -> Self {
		Self::Nearest
	}
}

pub(super) async fn run(name: &str, cfg: Config) -> Result<()> {
	let mut backends = Vec::new();

	for cfg_ in cfg.backends {
		backends.push(Backend {
			backend: crate::get_component::<HttpStreamHandler>(crate::component_id(&cfg_.name)),
			name:    cfg_.name,
			weight:  cfg_.weight,
			latency: AtomicUsize::new(0)
		});
	}

	backends.sort_by(|a, b| std::cmp::PartialOrd::partial_cmp(&a.weight, &b.weight)
		.unwrap_or(std::cmp::Ordering::Equal));

	let id = crate::component_id(name);
	crate::add_component::<HttpStreamHandler>(id, Box::new(BackendSelector {
		name:     name.to_string(),
		balancer: cfg.method,
		rr_idx:   AtomicUsize::new(0),
		backends
	}));
	Ok(())
}

struct BackendSelector {
	name:     String,
	balancer: ConfigMethod,
	rr_idx:   AtomicUsize,
	backends: Vec<Backend>
}

struct Backend {
	name:    String,
	backend: ComponentRef<HttpStreamHandler>,
	weight:  f32,
	latency: AtomicUsize
}

impl StreamHandler<dyn http::traits::AsyncStream> for BackendSelector {
	fn accept<'a>(&'a self, stream: &'static mut dyn http::traits::AsyncStream) -> DynFuture<'a, Result<()>> {
		let idx = match self.balancer {
			ConfigMethod::First      => 0,
			ConfigMethod::Nearest    => 0,
			ConfigMethod::RoundRobin => self.rr_idx.fetch_add(1, Ordering::Relaxed) % self.backends.len()
		};

		let backend = &self.backends[idx];

		log::info!("processor `{}`: selected backend #{} `{}` (weight: {}, latency: {}ms)",
			&self.name, idx, &backend.name, backend.weight, backend.latency.load(Ordering::Relaxed));

		backend.backend.accept(stream)
	}
}