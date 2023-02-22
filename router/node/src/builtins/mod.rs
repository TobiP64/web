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

///! Contains all builtin modules.

use {serde::Deserialize, std::str::FromStr, super::*};

pub mod api;
pub mod auth;
pub mod balancer;
pub mod cache;
pub mod relay;
pub mod router;
pub mod socket;
pub mod storage;

pub async fn run<'a>(cfg: &'a mut (dyn dyn_serde::Deserializer<'a> + Send + Sync)) -> Result<()> {
	let cfg = Config::deserialize(cfg)
		.with_msg("failed to parse config")?;

	for (name, cfg) in cfg.builtin {
		let (spec, e) = match cfg {
			Module::Socket(cfg) => ("socket", socket::run(&name, cfg)),

		};

		match r {
			Ok(_)  => log::info!("builtin module `{}` ({}) successfully initialized", &name, spec),
			Err(e) => log::error!("builtin module `{}` ({}) failed to initialize: {:?}", &name, spec, e)
		}
	}

	Ok(())
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
	pub builtin: HashMap<String, Module>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Module {
	Api(api::Config),
	Auth(auth::Config),
	Balancer(balancer::Config),
	Cache(cache::Config),
	Relay(relay::Config),
	Router(router::Config),
	Socket(socket::Config),
	Storage(storage::Config)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocket {
	pub pipe:  Option<String>,
	pub udp:   Option<ConfigSocketUdp>,
	pub tcp:   Option<ConfigSocketTcp>,
	pub quic:  Option<ConfigSocketQuic>,
	pub tls:   Option<ConfigSocketTls>,
	pub http1: Option<ConfigSocketHttp1>,
	pub http3: Option<ConfigSocketHttp3>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketUdp {
	pub host: Option<String>,
	pub port: Option<u16>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketTcp {
	pub host:           Option<String>,
	pub port:           Option<u16>,
	pub read_timeout:   Option<Duration>,
	pub write_timeout:  Option<Duration>,
	pub ttl:            Option<u32>,
	pub ipv6_flow_info: Option<u32>,
	pub ipv6_scope_id:  Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketQuic {
	#[serde(default = "usize_max")]
	pub max_concurrent_streams:         usize,
	#[serde(default = "usize_max")]
	pub initial_stream_window_size:     usize,
	#[serde(default = "usize_max")]
	pub initial_connection_window_size: usize
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketTls {
	pub private_key: String,
	pub certificate: String,
	#[serde(default)]
	pub alpn:        bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketHttp1 {
	#[serde(default = "usize_max")]
	pub buffer_size:             usize,
	#[serde(default = "usize_max")]
	pub idle_timeout:            usize,
	#[serde(default = "usize_max")]
	pub max_connection_duration: usize,
	#[serde(default = "usize_max")]
	pub max_stream_duration:     usize,
	#[serde(default = "usize_max")]
	pub max_concurrent_stream_count: usize,
	#[serde(default = "usize_max")]
	pub max_total_stream_count:  usize,
	#[serde(default = "usize_max")]
	pub max_headers_count:       usize,
	#[serde(default = "usize_max")]
	pub max_headers_length:      usize,
	#[serde(default = "usize_max")]
	pub max_payload_length:      usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSocketHttp3 {
	#[serde(default = "usize_max")]
	pub buffer_size:          usize,
	#[serde(default = "usize_max")]
	pub timeout:              usize,
	#[serde(default = "usize_max")]
	pub max_header_list_size: usize,
	#[serde(default = "usize_max")]
	pub num_placeholders:     usize,
}

#[derive(Clone, Debug)]
pub enum StringMatcher {
	Ignore,
	Present(bool),
	Exact(String),
	Prefix(String),
	Suffix(String),
	Contains(String),
	#[cfg(feature = "regex")]
	Pattern(regex::Regex)
}

impl StringMatcher {
	pub fn matches(&self, s: Option<&str>) -> bool {
		match (self, s) {
			(Self::Ignore, _)            => true,
			(Self::Present(m), s)        => *m == s.is_some(),
			(Self::Exact(m), Some(s))    => m == s,
			(Self::Prefix(m), Some(s))   => s.starts_with(m),
			(Self::Suffix(m), Some(s))   => s.ends_with(m),
			(Self::Contains(m), Some(s)) => s.contains(m),
			#[cfg(feature = "regex")]
			(Self::Pattern(p), Some(s))  => p.is_match(s),
			_ => false
		}
	}
}

impl Default for StringMatcher {
	fn default() -> Self {
		Self::Ignore
	}
}

impl<'de> serde::Deserialize<'de> for StringMatcher {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = StringMatcher;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a string or a boolean")
			}

			fn visit_bool<E: serde::de::Error>(self, v: bool) -> std::result::Result<Self::Value, E> {
				Ok(StringMatcher::Present(v))
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
				#[cfg(feature = "regex")]
				if v.starts_with("###") && v.ends_with("###") {
					return Ok(StringMatcher::Pattern(regex::Regex::new(v
						.strip_prefix("###").unwrap()
						.strip_suffix("###").unwrap())
						.map_err(|e| E::custom(format!("expected a regular expression, but {}", e)))?));
				}

				Ok(match (v.starts_with('*'), v.ends_with('*')) {
					(true,  true)  => StringMatcher::Contains(v.strip_prefix('*').unwrap().strip_suffix('*').unwrap().to_string()),
					(true,  false) => StringMatcher::Suffix(v.strip_prefix('*').unwrap().to_string()),
					(false, true)  => StringMatcher::Prefix(v.strip_suffix('*').unwrap().to_string()),
					(false, false) => StringMatcher::Exact(v.to_string())
				})
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigRateLimits {
	#[serde(default = "usize_max")]
	pub max_concurrent_connections:          usize,
	#[serde(default = "usize_max")]
	pub max_concurrent_requests:             usize,
	#[serde(default = "usize_max")]
	pub max_concurrent_connections_per_addr: usize,
	#[serde(default = "usize_max")]
	pub max_concurrent_requests_per_addr:    usize,
	#[serde(default = "usize_max")]
	pub max_connections_per_addr:            usize,
	#[serde(default = "usize_max")]
	pub max_requests_per_addr:               usize,
	#[serde(default = "usize_max")]
	pub max_bytes_per_addr:                  usize,
	#[serde(default = "usize_max")]
	pub max_requests_per_connection:         usize,
	#[serde(default = "usize_max")]
	pub max_connection_lifetime:             usize,
	#[serde(default = "usize_max")]
	pub request_timeout:                     usize,
	#[serde(default = "usize_max")]
	pub connection_timeout:                  usize,
}

fn usize_max() -> usize {
	usize::MAX
}

fn usize_zero() -> usize {
	0
}

type HeaderId<T> = std::result::Result<T, String>;

fn http_parse_header_ids<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<HeaderId<net::http::HeaderId>>, D::Error> {
	Ok(std::collections::HashSet::<String>::deserialize(deserializer)?
		.into_iter()
		.map(|v| net::http::HeaderId::from_str(&v).map_err(|_| v.to_string()))
		.collect())
}

fn http_parse_headers<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<net::http::Header>, D::Error> {
	Ok(HashMap::<String, String>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| net::http::Header::parse_name_value(&k, &v))
		.collect())
}

fn http_parse_headers_match<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<(HeaderId<net::http::HeaderId>, StringMatcher)>, D::Error> {
	Ok(std::collections::HashMap::<String, StringMatcher>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| (net::http::HeaderId::from_str(&k).map_err(|_| k.to_string()), v))
		.collect())
}

fn rtsp_parse_header_ids<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<HeaderId<net::rtsp::HeaderId>>, D::Error> {
	Ok(std::collections::HashSet::<String>::deserialize(deserializer)?
		.into_iter()
		.map(|v| net::rtsp::HeaderId::from_str(&v).map_err(|_| v.to_string()))
		.collect())
}

fn rtsp_parse_headers<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<net::rtsp::Header>, D::Error> {
	Ok(HashMap::<String, String>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| net::rtsp::Header::parse_name_value(&k, &v))
		.collect())
}

fn rtsp_parse_headers_match<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<(HeaderId<net::rtsp::HeaderId>, StringMatcher)>, D::Error> {
	Ok(std::collections::HashMap::<String, StringMatcher>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| (net::rtsp::HeaderId::from_str(&k).map_err(|_| k.to_string()), v))
		.collect())
}

fn imf_parse_header_ids<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<HeaderId<net::imf::HeaderId>>, D::Error> {
	Ok(std::collections::HashSet::<String>::deserialize(deserializer)?
		.into_iter()
		.map(|v| net::imf::HeaderId::from_str(&v).map_err(|_| v.to_string()))
		.collect())
}

fn imf_parse_headers<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<net::imf::Header>, D::Error> {
	Ok(HashMap::<String, String>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| net::imf::Header::parse_name_value(&k, &v))
		.collect())
}

fn imf_parse_headers_match<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<(HeaderId<net::imf::HeaderId>, StringMatcher)>, D::Error> {
	Ok(HashMap::<String, StringMatcher>::deserialize(deserializer)?
		.into_iter()
		.map(|(k, v)| (net::imf::HeaderId::from_str(&k).map_err(|_| k.to_string()), v))
		.collect())
}

fn dns_parse_record_types<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Option<Vec<net::dns::Type>>, D::Error> {
	use serde::de::Error;
	match Option::<Vec<String>>::deserialize(deserializer)? {
		None => Ok(None),
		Some(v) => Ok(Some(v.into_iter()
			.map(|v| net::dns::Type::from_str(&v)
				.map_err(|_| D::Error::custom("cannot parse DNS record type")))
			.collect::<std::result::Result<Vec<_>, _>>()?))
	}
}