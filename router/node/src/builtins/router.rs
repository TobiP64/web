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

use smol::io::AsyncReadExt;
use {
	super::*,
	crate::{interfaces::*, utils::*},
	std::{io, pin::Pin, task::Poll, task::Context, path::*},
	net::{*, http::traits::AsyncStreamExt},
	smol::io::AsyncWriteExt
};

const DEFAULT_ALLOW: [http::Method; 2] = [http::Method::Head, http::Method::Get];

pub(super) async fn run(name: &str, cfg: Config) -> Result<()> {
	let module: HttpStreamHandler = Box::new(Module {
		filters: cfg.filters.into_iter()
			.enumerate()
			.filter_map(|(idx, cfg)| smol::block_on(Filter::from_cfg(name, idx, cfg)))
			.collect()
	});
	
	let id = crate::component_id(name);
	crate::add_component::<HttpStreamHandler>(id, module);
	Ok(())
}

struct Module {
	filters: Vec<Filter>
}

impl StreamHandler<dyn http::traits::AsyncStream> for Module {
	fn accept<'a>(&'a self, stream: &'static mut dyn http::traits::AsyncStream) -> DynFuture<'a, Result<()>> {
		Box::pin(async move {
			let headers = stream.read_headers().await?;
			
			// TODO match HTTP response
			for filter in &self.filters {
				if !filter.match_http_request_headers(&headers) {
					continue;
				}
				
				let (method, path) = (
					headers.iter().find_map(http::Header::as_method),
					headers.iter().find_map(http::Header::as_path)
				);
				
				let (method, path) = match (method, path) {
					(Some(v0), Some(v1)) => (v0, v1),
					_ => {
						discard_body(stream).await?;
						return http::MessageBuilder::new()
							.status(http::Status::BadRequest)
							.content_length(0)
							.send_async(stream)
							.await.map_err(Into::into)
					}
				};
				
				// allowed
				
				let allowed_methods = filter.http_headers.iter()
					.find_map(http::Header::as_allow)
					.map(Vec::as_slice)
					.unwrap_or_else(|| DEFAULT_ALLOW.as_slice());
				
				if !allowed_methods.contains(method) {
					discard_body(stream).await?;
					return http::MessageBuilder::new()
						.status(http::Status::MethodNotAllowed)
						.content_length(0)
						.allow(allowed_methods.to_vec())
						.send_async(stream)
						.await.map_err(Into::into);
				}
				
				// content negotiation
				
				/*if !filter.http_content_locations.is_empty() {
					let (accept_types, accept_encoding, accept_language, _accept_charset) = (
						headers.iter().find_map(http::Header::as_accept_types),
						headers.iter().find_map(http::Header::as_accept_encoding),
						headers.iter().find_map(http::Header::as_accept_language),
						headers.iter().find_map(http::Header::as_accept_charset),
					);
					
					match filter.content_locations.iter().find(|res| {
						let res = smol::block_on(res.data.read());
						let res = res.as_ref().unwrap();
						
						let res = match res.iter().find(|v| v.filter.matches(&headers)) {
							Some(v) => v,
							None => return false
						};
						
						match (accept_types, res.headers.iter().find_map(http::Header::as_content_type)) {
							(Some(accept), Some(actual)) => accept.contains(actual),
							_ => true
						} && match (accept_encoding, res.headers.iter().find_map(http::Header::as_content_encoding)) {
							(_, Some(http::Encoding::Identity)) => true,
							(Some(accept), Some(actual)) => accept.contains(actual),
							_ => true
						} && match (accept_language, res.headers.iter().find_map(http::Header::as_content_language)) {
							(Some(accept), Some(actual)) => accept.contains(actual),
							_ => true
						} /*&& match (accept_charset, res.headers.iter().find_map(http::Header::as_content_type)) {
								(Some(accept), Some(actual)) => accept.contains(actual),
								_ => true
							}*/
					}) {
						Some(v) => {
							// TODO
							//response_headers.push(http::Header::ContentLocation(resource.headers.path_ref().clone()));
							node_new = Some(v.clone());
							continue 'outer;
						}
						None => break http::Status::NotAcceptable
					}
				}*/
				
				// conditional requests
				
				let status = match (
					filter.http_headers.iter().find_map(http::Header::as_e_tag),
					filter.http_headers.iter().find_map(http::Header::as_last_modified),
					headers.iter().find_map(http::Header::as_if_none_match),
					headers.iter().find_map(http::Header::as_if_modified_since),
					headers.iter().find_map(http::Header::as_if_match),
					headers.iter().find_map(http::Header::as_if_unmodified_since)
				) {
					(Some(etag), _, Some(etags), ..) if etags.contains(&etag.value)        => Some(http::Status::NotModified),
					(_, Some(date), _, Some(since), ..) if date <= since                      => Some(http::Status::NotModified),
					(Some(etag), _, _, _, Some(etags), ..) if !etags.contains(&etag.value) => Some(http::Status::PreconditionFailed),
					(_, Some(date), _, _, _, Some(since)) if date > since                     => Some(http::Status::PreconditionFailed),
					_ => None
				};
				
				if let Some(status) = status {
					discard_body(stream).await?;
					return http::MessageBuilder::new()
						.status(status)
						.content_length(0)
						.send_async(stream)
						.await.map_err(Into::into);
				}
				
				match &filter.action {
					FilterAction::Forward(module) => {
						let mut stream = FilteredHttpStream {
							inner:            stream,
							filter,
							request_headers:  headers.clone(),
							response_headers: Vec::new()
						};
						
						// this is unsafe, but that's ok, see HttpStreamHandler::accept
						let stream = unsafe { std::mem::transmute::<_, &'static mut FilteredHttpStream<'static>>(&mut stream) };
						return module.accept(stream).await;
					}
					FilterAction::Reply(v)   => {
						discard_body(stream).await?;
						stream.write_headers(&filter.http_headers).await?;
						stream.write_all(v).await?;
						stream.close().await?;
						return Ok(());
					}
					FilterAction::Close      => return Ok(()),
					FilterAction::Abort      => return Err(io::Error::from(io::ErrorKind::ConnectionAborted).into())
				}
			}
			
			send_response(stream, http::Status::NotFound).await
		})
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub filters: Vec<ConfigFilter>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilter {
	pub name:   Option<String>,
	pub action: ConfigAction,
	/// Applies `action` if the filter does not match
	pub invert: bool,
	/// If true, any additional headers that are not matched by any rule
	/// cause the request to not match. The same applies for query parameters.
	pub exact:  bool,
	pub ip:     Option<ConfigFilterIp>,
	pub tls:    Option<ConfigFilterTls>,
	pub http:   Option<ConfigFilterHttp>,
	pub rtsp:   Option<ConfigFilterRtsp>,
	pub smtp:   Option<ConfigFilterSmtp>,
	pub imf:    Option<ConfigFilterImf>,
	pub dns:    Option<ConfigFilterDns>
}

#[derive(Clone, Debug)]
pub enum ConfigAction {
	Forward(String),
	Reply(String),
	Close,
	Abort
}

impl<'de> Deserialize<'de> for ConfigAction {
	fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
		use ::serde::de::Error;
		let v = HashMap::<String, serde_dyn_repr::Value>::deserialize(deserializer)?.into_iter()
			.collect::<Vec<_>>();
		
		match v.get(0).map(|(k, v)| (k.as_str(), v)) {
			Some(("forward", serde_dyn_repr::Value::String(v))) => Ok(Self::Forward(v.to_string())),
			Some(("reply",   serde_dyn_repr::Value::String(v))) => Ok(Self::Forward(v.to_string())),
			Some(("close",   _))                                => Ok(Self::Close),
			Some(("abort",   _))                                => Ok(Self::Abort),
			_ => Err(D::Error::custom("failed to deserialize ConfigAction"))
		}
	}
}

impl Default for ConfigAction {
	fn default() -> Self {
		Self::Abort
	}
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterIp {
	pub addr: Option<String>,
	pub port: Option<(u16, u16)>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterTls {
	pub hostname: StringMatcher,
	pub ca_certs: Option<Vec<String>>,
	pub alpn:     Option<Vec<String>>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterHttp {
	/// Headers used for built-in filters, like content negotiation
	#[serde(deserialize_with = "http_parse_headers")]
	pub headers:                 Vec<http::Header>,
	pub content_locations:       Vec<String>,
	pub path_match:              StringMatcher,
	pub path_strip_prefix:       Option<String>,
	pub path_add_prefix:         Option<String>,
	pub path_add_suffix:         Option<String>,
	pub query_match:             HashMap<String, StringMatcher>,
	pub query_add:               HashMap<String, String>,
	pub query_modify:            HashMap<String, String>,
	pub query_del:               Vec<String>,
	#[serde(deserialize_with = "http_parse_headers_match")]
	pub request_headers_match:   Vec<(HeaderId<http::HeaderId>, StringMatcher)>,
	#[serde(deserialize_with = "http_parse_headers")]
	pub request_headers_add:     Vec<http::Header>,
	#[serde(deserialize_with = "http_parse_header_ids")]
	pub request_headers_del:     Vec<HeaderId<http::HeaderId>>,
	#[serde(deserialize_with = "http_parse_headers")]
	pub request_headers_modify:  Vec<http::Header>,
	pub request_content_match:   StringMatcher,
	pub request_content_modify:  Option<String>,
	#[serde(deserialize_with = "http_parse_headers_match")]
	pub response_headers_match:  Vec<(HeaderId<http::HeaderId>, StringMatcher)>,
	#[serde(deserialize_with = "http_parse_headers")]
	pub response_headers_add:    Vec<http::Header>,
	#[serde(deserialize_with = "http_parse_header_ids")]
	pub response_headers_del:    Vec<HeaderId<http::HeaderId>>,
	#[serde(deserialize_with = "http_parse_headers")]
	pub response_headers_modify: Vec<http::Header>,
	pub response_content_match:  StringMatcher,
	pub response_content_modify: Option<String>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterRtsp {
	/// Headers used for built-in filters, like content negotiation
	#[serde(deserialize_with = "rtsp_parse_headers")]
	pub headers:                 Vec<rtsp::Header>,
	pub path_match:              StringMatcher,
	pub path_strip_prefix:       Option<String>,
	pub path_add_prefix:         Option<String>,
	pub path_add_suffix:         Option<String>,
	pub query_match:             HashMap<String, StringMatcher>,
	pub query_add:               HashMap<String, String>,
	pub query_modify:            HashMap<String, String>,
	pub query_del:               Vec<String>,
	#[serde(deserialize_with = "rtsp_parse_headers_match")]
	pub request_headers_match:   Vec<(HeaderId<rtsp::HeaderId>, StringMatcher)>,
	#[serde(deserialize_with = "rtsp_parse_headers")]
	pub request_headers_add:     Vec<rtsp::Header>,
	#[serde(deserialize_with = "rtsp_parse_header_ids")]
	pub request_headers_del:     Vec<HeaderId<rtsp::HeaderId>>,
	#[serde(deserialize_with = "rtsp_parse_headers")]
	pub request_headers_modify:  Vec<rtsp::Header>,
	pub request_content_match:   StringMatcher,
	pub request_content_modify:  Option<String>,
	#[serde(deserialize_with = "rtsp_parse_headers_match")]
	pub response_headers_match:  Vec<(HeaderId<rtsp::HeaderId>, StringMatcher)>,
	#[serde(deserialize_with = "rtsp_parse_headers")]
	pub response_headers_add:    Vec<rtsp::Header>,
	#[serde(deserialize_with = "rtsp_parse_header_ids")]
	pub response_headers_del:    Vec<HeaderId<rtsp::HeaderId>>,
	#[serde(deserialize_with = "rtsp_parse_headers")]
	pub response_headers_modify: Vec<rtsp::Header>,
	pub response_content_match:  StringMatcher,
	pub response_content_modify: Option<String>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterImf {
	#[serde(deserialize_with = "imf_parse_headers_match")]
	pub request_headers_match:   Vec<(HeaderId<imf::HeaderId>, StringMatcher)>,
	#[serde(deserialize_with = "imf_parse_headers")]
	pub request_headers_add:     Vec<imf::Header>,
	#[serde(deserialize_with = "imf_parse_header_ids")]
	pub request_headers_del:     Vec<HeaderId<imf::HeaderId>>,
	#[serde(deserialize_with = "imf_parse_headers")]
	pub request_headers_modify:  Vec<imf::Header>,
	pub request_content_match:   StringMatcher,
	pub request_content_modify:  Option<String>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterSmtp {
	pub sender:        StringMatcher,
	pub recipient_any: StringMatcher,
	pub recipient_all: StringMatcher,
	pub recipient_add: String,
	pub recipient_del: String
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFilterDns {
	pub domain:      StringMatcher,
	#[serde(deserialize_with = "dns_parse_record_types")]
	pub record_type: Option<Vec<net::dns::Type>>
}

#[derive(Clone)]
enum FilterAction {
	Forward(ComponentRef<HttpStreamHandler>),
	Reply(Box<[u8]>),
	Close,
	Abort
}

impl Default for FilterAction {
	fn default() -> Self {
		Self::Abort
	}
}

#[derive(Clone, Default)]
struct Filter {
	name:                         String,
	action:                       FilterAction,
	match_invert:                 bool,
	match_exact:                  bool,
	ip_addr:                      u128,
	ip_mask:                      u128,
	ports:                        (u16, u16),
	tls_hostname:                 StringMatcher,
	tls_ca_certs:                 Option<Vec<Box<[u8]>>>,
	tls_alpn:                     Option<Vec<String>>,
	http_headers:                 Vec<http::Header>,
	http_content_locations:       Vec<String>,
	http_path_match:              StringMatcher,
	http_path_strip_prefix:       Option<Box<Path>>,
	http_path_add_prefix:         Option<Box<Path>>,
	http_path_add_suffix:         Option<Box<Path>>,
	http_query_match:             HashMap<String, StringMatcher>,
	http_query_add:               HashMap<String, String>,
	http_query_modify:            HashMap<String, String>,
	http_query_del:               Vec<String>,
	http_request_headers_match:   Vec<(HeaderId<http::HeaderId>, StringMatcher)>,
	http_request_headers_add:     Vec<http::Header>,
	http_request_headers_del:     Vec<HeaderId<http::HeaderId>>,
	http_request_headers_modify:  Vec<http::Header>,
	http_request_content_match:   StringMatcher,
	http_request_content_modify:  Option<String>,
	http_response_headers_match:  Vec<(HeaderId<http::HeaderId>, StringMatcher)>,
	http_response_headers_add:    Vec<http::Header>,
	http_response_headers_del:    Vec<HeaderId<http::HeaderId>>,
	http_response_headers_modify: Vec<http::Header>,
	http_response_content_match:  StringMatcher,
	http_response_content_modify: Option<String>,
	rtsp_headers:                 Vec<rtsp::Header>,
	rtsp_path_match:              StringMatcher,
	rtsp_path_strip_prefix:       Option<Box<Path>>,
	rtsp_path_add_prefix:         Option<Box<Path>>,
	rtsp_path_add_suffix:         Option<Box<Path>>,
	rtsp_query_match:             HashMap<String, StringMatcher>,
	rtsp_query_add:               HashMap<String, String>,
	rtsp_query_modify:            HashMap<String, String>,
	rtsp_query_del:               Vec<String>,
	rtsp_request_headers_match:   Vec<(HeaderId<rtsp::HeaderId>, StringMatcher)>,
	rtsp_request_headers_add:     Vec<rtsp::Header>,
	rtsp_request_headers_del:     Vec<HeaderId<rtsp::HeaderId>>,
	rtsp_request_headers_modify:  Vec<rtsp::Header>,
	rtsp_request_content_match:   StringMatcher,
	rtsp_request_content_modify:  Option<String>,
	rtsp_response_headers_match:  Vec<(HeaderId<rtsp::HeaderId>, StringMatcher)>,
	rtsp_response_headers_add:    Vec<rtsp::Header>,
	rtsp_response_headers_del:    Vec<HeaderId<rtsp::HeaderId>>,
	rtsp_response_headers_modify: Vec<rtsp::Header>,
	rtsp_response_content_match:  StringMatcher,
	rtsp_response_content_modify: Option<String>,
	imf_request_headers_match:    Vec<(HeaderId<imf::HeaderId>, StringMatcher)>,
	imf_request_headers_add:      Vec<imf::Header>,
	imf_request_headers_del:      Vec<HeaderId<imf::HeaderId>>,
	imf_request_headers_modify:   Vec<imf::Header>,
	imf_request_content_match:    StringMatcher,
	imf_request_content_modify:   Option<String>,
	smtp_sender:                  StringMatcher,
	smtp_recipient_any:           StringMatcher,
	smtp_recipient_all:           StringMatcher,
	smtp_recipient_add:           Vec<String>,
	smtp_recipient_del:           Vec<String>,
	dns_domain:                   StringMatcher,
	dns_record_type:              Option<Vec<net::dns::Type>>
}

impl Filter {
	fn sorted<T: std::cmp::Ord>(mut vec: Vec<T>) -> Vec<T> {
		vec.sort();
		vec
	}
	
	fn sorted_by<T, F: FnMut(&T) -> K, K: std::cmp::Ord>(mut vec: Vec<T>, f: F) -> Vec<T> {
		vec.sort_by_key(f);
		vec
	}
	
	#[allow(clippy::field_reassign_with_default)]
	async fn from_cfg(name: &str, idx: usize, cfg: ConfigFilter) -> Option<Self> {
		let mut self_ = Self::default();
		self_.name = cfg.name.unwrap_or_else(|| format!("#{:03}", idx));
		self_.action = match cfg.action {
			ConfigAction::Forward(v) => FilterAction::Forward(
				crate::get_component::<HttpStreamHandler>(crate::component_id(&v))),
			ConfigAction::Reply(v)   => FilterAction::Reply(v.into_boxed_str().into_boxed_bytes()),
			ConfigAction::Close      => FilterAction::Close,
			ConfigAction::Abort      => FilterAction::Abort
		};
		self_.match_invert = cfg.invert;
		self_.match_exact  = cfg.exact;
		
		if let Some(cfg) = cfg.ip {
			if let Some(addr) = cfg.addr {
				let (ip, mask) = addr.split_once('/').unwrap_or((&addr, "128"));
				
				self_.ip_addr = match std::net::IpAddr::from_str(ip) {
					Ok(v)  => match v {
						std::net::IpAddr::V4(v) => (u32::from_be_bytes(v.octets()) as u128) << 96,
						std::net::IpAddr::V6(v) => u128::from_be_bytes(v.octets())
					},
					Err(_) => {
						log::error!("processor `{}` filter `{}`: `ip.addr` is not a valid IP address", name, &self_.name);
						return None;
					}
				};
				self_.ip_mask = match usize::from_str(ip) {
					Ok(v)  => !0 << (128 - v),
					Err(_) => {
						log::error!("processor `{}` filter `{}`: `ip.addr` is not a valid IP address", name, &self_.name);
						return None;
					}
				};
			} else {
				self_.ip_mask = !0;
			}
			
			self_.ports = cfg.port.unwrap_or((0, u16::MAX));
		}
		
		if let Some(cfg) = cfg.tls {
			self_.tls_hostname = cfg.hostname;
			self_.tls_ca_certs = match cfg.ca_certs {
				Some(paths) => {
					let mut certs = Vec::new();
					for path in paths {
						match smol::fs::read(path).await {
							Ok(v)  => certs.push(v.into_boxed_slice()),
							Err(e) => log::warn!("processor `{}` filter `{}`: `tls.ca_certs` cannot read file: {}", name, &self_.name, e)
						}
					}
					
					Some(certs)
				},
				None => None
			};
			self_.tls_alpn = cfg.alpn;
		}
		
		if let Some(cfg) = cfg.http {
			self_.http_headers                 = Self::sorted_by(cfg.headers, |v| v.id().map_err(str::to_string));
			self_.http_content_locations       = cfg.content_locations;
			self_.http_path_match              = cfg.path_match;
			self_.http_path_strip_prefix       = cfg.path_strip_prefix.map(|v| PathBuf::from(v).into_boxed_path());
			self_.http_path_add_prefix         = cfg.path_add_prefix.map(|v| PathBuf::from(v).into_boxed_path());
			self_.http_path_add_suffix         = cfg.path_add_suffix.map(|v| PathBuf::from(v).into_boxed_path());
			self_.http_query_match             = cfg.query_match;
			self_.http_query_add               = cfg.query_add;
			self_.http_query_modify            = cfg.query_modify;
			self_.http_query_del               = cfg.query_del;
			self_.http_request_headers_match   = cfg.request_headers_match;
			self_.http_request_headers_add     = cfg.request_headers_add;
			self_.http_request_headers_del     = Self::sorted(cfg.request_headers_del);
			self_.http_request_headers_modify  = cfg.request_headers_modify;
			self_.http_request_content_match   = cfg.request_content_match;
			self_.http_request_content_modify  = cfg.request_content_modify;
			self_.http_response_headers_match  = cfg.response_headers_match;
			self_.http_response_headers_add    = cfg.response_headers_add;
			self_.http_response_headers_del    = Self::sorted(cfg.response_headers_del);
			self_.http_response_headers_modify = cfg.response_headers_modify;
			self_.http_response_content_match  = cfg.response_content_match;
			self_.http_response_content_modify = cfg.response_content_modify;
		}
		
		if let Some(cfg) = cfg.rtsp {
		
		}
		
		if let Some(cfg) = cfg.smtp {
		
		}
		
		if let Some(cfg) = cfg.imf {
		
		}
		
		Some(self_)
	}
	
	fn match_http_request_headers<'a>(&self, headers: impl IntoIterator<Item = &'a http::Header>) -> bool {
		if matches!(self.http_path_match, StringMatcher::Ignore) && self.http_query_match.is_empty() && self.http_request_headers_match.is_empty() {
			return true;
		}
		
		for header in headers {
			if let http::Header::Path(s) = header {
				if !self.http_path_match.matches(Some(s)) {
					return false;
				}
				
				if !self.http_query_match.is_empty() {
					let i = match s.find('?') {
						Some(v) => v,
						_ if self.match_exact => return false,
						_ => continue
					};
					
					for param in s[i + 1..].split('&') {
						let (key, val) = match param.split_once('=') {
							Some((key, val)) => (key, Some(val)),
							None => (param, None)
						};
						
						match self.http_query_match.get(key) {
							Some(matcher) if matcher.matches(val) => (),
							_ if self.match_exact => return false,
							_ => ()
						}
					}
				}
			}
			
			match self.http_request_headers_match.binary_search_by_key(&header.id(), |(k, _)| k.as_ref().map_err(String::as_str).map(|v| *v)) {
				Ok(i) if self.http_request_headers_match[i].1.matches(Some(&header.to_string())) => continue,
				Err(_) if self.match_exact => return false,
				_ => continue
			}
		}
		
		true
	}
	
	fn match_http_request_content(&self, content: Option<&str>) -> bool {
		self.http_request_content_match.matches(content)
	}
	
	fn filter_http_request_headers<'a>(&'a self, headers: impl IntoIterator<Item = http::Header> + 'a) -> impl Iterator<Item = http::Header> + 'a {
		headers.into_iter()
			.map(|h| match h {
				http::Header::Path(path) => {
					let mut buf = std::path::PathBuf::from(path);
					
					if let Some(prefix) = &self.http_path_strip_prefix {
						let path = match buf.strip_prefix(prefix) {
							Ok(v)  => v,
							Err(_) => &buf
						};
						
						let mut root = PathBuf::from("/");
						root.push(path);
						buf = root;
					}
					
					if let Some(prefix) = &self.http_path_add_prefix {
						let mut root = prefix.to_path_buf();
						root.push(buf);
						buf = root;
					}
					
					if let Some(suffix) = &self.http_path_add_suffix {
						buf.push(suffix);
					}
					
					http::Header::Path(buf.to_string_lossy().into_owned())
				}
				v => v
			})
			.map(move |h| match self.http_request_headers_modify.binary_search_by_key(&h.id(), |h| h.id()) {
				Ok(i) => self.http_request_headers_modify[i].clone(),
				Err(_) => h
			})
			.filter(move |h| self.http_request_headers_del.binary_search_by_key(
				&h.id(), |k| k.as_ref().map_err(String::as_str).map(|v| *v)).is_err())
			.chain(self.http_request_headers_add.iter().cloned())
	}
	
	fn match_http_response_headers<'a>(&self, headers: impl IntoIterator<Item = &'a http::Header>) -> bool {
		if self.http_response_headers_match.is_empty() {
			return true;
		}
		
		for header in headers {
			match self.http_response_headers_match.binary_search_by_key(&header.id(), |(k, _)| k.as_ref().map_err(String::as_str).map(|v| *v)) {
				Ok(i) if self.http_response_headers_match[i].1.matches(Some(&header.to_string())) => continue,
				Err(_) if self.match_exact => return false,
				_ => continue
			}
		}
		
		true
	}
	
	fn match_http_response_content(&self, content: Option<&str>) -> bool {
		self.http_response_content_match.matches(content)
	}
	
	fn filter_http_response_headers<'a>(&'a self, headers: impl IntoIterator<Item = &'a http::Header> + 'a) -> impl Iterator<Item = &'a http::Header> + 'a {
		headers.into_iter()
			.map(move |h| match self.http_response_headers_modify.binary_search_by_key(&h.id(), |h| h.id()) {
				Ok(i) => &self.http_response_headers_modify[i],
				Err(_) => h
			})
			.filter(move |h| self.http_response_headers_del.binary_search_by_key(
				&h.id(), |k| k.as_ref().map_err(String::as_str).map(|v| *v)).is_err())
			.chain(self.http_response_headers_add.iter())
	}
}

async fn discard_body(stream: &mut dyn http::traits::AsyncStream) -> io::Result<()> {
	let mut buf = Vec::new();
	stream.read_to_end(&mut buf).await?;
	Ok(())
}

// TODO implement content filtering
struct FilteredHttpStream<'a> {
	inner:            &'a mut dyn http::traits::AsyncStream,
	filter:           &'a Filter,
	request_headers:  Vec<http::Header>,
	response_headers: Vec<http::Header>
}

impl<'a> smol::io::AsyncRead for FilteredHttpStream<'a> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_read(cx, buf)
	}
	
	fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>, bufs: &mut [io::IoSliceMut<'_>]) -> Poll<io::Result<usize>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_read_vectored(cx, bufs)
	}
}

impl<'a> smol::io::AsyncWrite for FilteredHttpStream<'a> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_write(cx, buf)
	}
	
	fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>, bufs: &[io::IoSlice<'_>]) -> Poll<io::Result<usize>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_write_vectored(cx, bufs)
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<io::Result<()>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_flush(cx)
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<io::Result<()>> {
		unsafe { Pin::map_unchecked_mut(self, |v| v.inner) }.poll_close(cx)
	}
}

impl<'a> http::traits::AsyncStream for FilteredHttpStream<'a> {
	fn poll_read_headers(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Vec<http::Header>>> {
		let Self { inner, filter, request_headers, .. } = unsafe { Pin::into_inner_unchecked(self) };
		
		if request_headers.is_empty() {
			return Poll::Ready(Ok(Vec::new()));
		}
		
		Poll::Ready(Ok(filter
			.filter_http_request_headers(std::mem::take(request_headers))
			.collect()))
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[http::Header]) -> Poll<io::Result<()>> {
		let Self { inner, filter, response_headers, .. } = unsafe { Pin::into_inner_unchecked(self) };
		
		// on the first call the headers must be filtered
		if response_headers.is_empty() {
			response_headers.extend(filter.filter_http_response_headers(headers.iter()).cloned());
		}
		
		match unsafe { Pin::new_unchecked(&mut**inner) }.poll_write_headers(cx, response_headers) {
			Poll::Ready(Ok(())) => {
				response_headers.clear();
				Poll::Ready(Ok(()))
			},
			Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
			Poll::Pending       => Poll::Pending
		}
	}
}