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
	super::traits,
	std::{
		str::FromStr,
		fmt::{Formatter, Display, Result as FmtResult},
		convert::TryFrom
	},
	chrono::{DateTime, Utc}
};

pub use crate::utils::headers::*;

static NAMES_HTTP1: [&str; 62] = [
	"Accept-Charset",
	"Accept-Encoding",
	"Accept-Language",
	"Accept-Ranges",
	"Accept",
	"Access-Control-Allow-Credentials",
	"Access-Control-Allow-Headers",
	"Access-Control-Allow-Methods",
	"Access-Control-Allow-Origin",
	"Access-Control-Allow-Expose-Headers",
	"Access-Control-Allow-Max-Age",
	"Access-Control-Allow-Request-Headers",
	"Access-Control-Allow-Method",
	"Allow",
	":Authority",
	"Authorization",
	"Cache-Control",
	"Clear-Site-Data",
	"Connection",
	"Content-Disposition",
	"Content-Encoding",
	"Content-Language",
	"Content-Length",
	"Content-Location",
	"Content-Range",
	"Content-Type",
	"Cookie",
	"DNT",
	"Date",
	"ETag",
	"Expires",
	"Host",
	"If-Match",
	"If-Modified-Since",
	"If-None-Match",
	"If-Range",
	"If-Unmodified-Since",
	"Keep-Alive",
	"Last-Modified",
	"Location",
	":Method",
	":Path",
	"Proxy-Authenticate",
	"Proxy-Authorization",
	"Range",
	"Referer",
	"Retry-After",
	"Save-Data",
	":Scheme",
	"Sec-Web-Socket-Protocol",
	"Sec-Web-Socket-Extensions",
	"Sec-Web-Socket-Version",
	"Sec-Web-Socket-Accept",
	"Sec-Web-Socket-Key",
	"Server",
	"Set-Cookie",
	"Status",
	"Upgrade",
	"Upgrade-Insecure-Requests",
	"User-Agent",
	"Vary",
	"WWW-Authenticate"
];

static NAMES_HTTP2: [&str; 62] = [
	"accept-charset",
	"accept-encoding",
	"accept-language",
	"accept-ranges",
	"accept",
	"access-control-allow-credentials",
	"access-control-allow-headers",
	"access-control-allow-methods",
	"access-control-allow-origin",
	"access-control-allow-expose-headers",
	"access-control-allow-max-age",
	"access-control-allow-request-headers",
	"access-control-allow-method",
	"allow",
	":authority",
	"authorization",
	"cache-control",
	"clear-site-data",
	"connection",
	"content-disposition",
	"content-encoding",
	"content-language",
	"content-length",
	"content-location",
	"content-range",
	"content-type",
	"cookie",
	"dnt",
	"date",
	"etag",
	"expires",
	"host",
	"if-match",
	"if-modified-since",
	"if-none-match",
	"if-range",
	"if-unmodified-since",
	"keep-alive",
	"last-modified",
	"location",
	":method",
	":path",
	"proxy-authenticate",
	"proxy-authorization",
	"range",
	"referer",
	"retry-after",
	"save-data",
	":scheme",
	"sec-web-socket-protocol",
	"sec-web-socket-extensions",
	"sec-web-socket-version",
	"sec-web-socket-accept",
	"sec-web-socket-key",
	"server",
	"set-cookie",
	"status",
	"upgrade",
	"upgrade-insecure-requests",
	"user-agent",
	"vary",
	"www-authenticate"
];

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum HeaderId {
	AcceptCharset,
	AcceptEncoding,
	AcceptLanguage,
	AcceptRanges,
	AcceptTypes,
	AccessControlAllowCredentials,
	AccessControlAllowHeaders,
	AccessControlAllowMethods,
	AccessControlAllowOrigin,
	AccessControlAllowExposeHeaders,
	AccessControlAllowMaxAge,
	AccessControlAllowRequestHeaders,
	AccessControlAllowRequestMethod,
	Allow,
	Authority,
	Authorization,
	CacheControl,
	ClearSiteData,
	Connection,
	ContentDisposition,
	ContentEncoding,
	ContentLanguage,
	ContentLength,
	ContentLocation,
	ContentRange,
	ContentType,
	Cookie,
	Dnt,
	Date,
	ETag,
	Expires,
	Host,
	IfMatch,
	IfModifiedSince,
	IfNoneMatch,
	IfRange,
	IfUnmodifiedSince,
	KeepAlive,
	LastModified,
	Location,
	Method,
	Path,
	ProxyAuthenticate,
	ProxyAuthorization,
	Range,
	Referer,
	RetryAfter,
	SaveData,
	Scheme,
	SecWebSocketProtocol,
	SecWebSocketExtensions,
	SecWebSocketVersion,
	SecWebSocketAccept,
	SecWebSocketKey,
	Server,
	SetCookie,
	Status,
	Upgrade,
	UpgradeInsecureRequests,
	UserAgent,
	Vary,
	WwwAuthenticate
}

impl HeaderId {
	#[inline]
	pub fn parse(self, value: &str) -> Result<Header, ()> {
		Header::parse_id_value(self, value)
	}

	pub fn name_v1(self) -> &'static str {
		NAMES_HTTP1[self as usize]
	}

	pub fn name_v2(self) -> &'static str {
		NAMES_HTTP2[self as usize]
	}
}

impl FromStr for HeaderId {
	type Err = ();

	#[allow(unreachable_patterns)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Accept-Charset"                         | "accept-charset"                       => HeaderId::AcceptCharset,
			"Accept-Encoding"                        | "accept-encoding"                      => HeaderId::AcceptEncoding,
			"Accept-Language"                        | "accept-language"                      => HeaderId::AcceptLanguage,
			"Accept-Ranges"                          | "accept-ranges"                        => HeaderId::AcceptRanges,
			"Accept"                                 | "accept"                               => HeaderId::AcceptTypes,
			"Access-Control-Allow-Credentials"       | "access-control-allow-credentials"     => HeaderId::AccessControlAllowCredentials,
			"Access-Control-Allow-Headers"           | "access-control-allow-headers"         => HeaderId::AccessControlAllowHeaders,
			"Access-Control-Allow-Methods"           | "access-control-allow-methods"         => HeaderId::AccessControlAllowMethods,
			"Access-Control-Allow-Origin"            | "access-control-allow-origin"          => HeaderId::AccessControlAllowOrigin,
			"Access-Control-Allow-Expose-Headers"    | "access-control-allow-expose-headers"  => HeaderId::AccessControlAllowExposeHeaders,
			"Access-Control-Allow-Max-Age"           | "access-control-allow-max-age"         => HeaderId::AccessControlAllowMaxAge,
			"Access-Control-Allow-Request-Headers"   | "access-control-allow-request-headers" => HeaderId::AccessControlAllowRequestHeaders,
			"Access-Control-Allow-Method"            | "access-control-allow-method"          => HeaderId::AccessControlAllowRequestMethod,
			"Allow"                                  | "allow"                                => HeaderId::Allow,
			                                           ":authority"                           => HeaderId::Authority,
			"Authorization"                          | "authorization"                        => HeaderId::Authorization,
			"Cache-Control"                          | "cache-control"                        => HeaderId::CacheControl,
			"Clear-Site-Data"                        | "clear-site-data"                      => HeaderId::ClearSiteData,
			"Connection"                             | "connection"                           => HeaderId::Connection,
			"Content-Disposition"                    | "content-disposition"                  => HeaderId::ContentDisposition,
			"Content-Encoding"                       | "content-encoding"                     => HeaderId::ContentEncoding,
			"Content-Language"                       | "content-language"                     => HeaderId::ContentLanguage,
			"Content-Length"                         | "content-length"                       => HeaderId::ContentLength,
			"Content-Location"                       | "content-location"                     => HeaderId::ContentLocation,
			"Content-Range"                          | "content-range"                        => HeaderId::ContentRange,
			"Content-Type"                           | "content-type"                         => HeaderId::ContentType,
			"Cookie"                                 | "cookie"                               => HeaderId::Cookie,
			"DNT"                                    | "dnt"                                  => HeaderId::Dnt,
			"Date"                                   | "date"                                 => HeaderId::Date,
			"ETag"                                   | "etag"                                 => HeaderId::ETag,
			"Expires"                                | "expires"                              => HeaderId::Expires,
			"Host"                                   | "host"                                 => HeaderId::Host,
			"If-Match"                               | "if-match"                             => HeaderId::IfMatch,
			"If-Modified-Since"                      | "if-modified-since"                    => HeaderId::IfModifiedSince,
			"If-None-Match"                          | "if-none-match"                        => HeaderId::IfNoneMatch,
			"If-Range"                               | "if-range"                             => HeaderId::IfRange,
			"If-Unmodified-Since"                    | "if-unmodified-since"                  => HeaderId::IfUnmodifiedSince,
			"Keep-Alive"                             | "keep-alive"                           => HeaderId::KeepAlive,
			"Last-Modified"                          | "last-modified"                        => HeaderId::LastModified,
			"Location"                               | "location"                             => HeaderId::Location,
			                                           ":method"                              => HeaderId::Method,
			                                           ":path"                                => HeaderId::Path,
			"Proxy-Authenticate"                     | "proxy-authenticate"                   => HeaderId::ProxyAuthenticate,
			"Proxy-Authorization"                    | "proxy-authorization"                  => HeaderId::ProxyAuthorization,
			"Range"                                  | "range"                                => HeaderId::Range,
			"Referer"                                | "referer"                              => HeaderId::Referer,
			"Retry-After"                            | "retry-after"                          => HeaderId::RetryAfter,
			"Save-Data"                              | "save-data"                            => HeaderId::SaveData,
			                                           ":scheme"                              => HeaderId::Scheme,
			"sec-web-socket-protocol"                | "Sec-Web-Socket-Protocol"              => HeaderId::SecWebSocketProtocol,
			"sec-web-socket-extensions"              | "Sec-Web-Socket-Extensions"            => HeaderId::SecWebSocketExtensions,
			"sec-web-socket-version"                 | "Sec-Web-Socket-Version"               => HeaderId::SecWebSocketVersion,
			"sec-web-socket-accept"                  | "Sec-Web-Socket-Accept"                => HeaderId::SecWebSocketAccept,
			"sec-web-socket-key"                     | "Sec-Web-Socket-Key"                   => HeaderId::SecWebSocketKey,
			"Server"                                 | "server"                               => HeaderId::Server,
			"Set-Cookie"                             | "set-cookie"                           => HeaderId::SetCookie,
			"Status"                                 | "status"                               => HeaderId::Status,
			"Upgrade"                                | "upgrade"                              => HeaderId::Upgrade,
			"Upgrade-Insecure-Requests"              | "upgrade-insecure-requests"            => HeaderId::UpgradeInsecureRequests,
			"User-Agent"                             | "user-agent"                           => HeaderId::UserAgent,
			"Vary"                                   | "vary"                                 => HeaderId::Vary,
			"WWW-Authenticate"                       | "www-authenticate"                     => HeaderId::WwwAuthenticate,
			_                                                                                 => return Err(())
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum Header {
	AcceptCharset(Vec<Charset>),
	AcceptEncoding(Vec<Encoding>),
	AcceptLanguage(Vec<Language>),
	AcceptRanges(AcceptRanges),
	AcceptTypes(Vec<MediaType>),
	AccessControlAllowCredentials(bool),
	AccessControlAllowHeaders(Vec<HeaderId>),
	AccessControlAllowMethods(Vec<Method>),
	AccessControlAllowOrigin(String),
	AccessControlAllowExposeHeaders(Vec<HeaderId>),
	AccessControlAllowMaxAge(usize),
	AccessControlAllowRequestHeaders(Vec<HeaderId>),
	AccessControlAllowRequestMethod(Method),
	Allow(Vec<Method>),
	Authority(String),
	Authorization(Authorization),
	CacheControl(CacheControl),
	ClearSiteData(ClearSiteDataDirective),
	Connection(Connection),
	ContentDisposition(String),
	ContentEncoding(Encoding),
	ContentLanguage(Language),
	ContentLength(usize),
	ContentLocation(String),
	ContentRange(ContentRange),
	ContentType(Box<MediaType>),
	Cookie(Vec<(String, String)>),
	Dnt(bool),
	Date(DateTime<Utc>),
	ETag(ETag),
	Expires(DateTime<Utc>),
	Host(String),
	IfMatch(Vec<String>),
	IfModifiedSince(DateTime<Utc>),
	IfNoneMatch(Vec<String>),
	IfRange(()),
	IfUnmodifiedSince(DateTime<Utc>),
	KeepAlive(KeepAlive),
	LastModified(DateTime<Utc>),
	Location(String),
	Method(Method),
	Path(String),
	ProxyAuthenticate(String),
	ProxyAuthorization(String),
	Range(Ranges),
	Referer(String),
	RetryAfter(RetryAfter),
	SaveData(bool),
	Scheme(Scheme),
	SecWebSocketProtocol(Vec<String>),
	SecWebSocketExtensions(Vec<String>),
	SecWebSocketVersion(Vec<usize>),
	SecWebSocketAccept(String),
	SecWebSocketKey(String),
	Server(String),
	SetCookie(SetCookie),
	Status(Status),
	Upgrade(String),
	UpgradeInsecureRequests(bool),
	UserAgent(String),
	Vary(Vec<HeaderId>),
	WwwAuthenticate(String),
	Custom(String, String)
}

// REGEX:
//
/*

SELECT

\s*([a-zA-Z]*)\((.*)\),

FUNCTIONS

pub fn as_$1(&self) -> Option<&$2> {
	match self {
		Self::$1(v) => Some(v),
		_ => None
	}
}

pub fn as_mut_$1(&mut self) -> Option<&mut $2> {
	match self {
		Self::$1(v) => Some(v),
		_ => None
	}
}

pub fn into_$1(self) -> Option<$2> {
	match self {
		Self::$1(v) => Some(v),
		_ => None
	}
}

NAMES TO SNAKE CASE:

fn ([a-z_]*)([A-Z])(.*)\(

fn $1_\L$2\E$3(

 */
impl Header {
	pub fn parse_name_value(name: &str, value: &str) -> Self {
		match name.parse::<HeaderId>().and_then(|id| Self::parse_id_value(id, value)) {
			Ok(v) => v,
			Err(_) => Self::Custom(name.to_string(), value.to_string())
		}
	}

	pub fn parse_id_value(id: HeaderId, v: &str) -> Result<Self, ()> {
		Ok(match id {
			HeaderId::AcceptCharset                    => Self::AcceptCharset(parse_list::<Charset>(v)?),
			HeaderId::AcceptEncoding                   => Self::AcceptEncoding(parse_list::<Encoding>(v)?),
			HeaderId::AcceptLanguage                   => Self::AcceptLanguage(parse_list::<Language>(v)?),
			HeaderId::AcceptRanges                     => Self::AcceptRanges(v.parse()?),
			HeaderId::AcceptTypes                      => Self::AcceptTypes(parse_list::<MediaType>(v)?),
			HeaderId::AccessControlAllowCredentials    => Self::AccessControlAllowCredentials(true),
			HeaderId::AccessControlAllowHeaders        => Self::AccessControlAllowHeaders(parse_list::<HeaderId>(v)?),
			HeaderId::AccessControlAllowMethods        => Self::AccessControlAllowMethods(parse_list::<Method>(v)?),
			HeaderId::AccessControlAllowOrigin         => Self::AccessControlAllowOrigin(v.to_string()),
			HeaderId::AccessControlAllowExposeHeaders  => Self::AccessControlAllowExposeHeaders(parse_list::<HeaderId>(v)?),
			HeaderId::AccessControlAllowMaxAge         => Self::AccessControlAllowMaxAge(v.parse().map_err(|_| ())?),
			HeaderId::AccessControlAllowRequestHeaders => Self::AccessControlAllowRequestHeaders(parse_list::<HeaderId>(v)?),
			HeaderId::AccessControlAllowRequestMethod  => Self::AccessControlAllowRequestMethod(v.parse()?),
			HeaderId::Allow                            => Self::Allow(parse_list::<Method>(v)?),
			HeaderId::Authority                        => Self::Authority(v.to_string()),
			HeaderId::Authorization                    => Self::Authorization(v.parse()?),
			HeaderId::CacheControl                     => Self::CacheControl(v.parse()?),
			HeaderId::ClearSiteData                    => Self::ClearSiteData(v.parse()?),
			HeaderId::Connection                       => Self::Connection(v.parse()?),
			HeaderId::ContentDisposition               => Self::ContentDisposition(v.to_string()),
			HeaderId::ContentEncoding                  => Self::ContentEncoding(v.parse()?),
			HeaderId::ContentLanguage                  => Self::ContentLanguage(v.parse()?),
			HeaderId::ContentLength                    => Self::ContentLength(v.parse().map_err(|_| ())?),
			HeaderId::ContentLocation                  => Self::ContentLocation(v.to_string()),
			HeaderId::ContentRange                     => Self::ContentRange(v.parse()?),
			HeaderId::ContentType                      => Self::ContentType(Box::new(v.parse()?)),
			HeaderId::Cookie                           => Self::Cookie(parse_map(v)),
			HeaderId::Dnt                              => Self::Dnt(v == "1"),
			HeaderId::Date                             => Self::Date(parse_date(v)?),
			HeaderId::ETag                             => Self::ETag(v.parse()?),
			HeaderId::Expires                          => Self::Expires(parse_date(v)?),
			HeaderId::Host                             => Self::Host(v.to_string()),
			HeaderId::IfMatch                          => Self::IfMatch(parse_list::<String>(v)?),
			HeaderId::IfModifiedSince                  => Self::IfModifiedSince(parse_date(v)?),
			HeaderId::IfNoneMatch                      => Self::IfNoneMatch(v.split(',')
				.map(|s| s.trim().trim_matches('"').to_string())
				.collect()),
			HeaderId::IfRange                          => Self::IfRange(()),
			HeaderId::IfUnmodifiedSince                => Self::IfUnmodifiedSince(parse_date(v)?),
			HeaderId::KeepAlive                        => Self::KeepAlive(v.parse()?),
			HeaderId::LastModified                     => Self::LastModified(parse_date(v)?),
			HeaderId::Location                         => Self::Location(v.to_string()),
			HeaderId::Method                           => Self::Method(v.parse()?),
			HeaderId::Path                             => Self::Path(v.to_string()),
			HeaderId::ProxyAuthenticate                => Self::ProxyAuthenticate(v.to_string()),
			HeaderId::ProxyAuthorization               => Self::ProxyAuthorization(v.to_string()),
			HeaderId::Range                            => Self::Range(v.parse()?),
			HeaderId::Referer                          => Self::Referer(v.to_string()),
			HeaderId::RetryAfter                       => Self::RetryAfter(v.parse()?),
			HeaderId::SaveData                         => Self::SaveData(v.parse().map_err(|_| ())?),
			HeaderId::Scheme                           => Self::Scheme(v.parse()?),
			HeaderId::SecWebSocketProtocol             => Self::SecWebSocketProtocol(parse_list::<String>(v)?),
			HeaderId::SecWebSocketExtensions           => Self::SecWebSocketExtensions(parse_list::<String>(v)?),
			HeaderId::SecWebSocketVersion              => Self::SecWebSocketVersion(parse_list::<usize>(v)?),
			HeaderId::SecWebSocketAccept               => Self::SecWebSocketAccept(v.to_string()),
			HeaderId::SecWebSocketKey                  => Self::SecWebSocketKey(v.to_string()),
			HeaderId::Server                           => Self::Server(v.to_string()),
			HeaderId::SetCookie                        => Self::Custom(id.name_v1().to_string(), v.to_string()),//Self::SetCookie(v.parse().map_err(|_| ())?),
			HeaderId::Status                           => Self::Status(v.parse()?),
			HeaderId::Upgrade                          => Self::Upgrade(v.to_string()),
			HeaderId::UpgradeInsecureRequests          => Self::UpgradeInsecureRequests(v.parse().map_err(|_| ())?),
			HeaderId::UserAgent                        => Self::UserAgent(v.to_string()),
			HeaderId::Vary                             => Self::Vary(parse_list::<HeaderId>(v)?),
			HeaderId::WwwAuthenticate                  => Self::WwwAuthenticate(v.to_string()),
		})
	}

	pub fn id(&self) -> Result<HeaderId, &str> {
		Ok(match self {
			Self::AcceptCharset(_)                    => HeaderId::AcceptCharset,
			Self::AcceptEncoding(_)                   => HeaderId::AcceptEncoding,
			Self::AcceptLanguage(_)                   => HeaderId::AcceptLanguage,
			Self::AcceptRanges(_)                     => HeaderId::AcceptRanges,
			Self::AcceptTypes(_)                      => HeaderId::AcceptTypes,
			Self::AccessControlAllowCredentials(_)    => HeaderId::AccessControlAllowCredentials,
			Self::AccessControlAllowHeaders(_)        => HeaderId::AccessControlAllowHeaders,
			Self::AccessControlAllowMethods(_)        => HeaderId::AccessControlAllowMethods,
			Self::AccessControlAllowOrigin(_)         => HeaderId::AccessControlAllowOrigin,
			Self::AccessControlAllowExposeHeaders(_)  => HeaderId::AccessControlAllowExposeHeaders,
			Self::AccessControlAllowMaxAge(_)         => HeaderId::AccessControlAllowMaxAge,
			Self::AccessControlAllowRequestHeaders(_) => HeaderId::AccessControlAllowRequestHeaders,
			Self::AccessControlAllowRequestMethod(_)  => HeaderId::AccessControlAllowRequestMethod,
			Self::Allow(_)                            => HeaderId::Allow,
			Self::Authority(_)                        => HeaderId::Authority,
			Self::Authorization(_)                    => HeaderId::Authorization,
			Self::CacheControl(_)                     => HeaderId::CacheControl,
			Self::ClearSiteData(_)                    => HeaderId::ClearSiteData,
			Self::Connection(_)                       => HeaderId::Connection,
			Self::ContentDisposition(_)               => HeaderId::ContentDisposition,
			Self::ContentEncoding(_)                  => HeaderId::ContentEncoding,
			Self::ContentLanguage(_)                  => HeaderId::ContentLanguage,
			Self::ContentLength(_)                    => HeaderId::ContentLength,
			Self::ContentLocation(_)                  => HeaderId::ContentLocation,
			Self::ContentRange(_)                     => HeaderId::ContentRange,
			Self::ContentType(_)                      => HeaderId::ContentType,
			Self::Cookie(_)                           => HeaderId::Cookie,
			Self::Dnt(_)                              => HeaderId::Dnt,
			Self::Date(_)                             => HeaderId::Date,
			Self::ETag(_)                             => HeaderId::ETag,
			Self::Expires(_)                          => HeaderId::Expires,
			Self::Host(_)                             => HeaderId::Host,
			Self::IfMatch(_)                          => HeaderId::IfMatch,
			Self::IfModifiedSince(_)                  => HeaderId::IfModifiedSince,
			Self::IfNoneMatch(_)                      => HeaderId::IfNoneMatch,
			Self::IfRange(_)                          => HeaderId::IfRange,
			Self::IfUnmodifiedSince(_)                => HeaderId::IfUnmodifiedSince,
			Self::KeepAlive(_)                        => HeaderId::KeepAlive,
			Self::LastModified(_)                     => HeaderId::LastModified,
			Self::Location(_)                         => HeaderId::Location,
			Self::Method(_)                           => HeaderId::Method,
			Self::Path(_)                             => HeaderId::Path,
			Self::ProxyAuthenticate(_)                => HeaderId::ProxyAuthenticate,
			Self::ProxyAuthorization(_)               => HeaderId::ProxyAuthorization,
			Self::Range(_)                            => HeaderId::Range,
			Self::Referer(_)                          => HeaderId::Referer,
			Self::RetryAfter(_)                       => HeaderId::RetryAfter,
			Self::SaveData(_)                         => HeaderId::SaveData,
			Self::Scheme(_)                           => HeaderId::Scheme,
			Self::SecWebSocketProtocol(_)             => HeaderId::SecWebSocketProtocol,
			Self::SecWebSocketExtensions(_)           => HeaderId::SecWebSocketExtensions,
			Self::SecWebSocketVersion(_)              => HeaderId::SecWebSocketVersion,
			Self::SecWebSocketAccept(_)               => HeaderId::SecWebSocketAccept,
			Self::SecWebSocketKey(_)                  => HeaderId::SecWebSocketKey,
			Self::Server(_)                           => HeaderId::Server,
			Self::SetCookie(_)                        => HeaderId::SetCookie,
			Self::Status(_)                           => HeaderId::Status,
			Self::Upgrade(_)                          => HeaderId::Upgrade,
			Self::UpgradeInsecureRequests(_)          => HeaderId::UpgradeInsecureRequests,
			Self::UserAgent(_)                        => HeaderId::UserAgent,
			Self::Vary(_)                             => HeaderId::Vary,
			Self::WwwAuthenticate(_)                  => HeaderId::WwwAuthenticate,
			Self::Custom(name, _)                     => return Err(name),
		})
	}

	pub fn name_v1(&self) -> &str {
		crate::utils::unstable::_82223_into_ok_or_err(self.id().map(HeaderId::name_v1))
	}

	pub fn name_v2(&self) -> &str {
		crate::utils::unstable::_82223_into_ok_or_err(self.id().map(HeaderId::name_v2))
	}

	pub fn as_accept_charset(&self) -> Option<&Vec<Charset>> {
		match self {
			Self::AcceptCharset(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_accept_charset(&mut self) -> Option<&mut Vec<Charset>> {
		match self {
			Self::AcceptCharset(v) => Some(v),
			_ => None
		}
	}

	pub fn into_accept_charset(self) -> Option<Vec<Charset>> {
		match self {
			Self::AcceptCharset(v) => Some(v),
			_ => None
		}
	}

	pub fn as_accept_encoding(&self) -> Option<&Vec<Encoding>> {
		match self {
			Self::AcceptEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_accept_encoding(&mut self) -> Option<&mut Vec<Encoding>> {
		match self {
			Self::AcceptEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn into_accept_encoding(self) -> Option<Vec<Encoding>> {
		match self {
			Self::AcceptEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn as_accept_language(&self) -> Option<&Vec<Language>> {
		match self {
			Self::AcceptLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_accept_language(&mut self) -> Option<&mut Vec<Language>> {
		match self {
			Self::AcceptLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn into_accept_language(self) -> Option<Vec<Language>> {
		match self {
			Self::AcceptLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn as_accept_ranges(&self) -> Option<&AcceptRanges> {
		match self {
			Self::AcceptRanges(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_accept_ranges(&mut self) -> Option<&mut AcceptRanges> {
		match self {
			Self::AcceptRanges(v) => Some(v),
			_ => None
		}
	}

	pub fn into_accept_ranges(self) -> Option<AcceptRanges> {
		match self {
			Self::AcceptRanges(v) => Some(v),
			_ => None
		}
	}

	pub fn as_accept_types(&self) -> Option<&Vec<MediaType>> {
		match self {
			Self::AcceptTypes(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_accept_types(&mut self) -> Option<&mut Vec<MediaType>> {
		match self {
			Self::AcceptTypes(v) => Some(v),
			_ => None
		}
	}

	pub fn into_accept_types(self) -> Option<Vec<MediaType>> {
		match self {
			Self::AcceptTypes(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_credentials(&self) -> Option<&bool> {
		match self {
			Self::AccessControlAllowCredentials(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_credentials(&mut self) -> Option<&mut bool> {
		match self {
			Self::AccessControlAllowCredentials(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_credentials(self) -> Option<bool> {
		match self {
			Self::AccessControlAllowCredentials(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_headers(&self) -> Option<&Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_headers(&mut self) -> Option<&mut Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_headers(self) -> Option<Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_methods(&self) -> Option<&Vec<Method>> {
		match self {
			Self::AccessControlAllowMethods(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_methods(&mut self) -> Option<&mut Vec<Method>> {
		match self {
			Self::AccessControlAllowMethods(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_methods(self) -> Option<Vec<Method>> {
		match self {
			Self::AccessControlAllowMethods(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_origin(&self) -> Option<&String> {
		match self {
			Self::AccessControlAllowOrigin(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_origin(&mut self) -> Option<&mut String> {
		match self {
			Self::AccessControlAllowOrigin(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_origin(self) -> Option<String> {
		match self {
			Self::AccessControlAllowOrigin(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_expose_headers(&self) -> Option<&Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowExposeHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_expose_headers(&mut self) -> Option<&mut Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowExposeHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_expose_headers(self) -> Option<Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowExposeHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_max_age(&self) -> Option<&usize> {
		match self {
			Self::AccessControlAllowMaxAge(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_max_age(&mut self) -> Option<&mut usize> {
		match self {
			Self::AccessControlAllowMaxAge(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_max_age(self) -> Option<usize> {
		match self {
			Self::AccessControlAllowMaxAge(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_request_headers(&self) -> Option<&Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowRequestHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_request_headers(&mut self) -> Option<&mut Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowRequestHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_request_headers(self) -> Option<Vec<HeaderId>> {
		match self {
			Self::AccessControlAllowRequestHeaders(v) => Some(v),
			_ => None
		}
	}

	pub fn as_access_control_allow_request_method(&self) -> Option<&Method> {
		match self {
			Self::AccessControlAllowRequestMethod(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_access_control_allow_request_method(&mut self) -> Option<&mut Method> {
		match self {
			Self::AccessControlAllowRequestMethod(v) => Some(v),
			_ => None
		}
	}

	pub fn into_access_control_allow_request_method(self) -> Option<Method> {
		match self {
			Self::AccessControlAllowRequestMethod(v) => Some(v),
			_ => None
		}
	}

	pub fn as_allow(&self) -> Option<&Vec<Method>> {
		match self {
			Self::Allow(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_allow(&mut self) -> Option<&mut Vec<Method>> {
		match self {
			Self::Allow(v) => Some(v),
			_ => None
		}
	}

	pub fn into_allow(self) -> Option<Vec<Method>> {
		match self {
			Self::Allow(v) => Some(v),
			_ => None
		}
	}

	pub fn as_authority(&self) -> Option<&String> {
		match self {
			Self::Authority(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_authority(&mut self) -> Option<&mut String> {
		match self {
			Self::Authority(v) => Some(v),
			_ => None
		}
	}

	pub fn into_authority(self) -> Option<String> {
		match self {
			Self::Authority(v) => Some(v),
			_ => None
		}
	}

	pub fn as_authorization(&self) -> Option<&Authorization> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_authorization(&mut self) -> Option<&mut Authorization> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}

	pub fn into_authorization(self) -> Option<Authorization> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}

	pub fn as_cache_control(&self) -> Option<&CacheControl> {
		match self {
			Self::CacheControl(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_cache_control(&mut self) -> Option<&mut CacheControl> {
		match self {
			Self::CacheControl(v) => Some(v),
			_ => None
		}
	}

	pub fn into_cache_control(self) -> Option<CacheControl> {
		match self {
			Self::CacheControl(v) => Some(v),
			_ => None
		}
	}

	pub fn as_clear_site_data(&self) -> Option<&ClearSiteDataDirective> {
		match self {
			Self::ClearSiteData(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_clear_site_data(&mut self) -> Option<&mut ClearSiteDataDirective> {
		match self {
			Self::ClearSiteData(v) => Some(v),
			_ => None
		}
	}

	pub fn into_clear_site_data(self) -> Option<ClearSiteDataDirective> {
		match self {
			Self::ClearSiteData(v) => Some(v),
			_ => None
		}
	}

	pub fn as_connection(&self) -> Option<&Connection> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_connection(&mut self) -> Option<&mut Connection> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}

	pub fn into_connection(self) -> Option<Connection> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_disposition(&self) -> Option<&String> {
		match self {
			Self::ContentDisposition(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_disposition(&mut self) -> Option<&mut String> {
		match self {
			Self::ContentDisposition(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_disposition(self) -> Option<String> {
		match self {
			Self::ContentDisposition(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_encoding(&self) -> Option<&Encoding> {
		match self {
			Self::ContentEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_encoding(&mut self) -> Option<&mut Encoding> {
		match self {
			Self::ContentEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_encoding(self) -> Option<Encoding> {
		match self {
			Self::ContentEncoding(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_language(&self) -> Option<&Language> {
		match self {
			Self::ContentLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_language(&mut self) -> Option<&mut Language> {
		match self {
			Self::ContentLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_language(self) -> Option<Language> {
		match self {
			Self::ContentLanguage(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_length(&self) -> Option<&usize> {
		match self {
			Self::ContentLength(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_length(&mut self) -> Option<&mut usize> {
		match self {
			Self::ContentLength(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_length(self) -> Option<usize> {
		match self {
			Self::ContentLength(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_location(&self) -> Option<&String> {
		match self {
			Self::ContentLocation(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_location(&mut self) -> Option<&mut String> {
		match self {
			Self::ContentLocation(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_location(self) -> Option<String> {
		match self {
			Self::ContentLocation(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_range(&self) -> Option<&ContentRange> {
		match self {
			Self::ContentRange(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_range(&mut self) -> Option<&mut ContentRange> {
		match self {
			Self::ContentRange(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_range(self) -> Option<ContentRange> {
		match self {
			Self::ContentRange(v) => Some(v),
			_ => None
		}
	}

	pub fn as_content_type(&self) -> Option<&MediaType> {
		match self {
			Self::ContentType(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_content_type(&mut self) -> Option<&mut MediaType> {
		match self {
			Self::ContentType(v) => Some(v),
			_ => None
		}
	}

	pub fn into_content_type(self) -> Option<Box<MediaType>> {
		match self {
			Self::ContentType(v) => Some(v),
			_ => None
		}
	}

	pub fn as_cookie(&self) -> Option<&Vec<(String, String)>> {
		match self {
			Self::Cookie(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_cookie(&mut self) -> Option<&mut Vec<(String, String)>> {
		match self {
			Self::Cookie(v) => Some(v),
			_ => None
		}
	}

	pub fn into_cookie(self) -> Option<Vec<(String, String)>> {
		match self {
			Self::Cookie(v) => Some(v),
			_ => None
		}
	}

	pub fn as_dnt(&self) -> Option<&bool> {
		match self {
			Self::Dnt(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_dnt(&mut self) -> Option<&mut bool> {
		match self {
			Self::Dnt(v) => Some(v),
			_ => None
		}
	}

	pub fn into_dnt(self) -> Option<bool> {
		match self {
			Self::Dnt(v) => Some(v),
			_ => None
		}
	}

	pub fn as_date(&self) -> Option<&DateTime<Utc>> {
		match self {
			Self::Date(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_date(&mut self) -> Option<&mut DateTime<Utc>> {
		match self {
			Self::Date(v) => Some(v),
			_ => None
		}
	}

	pub fn into_date(self) -> Option<DateTime<Utc>> {
		match self {
			Self::Date(v) => Some(v),
			_ => None
		}
	}

	pub fn as_e_tag(&self) -> Option<&ETag> {
		match self {
			Self::ETag(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_e_tag(&mut self) -> Option<&mut ETag> {
		match self {
			Self::ETag(v) => Some(v),
			_ => None
		}
	}

	pub fn into_e_tag(self) -> Option<ETag> {
		match self {
			Self::ETag(v) => Some(v),
			_ => None
		}
	}

	pub fn as_expires(&self) -> Option<&DateTime<Utc>> {
		match self {
			Self::Expires(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_expires(&mut self) -> Option<&mut DateTime<Utc>> {
		match self {
			Self::Expires(v) => Some(v),
			_ => None
		}
	}

	pub fn into_expires(self) -> Option<DateTime<Utc>> {
		match self {
			Self::Expires(v) => Some(v),
			_ => None
		}
	}

	pub fn as_host(&self) -> Option<&String> {
		match self {
			Self::Host(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_host(&mut self) -> Option<&mut String> {
		match self {
			Self::Host(v) => Some(v),
			_ => None
		}
	}

	pub fn into_host(self) -> Option<String> {
		match self {
			Self::Host(v) => Some(v),
			_ => None
		}
	}

	pub fn as_if_match(&self) -> Option<&Vec<String>> {
		match self {
			Self::IfMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_if_match(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::IfMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn into_if_match(self) -> Option<Vec<String>> {
		match self {
			Self::IfMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn as_if_modified_since(&self) -> Option<&DateTime<Utc>> {
		match self {
			Self::IfModifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_if_modified_since(&mut self) -> Option<&mut DateTime<Utc>> {
		match self {
			Self::IfModifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn into_if_modified_since(self) -> Option<DateTime<Utc>> {
		match self {
			Self::IfModifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn as_if_none_match(&self) -> Option<&Vec<String>> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_if_none_match(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn into_if_none_match(self) -> Option<Vec<String>> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
			_ => None
		}
	}

	pub fn as_if_range(&self) -> Option<&()> {
		match self {
			Self::IfRange(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_if_range(&mut self) -> Option<&mut ()> {
		match self {
			Self::IfRange(v) => Some(v),
			_ => None
		}
	}

	pub fn into_if_range(self) -> Option<()> {
		match self {
			Self::IfRange(v) => Some(v),
			_ => None
		}
	}

	pub fn as_if_unmodified_since(&self) -> Option<&DateTime<Utc>> {
		match self {
			Self::IfUnmodifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_if_unmodified_since(&mut self) -> Option<&mut DateTime<Utc>> {
		match self {
			Self::IfUnmodifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn into_if_unmodified_since(self) -> Option<DateTime<Utc>> {
		match self {
			Self::IfUnmodifiedSince(v) => Some(v),
			_ => None
		}
	}

	pub fn as_keep_alive(&self) -> Option<&KeepAlive> {
		match self {
			Self::KeepAlive(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_keep_alive(&mut self) -> Option<&mut KeepAlive> {
		match self {
			Self::KeepAlive(v) => Some(v),
			_ => None
		}
	}

	pub fn into_keep_alive(self) -> Option<KeepAlive> {
		match self {
			Self::KeepAlive(v) => Some(v),
			_ => None
		}
	}

	pub fn as_last_modified(&self) -> Option<&DateTime<Utc>> {
		match self {
			Self::LastModified(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_last_modified(&mut self) -> Option<&mut DateTime<Utc>> {
		match self {
			Self::LastModified(v) => Some(v),
			_ => None
		}
	}

	pub fn into_last_modified(self) -> Option<DateTime<Utc>> {
		match self {
			Self::LastModified(v) => Some(v),
			_ => None
		}
	}

	pub fn as_location(&self) -> Option<&String> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_location(&mut self) -> Option<&mut String> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}

	pub fn into_location(self) -> Option<String> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}

	pub fn as_method(&self) -> Option<&Method> {
		match self {
			Self::Method(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_method(&mut self) -> Option<&mut Method> {
		match self {
			Self::Method(v) => Some(v),
			_ => None
		}
	}

	pub fn into_method(self) -> Option<Method> {
		match self {
			Self::Method(v) => Some(v),
			_ => None
		}
	}

	pub fn as_path(&self) -> Option<&String> {
		match self {
			Self::Path(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_path(&mut self) -> Option<&mut String> {
		match self {
			Self::Path(v) => Some(v),
			_ => None
		}
	}

	pub fn into_path(self) -> Option<String> {
		match self {
			Self::Path(v) => Some(v),
			_ => None
		}
	}

	pub fn as_range(&self) -> Option<&Ranges> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_range(&mut self) -> Option<&mut Ranges> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}

	pub fn into_range(self) -> Option<Ranges> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}

	pub fn as_referer(&self) -> Option<&String> {
		match self {
			Self::Referer(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_referer(&mut self) -> Option<&mut String> {
		match self {
			Self::Referer(v) => Some(v),
			_ => None
		}
	}

	pub fn into_referer(self) -> Option<String> {
		match self {
			Self::Referer(v) => Some(v),
			_ => None
		}
	}

	pub fn as_retry_after(&self) -> Option<&RetryAfter> {
		match self {
			Self::RetryAfter(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_retry_after(&mut self) -> Option<&mut RetryAfter> {
		match self {
			Self::RetryAfter(v) => Some(v),
			_ => None
		}
	}

	pub fn into_retry_after(self) -> Option<RetryAfter> {
		match self {
			Self::RetryAfter(v) => Some(v),
			_ => None
		}
	}

	pub fn as_save_data(&self) -> Option<&bool> {
		match self {
			Self::SaveData(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_save_data(&mut self) -> Option<&mut bool> {
		match self {
			Self::SaveData(v) => Some(v),
			_ => None
		}
	}

	pub fn into_save_data(self) -> Option<bool> {
		match self {
			Self::SaveData(v) => Some(v),
			_ => None
		}
	}

	pub fn as_scheme(&self) -> Option<&Scheme> {
		match self {
			Self::Scheme(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_scheme(&mut self) -> Option<&mut Scheme> {
		match self {
			Self::Scheme(v) => Some(v),
			_ => None
		}
	}

	pub fn into_scheme(self) -> Option<Scheme> {
		match self {
			Self::Scheme(v) => Some(v),
			_ => None
		}
	}

	pub fn as_server(&self) -> Option<&String> {
		match self {
			Self::Server(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_server(&mut self) -> Option<&mut String> {
		match self {
			Self::Server(v) => Some(v),
			_ => None
		}
	}

	pub fn into_server(self) -> Option<String> {
		match self {
			Self::Server(v) => Some(v),
			_ => None
		}
	}

	pub fn as_set_cookie(&self) -> Option<&SetCookie> {
		match self {
			Self::SetCookie(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_set_cookie(&mut self) -> Option<&mut SetCookie> {
		match self {
			Self::SetCookie(v) => Some(v),
			_ => None
		}
	}

	pub fn into_set_cookie(self) -> Option<SetCookie> {
		match self {
			Self::SetCookie(v) => Some(v),
			_ => None
		}
	}

	pub fn as_status(&self) -> Option<&Status> {
		match self {
			Self::Status(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_status(&mut self) -> Option<&mut Status> {
		match self {
			Self::Status(v) => Some(v),
			_ => None
		}
	}

	pub fn into_status(self) -> Option<Status> {
		match self {
			Self::Status(v) => Some(v),
			_ => None
		}
	}

	pub fn as_upgrade(&self) -> Option<&String> {
		match self {
			Self::Upgrade(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_upgrade(&mut self) -> Option<&mut String> {
		match self {
			Self::Upgrade(v) => Some(v),
			_ => None
		}
	}

	pub fn into_upgrade(self) -> Option<String> {
		match self {
			Self::Upgrade(v) => Some(v),
			_ => None
		}
	}

	pub fn as_upgrade_insecure_requests(&self) -> Option<&bool> {
		match self {
			Self::UpgradeInsecureRequests(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_upgrade_insecure_requests(&mut self) -> Option<&mut bool> {
		match self {
			Self::UpgradeInsecureRequests(v) => Some(v),
			_ => None
		}
	}

	pub fn into_upgrade_insecure_requests(self) -> Option<bool> {
		match self {
			Self::UpgradeInsecureRequests(v) => Some(v),
			_ => None
		}
	}

	pub fn as_user_agent(&self) -> Option<&String> {
		match self {
			Self::UserAgent(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_user_agent(&mut self) -> Option<&mut String> {
		match self {
			Self::UserAgent(v) => Some(v),
			_ => None
		}
	}

	pub fn into_user_agent(self) -> Option<String> {
		match self {
			Self::UserAgent(v) => Some(v),
			_ => None
		}
	}

	pub fn as_vary(&self) -> Option<&Vec<HeaderId>> {
		match self {
			Self::Vary(v) => Some(v),
			_ => None
		}
	}

	pub fn as_mut_vary(&mut self) -> Option<&mut Vec<HeaderId>> {
		match self {
			Self::Vary(v) => Some(v),
			_ => None
		}
	}

	pub fn into_vary(self) -> Option<Vec<HeaderId>> {
		match self {
			Self::Vary(v) => Some(v),
			_ => None
		}
	}

	pub fn as_custom(&self) -> Option<(&String, &String)> {
		match self {
			Self::Custom(v0, v1) => Some((v0, v1)),
			_ => None
		}
	}

	pub fn as_mut_custom(&mut self) -> Option<(&mut String, &mut String)> {
		match self {
			Self::Custom(v0, v1) => Some((v0, v1)),
			_ => None
		}
	}

	pub fn into_custom(self) -> Option<(String, String)> {
		match self {
			Self::Custom(v0, v1) => Some((v0, v1)),
			_ => None
		}
	}
}

impl Display for Header {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::AcceptCharset(v)                    => fmt_list(f, v),
			Self::AcceptEncoding(v)                   => fmt_list(f, v),
			Self::AcceptLanguage(v)                   => fmt_list(f, v),
			Self::AcceptRanges(v)                     => Display::fmt(v, f),
			Self::AcceptTypes(v)                      => fmt_list(f, v),
			Self::AccessControlAllowCredentials(v)    => Display::fmt(v, f),
			Self::AccessControlAllowHeaders(v)        => fmt_list(f, v.iter().map(|v| v.name_v1())),
			Self::AccessControlAllowMethods(v)        => fmt_list(f, v),
			Self::AccessControlAllowOrigin(v)         => Display::fmt(v, f),
			Self::AccessControlAllowExposeHeaders(v)  => fmt_list(f, v.iter().map(|v| v.name_v1())),
			Self::AccessControlAllowMaxAge(v)         => Display::fmt(v, f),
			Self::AccessControlAllowRequestHeaders(v) => fmt_list(f, v.iter().map(|v| v.name_v1())),
			Self::AccessControlAllowRequestMethod(v)  => Display::fmt(v, f),
			Self::Allow(v)                            => fmt_list(f, v),
			Self::Authority(v)                        => Display::fmt(v, f),
			Self::Authorization(v)                    => Display::fmt(v, f),
			Self::CacheControl(v)                     => Display::fmt(v, f),
			Self::ClearSiteData(v)                    => Display::fmt(v, f),
			Self::Connection(v)                       => Display::fmt(v, f),
			Self::ContentDisposition(v)               => Display::fmt(v, f),
			Self::ContentEncoding(v)                  => Display::fmt(v, f),
			Self::ContentLanguage(v)                  => Display::fmt(v, f),
			Self::ContentLength(v)                    => Display::fmt(v, f),
			Self::ContentLocation(v)                  => Display::fmt(v, f),
			Self::ContentRange(v)                     => Display::fmt(v, f),
			Self::ContentType(v)                      => Display::fmt(v, f),
			Self::Cookie(v)                           => fmt_map(f, v.iter().map(|(k, v)| (k, v))),
			Self::Date(v)                             => write!(f, "{}", v.format(DATE_FORMAT)),
			Self::Dnt(v)                              => Display::fmt(v, f),
			Self::ETag(v)                             => Display::fmt(v, f),
			Self::Expires(v)                          => write!(f, "{}", v.format(DATE_FORMAT)),
			Self::Host(v)                             => Display::fmt(v, f),
			Self::IfMatch(v)                          => fmt_list(f, v),
			Self::IfModifiedSince(v)                  => write!(f, "{}", v.format(DATE_FORMAT)),
			Self::IfNoneMatch(v)                      => fmt_list(f, v),
			Self::IfRange(())                         => write!(f, ""),
			Self::IfUnmodifiedSince(v)                => write!(f, "{}", v.format(DATE_FORMAT)),
			Self::KeepAlive(v)                        => write!(f, "{}", v),
			Self::LastModified(v)                     => write!(f, "{}", v.format(DATE_FORMAT)),
			Self::Location(v)                         => Display::fmt(v, f),
			Self::Method(v)                           => Display::fmt(v, f),
			Self::Path(v)                             => Display::fmt(v, f),
			Self::ProxyAuthenticate(v)                => Display::fmt(v, f),
			Self::ProxyAuthorization(v)               => Display::fmt(v, f),
			Self::Range(v)                            => Display::fmt(v, f),
			Self::Referer(v)                          => Display::fmt(v, f),
			Self::RetryAfter(v)                       => Display::fmt(v, f),
			Self::SaveData(v)                         => Display::fmt(v, f),
			Self::Scheme(v)                           => Display::fmt(v, f),
			Self::SecWebSocketProtocol(v)             => fmt_list(f, v),
			Self::SecWebSocketExtensions(v)           => fmt_list(f, v),
			Self::SecWebSocketVersion(v)              => fmt_list(f, v),
			Self::SecWebSocketAccept(v)               => Display::fmt(v, f),
			Self::SecWebSocketKey(v)                  => Display::fmt(v, f),
			Self::Server(v)                           => Display::fmt(v, f),
			Self::SetCookie(v)                        => Display::fmt(v, f),
			Self::Status(v)                           => write!(f, "{} {}", *v as usize, v),
			Self::Upgrade(v)                          => Display::fmt(v, f),
			Self::UpgradeInsecureRequests(v)          => Display::fmt(v, f),
			Self::UserAgent(v)                        => Display::fmt(v, f),
			Self::Vary(v)                             => fmt_list(f, v.iter().map(|v| v.name_v1())),
			Self::WwwAuthenticate(v)                  => Display::fmt(v, f),
			Self::Custom(_, v)                        => Display::fmt(v, f),

		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Unit {
	Bytes,
	Other(Box<str>)
}

impl FromStr for Unit {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"bytes" => Self::Bytes,
			s       => Self::Other(s.to_string().into_boxed_str())
		})
	}
}

impl Display for Unit {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Bytes    => "bytes",
			Self::Other(s) => s
		})
	}
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Method {
	Options,
	Get,
	Head,
	Post,
	Put,
	Delete,
	Trace,
	Connect,
	Patch,
	Other(Box<str>)
}

impl FromStr for Method {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"OPTIONS" => Self::Options,
			"GET"     => Self::Get,
			"HEAD"    => Self::Head,
			"POST"    => Self::Post,
			"PUT"     => Self::Put,
			"DELETE"  => Self::Delete,
			"TRACE"   => Self::Trace,
			"CONNECT" => Self::Connect,
			"PATCH"   => Self::Patch,
			v         => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for Method {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Options  => "OPTIONS",
			Self::Get      => "GET",
			Self::Head     => "HEAD",
			Self::Post     => "POST",
			Self::Put      => "PUT",
			Self::Delete   => "DELETE",
			Self::Trace    => "TRACE",
			Self::Connect  => "CONNECT",
			Self::Patch    => "PATCH",
			Self::Other(v) => v
		})
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Status {
	Continue                      = 100,
	SwitchingProtocols            = 101,
	Processing                    = 102,
	EarlyHints                    = 103,
	Ok                            = 200,
	Created                       = 201,
	Accepted                      = 202,
	NonAuthoritativeInformation   = 203,
	NoContent                     = 204,
	ResetContent                  = 205,
	PartialContent                = 206,
	MultiStatus                   = 207,
	AlreadyReported               = 208,
	ImUsed                        = 226,
	MultipleChoices               = 300,
	MovedPermanently              = 301,
	Found                         = 302,
	SeeOther                      = 303,
	NotModified                   = 304,
	UseProxy                      = 305,
	TemporaryRedirect             = 307,
	PermanentRedirect             = 308,
	BadRequest                    = 400,
	Unauthorized                  = 401,
	PaymentRequired               = 402,
	Forbidden                     = 403,
	NotFound                      = 404,
	MethodNotAllowed              = 405,
	NotAcceptable                 = 406,
	ProxyAuthenticationRequired   = 407,
	RequestTimeout                = 408,
	Conflict                      = 409,
	Gone                          = 410,
	LengthRequired                = 411,
	PreconditionFailed            = 412,
	PayloadTooLarge               = 413,
	UriTooLong                    = 414,
	UnsupportedMediaType          = 415,
	RangeNotSatisfiable           = 416,
	ExpectationFailed             = 417,
	MisdirectedRequest            = 421,
	UnprocessableEntity           = 422,
	Locked                        = 423,
	FailedDependency              = 424,
	UpgradeRequired               = 426,
	PreconditionRequired          = 428,
	TooManyRequests               = 429,
	RequestHeaderFilesTooLarge    = 431,
	UnavailableForLegalReasons    = 451,
	InternalServerError           = 500,
	NotImplemented                = 501,
	BadGateway                    = 502,
	ServiceUnavailable            = 503,
	GatewayTimeout                = 504,
	HttpVersionNotSupported       = 505,
	VariantAlsoNegotiates         = 506,
	InsufficientStorage           = 507,
	LoopDetected                  = 508,
	NotExtended                   = 510,
	NetworkAuthenticationRequired = 511
}

impl std::error::Error for Status {}

impl TryFrom<usize> for Status {
	type Error = ();

	fn try_from(value: usize) -> Result<Self, Self::Error> {
		Ok(match value {
			100 => Self::Continue,
			101 => Self::SwitchingProtocols,
			102 => Self::Processing,
			103 => Self::EarlyHints,
			200 => Self::Ok,
			201 => Self::Created,
			202 => Self::Accepted,
			203 => Self::NonAuthoritativeInformation,
			204 => Self::NoContent,
			205 => Self::ResetContent,
			206 => Self::PartialContent,
			207 => Self::MultiStatus,
			208 => Self::AlreadyReported,
			226 => Self::ImUsed,
			300 => Self::MultipleChoices,
			301 => Self::MovedPermanently,
			302 => Self::Found,
			303 => Self::SeeOther,
			304 => Self::NotModified,
			305 => Self::UseProxy,
			307 => Self::TemporaryRedirect,
			308 => Self::PermanentRedirect,
			400 => Self::BadRequest,
			401 => Self::Unauthorized,
			402 => Self::PaymentRequired,
			403 => Self::Forbidden,
			404 => Self::NotFound,
			405 => Self::MethodNotAllowed,
			406 => Self::NotAcceptable,
			407 => Self::ProxyAuthenticationRequired,
			408 => Self::RequestTimeout,
			409 => Self::Conflict,
			410 => Self::Gone,
			411 => Self::LengthRequired,
			412 => Self::PreconditionFailed,
			413 => Self::PayloadTooLarge,
			414 => Self::UriTooLong,
			415 => Self::UnsupportedMediaType,
			416 => Self::RangeNotSatisfiable,
			417 => Self::ExpectationFailed,
			421 => Self::MisdirectedRequest,
			422 => Self::UnprocessableEntity,
			423 => Self::Locked,
			424 => Self::FailedDependency,
			426 => Self::UpgradeRequired,
			428 => Self::PreconditionRequired,
			429 => Self::TooManyRequests,
			431 => Self::RequestHeaderFilesTooLarge,
			451 => Self::UnavailableForLegalReasons,
			500 => Self::InternalServerError,
			501 => Self::NotImplemented,
			502 => Self::BadGateway,
			503 => Self::ServiceUnavailable,
			504 => Self::GatewayTimeout,
			505 => Self::HttpVersionNotSupported,
			506 => Self::VariantAlsoNegotiates,
			507 => Self::InsufficientStorage,
			508 => Self::LoopDetected,
			510 => Self::NotExtended,
			511 => Self::NetworkAuthenticationRequired,
			_   => return Err(())
		})
	}
}

impl FromStr for Status {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::try_from(s.parse::<usize>().map_err(|_| ())?)
	}
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Continue                       => "CONTINUE",
			Self::SwitchingProtocols             => "SWITCHING PROTOCOLS",
			Self::Processing                     => "PROCESSING",
			Self::EarlyHints                     => "EARLY HINTS",
			Self::Ok                             => "OK",
			Self::Created                        => "CREATED",
			Self::Accepted                       => "ACCEPTED",
			Self::NonAuthoritativeInformation    => "NON AUTHORITATIVE INFORMATION",
			Self::NoContent                      => "NO CONTENT",
			Self::ResetContent                   => "RESET CONTENT",
			Self::PartialContent                 => "PARTIAL CONTENT",
			Self::MultiStatus                    => "MULTI STATUS",
			Self::AlreadyReported                => "ALREADY REPORTED",
			Self::ImUsed                         => "IM USED",
			Self::MultipleChoices                => "MULTIPLE CHOICES",
			Self::MovedPermanently               => "MOVED PERMANENTLY",
			Self::Found                          => "FOUND",
			Self::SeeOther                       => "SEE OTHER",
			Self::NotModified                    => "NOT MODIFIED",
			Self::UseProxy                       => "USE PROXY",
			Self::TemporaryRedirect              => "TEMPORARY REDIRECT",
			Self::PermanentRedirect              => "PERMANENT REDIRECT",
			Self::BadRequest                     => "BAD REQUEST",
			Self::Unauthorized                   => "UNAUTHORIZED",
			Self::PaymentRequired                => "PAYMENT REQUIRED",
			Self::Forbidden                      => "FORBIDDEN",
			Self::NotFound                       => "NOT FOUND",
			Self::MethodNotAllowed               => "METHOD NOT ALLOWED",
			Self::NotAcceptable                  => "NOT ACCEPTABLE",
			Self::ProxyAuthenticationRequired    => "PROXY AUTHENTICATION REQUIRED",
			Self::RequestTimeout                 => "REQUEST TIMEOUT",
			Self::Conflict                       => "CONFLICT",
			Self::Gone                           => "GONE",
			Self::LengthRequired                 => "LENGTH REQUIRED",
			Self::PreconditionFailed             => "PRECONDITION FAILED",
			Self::PayloadTooLarge                => "PAYLOAD TOO LARGE",
			Self::UriTooLong                     => "URI TOO LONG",
			Self::UnsupportedMediaType           => "UNSUPPORTED MEDIA TYPE",
			Self::RangeNotSatisfiable            => "RANGE NOT SATISFIABLE",
			Self::ExpectationFailed              => "EXPECTATION FAILED",
			Self::MisdirectedRequest             => "MISDIRECTED REQUEST",
			Self::UnprocessableEntity            => "UNPROCESSABLE ENTITY",
			Self::Locked                         => "LOCKED",
			Self::FailedDependency               => "FAILED DEPENDENCY",
			Self::UpgradeRequired                => "UPGRADE REQUIRED",
			Self::PreconditionRequired           => "PRECONDITION REQUIRED",
			Self::TooManyRequests                => "TOO MANY REQUESTS",
			Self::RequestHeaderFilesTooLarge     => "REQUEST HEADER FILES TOO LARGE",
			Self::UnavailableForLegalReasons     => "UNAVAILABLE FOR LEGAL REASONS",
			Self::InternalServerError            => "INTERNAL SERVER ERROR",
			Self::NotImplemented                 => "NOT IMPLEMENTED",
			Self::BadGateway                     => "BAD GATEWAY",
			Self::ServiceUnavailable             => "SERVICE UNAVAILABLE",
			Self::GatewayTimeout                 => "GATEWAY TIMEOUT",
			Self::HttpVersionNotSupported        => "HTTP VERSION NOT SUPPORTED",
			Self::VariantAlsoNegotiates          => "VARIANT ALSO NEGOTIATES",
			Self::InsufficientStorage            => "INSUFFICIENT STORAGE",
			Self::LoopDetected                   => "LOOP DETECTED",
			Self::NotExtended                    => "NOT EXTENDED",
			Self::NetworkAuthenticationRequired  => "NETWORK AUTHENTICATION REQUIRED",
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Scheme {
	Http,
	Https,
	Ssh,
	Other(Box<str>)
}

impl FromStr for Scheme {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"http"  => Self::Http,
			"https" => Self::Https,
			"ssh"   => Self::Ssh,
			v       => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for Scheme {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Http     => "http",
			Self::Https    => "https",
			Self::Ssh      => "ssh",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Connection {
	KeepAlive,
	Close,
	Upgrade,
	Other(Box<str>)
}

impl FromStr for Connection {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"keep-alive" => Self::KeepAlive,
			"close"      => Self::Close,
			"Upgrade"    => Self::Upgrade,
			v            => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for Connection {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::KeepAlive => "keep-alive",
			Self::Close     => "close",
			Self::Upgrade   => "Upgrade",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Charset {
	UsAscii,
	Utf8,
	Utf16,
	Utf16LE,
	Utf16BE,
	Utf32,
	Utf32LE,
	Utf32BE,
	Iso8859_1,
	Iso8859_2,
	Iso8859_3,
	Iso8859_4,
	Iso8859_5,
	Iso8859_6,
	Iso8859_7,
	Iso8859_8,
	Iso8859_9,
	Iso8859_10,
	Iso8859_11,
	Iso8859_13,
	Iso8859_14,
	Iso8859_15,
	Iso8859_16,
	Windows1251,
	Windows1252,
	Windows1253,
	Windows1254,
	Windows1255,
	Windows1256,
	Windows1257,
	Windows1258,
	Other(Box<str>)
}

impl FromStr for Charset {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"us-ascii" | "US-ASCII" => Self::UsAscii,
			"utf-8"    | "UTF-8"    => Self::Utf8,
			"utf-16"   | "UTF-16"   => Self::Utf16,
			"utf-16le" | "UTF-16LE" => Self::Utf16LE,
			"utf-16be" | "UTF-16BE" => Self::Utf16BE,
			"utf-32"   | "UTF-32"   => Self::Utf32,
			"utf-32le" | "UTF-32LE" => Self::Utf32LE,
			"utf-32be" | "UTF-32BE" => Self::Utf32BE,
			"ISO-8859-1"            => Self::Iso8859_1,
			"ISO-8859-2"            => Self::Iso8859_2,
			"ISO-8859-3"            => Self::Iso8859_3,
			"ISO-8859-4"            => Self::Iso8859_4,
			"ISO-8859-5"            => Self::Iso8859_5,
			"ISO-8859-6"            => Self::Iso8859_6,
			"ISO-8859-7"            => Self::Iso8859_7,
			"ISO-8859-8"            => Self::Iso8859_8,
			"ISO-8859-9"            => Self::Iso8859_9,
			"ISO-8859-10"           => Self::Iso8859_10,
			"ISO-8859-11"           => Self::Iso8859_11,
			"ISO-8859-13"           => Self::Iso8859_13,
			"ISO-8859-14"           => Self::Iso8859_14,
			"ISO-8859-15"           => Self::Iso8859_15,
			"ISO-8859-16"           => Self::Iso8859_16,
			"windows-1251"          => Self::Windows1251,
			"windows-1252"          => Self::Windows1252,
			"windows-1253"          => Self::Windows1253,
			"windows-1254"          => Self::Windows1254,
			"windows-1255"          => Self::Windows1255,
			"windows-1256"          => Self::Windows1256,
			"windows-1257"          => Self::Windows1257,
			"windows-1258"          => Self::Windows1258,
			v                       => Self::Other(v.to_lowercase().into_boxed_str())
		})
	}
}

impl Display for Charset {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::UsAscii     => "US-ASCII",
			Self::Utf8        => "UTF-8",
			Self::Utf16       => "UTF-16",
			Self::Utf16LE     => "UTF-16LE",
			Self::Utf16BE     => "UTF-16BE",
			Self::Utf32       => "UTF-32",
			Self::Utf32LE     => "UTF-32LE",
			Self::Utf32BE     => "UTF-32BE",
			Self::Iso8859_1   => "ISO-8859-1",
			Self::Iso8859_2   => "ISO-8859-2",
			Self::Iso8859_3   => "ISO-8859-3",
			Self::Iso8859_4   => "ISO-8859-4",
			Self::Iso8859_5   => "ISO-8859-5",
			Self::Iso8859_6   => "ISO-8859-6",
			Self::Iso8859_7   => "ISO-8859-7",
			Self::Iso8859_8   => "ISO-8859-8",
			Self::Iso8859_9   => "ISO-8859-9",
			Self::Iso8859_10  => "ISO-8859-10",
			Self::Iso8859_11  => "ISO-8859-11",
			Self::Iso8859_13  => "ISO-8859-13",
			Self::Iso8859_14  => "ISO-8859-14",
			Self::Iso8859_15  => "ISO-8859-15",
			Self::Iso8859_16  => "ISO-8859-16",
			Self::Windows1251 => "windows-1251",
			Self::Windows1252 => "windows-1252",
			Self::Windows1253 => "windows-1253",
			Self::Windows1254 => "windows-1254",
			Self::Windows1255 => "windows-1255",
			Self::Windows1256 => "windows-1256",
			Self::Windows1257 => "windows-1257",
			Self::Windows1258 => "windows-1258",
			Self::Other(v)    => v,
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum AcceptRanges {
	None,
	Bytes,
	Other(Box<str>)
}

impl FromStr for AcceptRanges {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"none"  => Self::None,
			"bytes" => Self::Bytes,
			v       => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for AcceptRanges {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::None     => "none",
			Self::Bytes    => "bytes",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum AuthorizationType {
	Basic,
	Bearer,
	Digest,
	Hoba,
	Mutual,
	Negotiate,
	OAuth,
	ScramSha1,
	ScramSha256,
	Vapid,
	Other(Box<str>)
}

impl FromStr for AuthorizationType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Basic"         => Self::Basic,
			"Bearer"        => Self::Bearer,
			"Digest"        => Self::Digest,
			"HOBA"          => Self::Hoba,
			"Mutual"        => Self::Mutual,
			"Negotiate"     => Self::Negotiate,
			"OAuth"         => Self::OAuth,
			"SCRAM-SHA-1"   => Self::ScramSha1,
			"SCRAM-SHA-256" => Self::ScramSha256,
			"vapid"         => Self::Vapid,
			v               => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for AuthorizationType {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Basic       => "Basic",
			Self::Bearer      => "Bearer",
			Self::Digest      => "Digest",
			Self::Hoba        => "HOBA",
			Self::Mutual      => "Mutual",
			Self::Negotiate   => "Negotiate",
			Self::OAuth       => "OAuth",
			Self::ScramSha1   => "SCRAM-SHA-1",
			Self::ScramSha256 => "SCRAM-SHA-256",
			Self::Vapid       => "vapid",
			Self::Other(v)    => v
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Authorization {
	pub r#type:      AuthorizationType,
	pub credentials: String
}

impl FromStr for Authorization {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut split = s.split_ascii_whitespace();
		Ok(Self {
			r#type: split.next().ok_or(())?.parse().map_err(|_| ())?,
			credentials: split.next().ok_or(())?.to_string(),
		})
	}
}

impl Display for Authorization {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{} {}", self.r#type, self.credentials)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CacheControl {
	MaxAge(u32),
	MaxStale(u32),
	MinFresh(u32),
	NoCache,
	NoStore,
	NoTransform,
	OnlyIfCached,
	MustRevalidate,
	Public,
	Private,
	ProxyRevalidate,
	SMaxage(u32),
	Immutable,
	StaleWhileRevalidate(u32),
	StaleIfError(u32)
}

impl FromStr for CacheControl {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match &s[..s.find('=').unwrap_or(s.len())] {
			"max-age"                => Self::MaxAge(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			"max-stale"              => Self::MaxStale(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			"min-fresh"              => Self::MinFresh(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			"no-cache"               => Self::NoCache,
			"no-store"               => Self::NoStore,
			"no-transform"           => Self::NoTransform,
			"only-if-cached"         => Self::OnlyIfCached,
			"must-revalidate"        => Self::MustRevalidate,
			"public"                 => Self::Public,
			"private"                => Self::Private,
			"proxy-revalidate"       => Self::ProxyRevalidate,
			"s-maxage"               => Self::SMaxage(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			"immutable"              => Self::Immutable,
			"stale-while-revalidate" => Self::StaleWhileRevalidate(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			"stale-if-error"         => Self::StaleIfError(s[s.find('=')
				.ok_or(())? + 1..].parse().map_err(|_| ())?),
			_ => return Err(())
		})
	}
}

impl Display for CacheControl {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::MaxAge(value)             => write!(f, "max-age={}", value),
			Self::MaxStale(value)           => write!(f, "max-stale={}", value),
			Self::MinFresh(value)           => write!(f, "min-fresh={}", value),
			Self::NoCache                   => f.write_str("no-cache"),
			Self::NoStore                   => f.write_str("no-store"),
			Self::NoTransform               => f.write_str("no-transform"),
			Self::OnlyIfCached              => f.write_str("only-if-cached"),
			Self::MustRevalidate            => f.write_str("must-revalidate"),
			Self::Public                    => f.write_str("public"),
			Self::Private                   => f.write_str("private"),
			Self::ProxyRevalidate           => f.write_str("proxy-revalidate"),
			Self::SMaxage(value)            => write!(f, "s-maxage={}", value),
			Self::Immutable                 => f.write_str("nimmutable"),
			Self::StaleWhileRevalidate(_)   => f.write_str("stale-while-revalidate"),
			Self::StaleIfError(value)       => write!(f, "stale-if-error={}", value),
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ContentRange {
	pub unit:  Unit,
	pub range: Option<(usize, usize)>,
	pub size:  Option<usize>
}

impl FromStr for ContentRange {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut split = s.split(|ch| ch == ' ' || ch == '/');
		Ok(Self {
			unit: split.next().ok_or(())?.parse()?,
			range: match split.next().ok_or(())? {
				"*" => None,
				str => {
					let mut split = str.split('-');
					Some((split.next().ok_or(())?.parse().map_err(|_| ())?,
						  split.next().ok_or(())?.parse().map_err(|_| ())?))
				}
			},
			size: match split.next().ok_or(())? {
				"*" => None,
				str => Some(str.parse().map_err(|_| ())?)
			}
		})
	}
}

impl Display for ContentRange {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.unit)?;
		match &self.range {
			Some((start, end)) => write!(f, " {}-{}", start, end),
			None => f.write_str(" *")
		}?;
		match &self.size {
			Some(size) => write!(f, "/{}", size),
			None => f.write_str("/*"),
		}?;
		Ok(())
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ClearSiteDataDirective {
	Cache,
	Cookies,
	Storage,
	ExecutionContext,
	All
}

impl FromStr for ClearSiteDataDirective {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"cache"            => Self::Cache,
			"cookies"          => Self::Cookies,
			"storage"          => Self::Storage,
			"executionContext" => Self::ExecutionContext,
			"*"                => Self::All,
			_                  => return Err(())
		})
	}
}

impl Display for ClearSiteDataDirective {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Cache            => "cache",
			Self::Cookies          => "cookies",
			Self::Storage          => "storage",
			Self::ExecutionContext => "executionContext",
			Self::All              => "*",
		})
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct KeepAlive {
	pub timeout: usize,
	pub max:     usize
}

impl FromStr for KeepAlive {
	type Err = ();

	fn from_str(_s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl Display for KeepAlive {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "timeout={}, max={}", self.timeout, self.max)
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ETag {
	pub weak_validator: bool,
	pub value:          String
}

impl FromStr for ETag {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self {
			weak_validator: s.starts_with("W/"),
			value: s[s.find('"').ok_or(())? + 1 .. s.rfind('"').ok_or(())?].to_string()
		})
	}
}

impl Display for ETag {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}\"{}\"", if self.weak_validator { "W/" } else { "" }, self.value)
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SetCookie {
	pub name:      String,
	pub value:     String,
	pub expires:   Option<DateTime<Utc>>,
	pub max_age:   Option<usize>,
	pub domain:    Option<String>,
	pub path:      Option<String>,
	pub secure:    bool,
	pub http_only: bool,
	pub same_site: Option<SameSite>
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SameSite {
	Strict,
	Lax,
	None
}

impl Display for SetCookie {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}={}", self.name, self.value)?;

		if let Some(expires) = self.expires.as_ref() {
			write!(f, "; Expires={}", expires.format(DATE_FORMAT))?
		}

		if let Some(max_age) = self.max_age.as_ref() {
			write!(f, "; Max-Age={}", max_age)?;
		}

		if let Some(domain) = self.domain.as_ref()  {
			write!(f, "; Domain={}", domain)?;
		}

		if let Some(path) = self.path.as_ref() {
			write!(f, "; Path={}", path)?;
		}

		if self.secure {
			f.write_str("; Secure")?;
		}

		if self.http_only {
			f.write_str("; HttpOnly")?;
		}

		if let Some(same_site) = self.same_site.as_ref() {
			write!(f, "; SameSite={:?}", same_site)?;
		}

		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ranges {
	pub unit:   Unit,
	pub ranges: Vec<Range>
}

impl FromStr for Ranges {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (unit, s) = s.split_once('=').ok_or(())?;
		Ok(Self {
			unit:   Unit::from_str(unit)?,
			ranges: s.split(',')
				.map(|s| Some(match s.trim().split_once('-')? {
					("",    "")  => Range { start: None,                      end: None },
					("",    end) => Range { start: None,                      end: Some(end.parse().ok()?) },
					(start, "")  => Range { start: Some(start.parse().ok()?), end: None },
					(start, end) => Range { start: Some(start.parse().ok()?), end: Some(end.parse().ok()?) }
				}))
				.collect::<Option<_>>()
				.ok_or(())?
		})
	}
}

impl Display for Ranges {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}=", &self.unit)?;

		for range in &self.ranges  {
			match range {
				Range { start: Some(start), end: None }      => write!(f, "{}-, ", start),
				Range { start: None,        end: Some(end) } => write!(f, "-{}, ", end),
				Range { start: Some(start), end: Some(end) } => write!(f, "{}-{}, ", start, end),
				Range { start: None,        end: None }      => Ok(())
			}?;
		}

		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Range {
	pub start: Option<usize>,
	pub end:   Option<usize>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RetryAfter {
	Date(DateTime<Utc>),
	Seconds(usize)
}

impl FromStr for RetryAfter {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.chars().next().ok_or(())?.is_ascii_digit() {
			s.parse().map(Self::Seconds).map_err(|_| ())
		} else {
			parse_date(s).map(Self::Date).map_err(|_| ())
		}
	}
}

impl Display for RetryAfter {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Date(date)       => write!(f, "{}", date.format(DATE_FORMAT)),
			Self::Seconds(seconds) => write!(f, "{}", seconds)
		}
	}
}

pub struct MessageBuilder<H: Extend<Header>, D>(H, D);

impl<T: Extend<Header>> From<T> for MessageBuilder<T, [u8; 0]> {
	fn from(buf: T) -> Self {
		Self(buf, [])
	}
}

impl Default for MessageBuilder<Vec<Header>, [u8; 0]> {
	fn default() -> Self {
		Self(Vec::new(), [])
	}
}

impl MessageBuilder<Vec<Header>, [u8; 0]> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<H: Extend<Header>, D> MessageBuilder<H, D> {
	pub fn body<T>(self, data: T) -> MessageBuilder<H, T> {
		MessageBuilder(self.0, data)
	}

	pub fn send(&mut self, stream: &mut (impl traits::Stream + ?Sized)) -> std::io::Result<()> where H: AsRef<[Header]>, D: AsRef<[u8]> {
		stream.write_headers(self.0.as_ref())?;
		stream.write_all(self.1.as_ref())?;
		Ok(())
	}

	pub async fn send_async(&mut self, stream: &mut (impl traits::AsyncStream + ?Sized)) -> std::io::Result<()> where H: AsRef<[Header]>, D: AsRef<[u8]> {
		use {futures_lite::io::AsyncWriteExt, traits::AsyncStreamExt};
		stream.write_headers(self.0.as_ref()).await?;
		stream.write_all(self.1.as_ref()).await?;
		Ok(())
	}

/*

REGEX:

	pub fn $1(self, v: $2) -> Self {
		self.0.extend(std::iter::once(Header::$1(v)));
		self
	}


 */

	pub fn accept_charset(mut self, v: Vec<Charset>) -> Self {
		self.0.extend(std::iter::once(Header::AcceptCharset(v)));
		self
	}

	pub fn accept_encoding(mut self, v: Vec<Encoding>) -> Self {
		self.0.extend(std::iter::once(Header::AcceptEncoding(v)));
		self
	}

	pub fn accept_language(mut self, v: Vec<Language>) -> Self {
		self.0.extend(std::iter::once(Header::AcceptLanguage(v)));
		self
	}

	pub fn accept_ranges(mut self, v: AcceptRanges) -> Self {
		self.0.extend(std::iter::once(Header::AcceptRanges(v)));
		self
	}

	pub fn accept_types(mut self, v: Vec<MediaType>) -> Self {
		self.0.extend(std::iter::once(Header::AcceptTypes(v)));
		self
	}

	pub fn access_control_allow_credentials(mut self, v: bool) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowCredentials(v)));
		self
	}

	pub fn access_control_allow_headers(mut self, v: Vec<HeaderId>) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowHeaders(v)));
		self
	}

	pub fn access_control_allow_methods(mut self, v: Vec<Method>) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowMethods(v)));
		self
	}

	pub fn access_control_allow_origin(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowOrigin(v)));
		self
	}

	pub fn access_control_allow_expose_headers(mut self, v: Vec<HeaderId>) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowExposeHeaders(v)));
		self
	}

	pub fn access_control_allow_max_age(mut self, v: usize) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowMaxAge(v)));
		self
	}

	pub fn access_control_allow_request_headers(mut self, v: Vec<HeaderId>) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowRequestHeaders(v)));
		self
	}

	pub fn access_control_allow_request_method(mut self, v: Method) -> Self {
		self.0.extend(std::iter::once(Header::AccessControlAllowRequestMethod(v)));
		self
	}

	pub fn allow(mut self, v: Vec<Method>) -> Self {
		self.0.extend(std::iter::once(Header::Allow(v)));
		self
	}

	pub fn authority(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Authority(v)));
		self
	}

	pub fn authorization(mut self, v: Authorization) -> Self {
		self.0.extend(std::iter::once(Header::Authorization(v)));
		self
	}

	pub fn cache_control(mut self, v: CacheControl) -> Self {
		self.0.extend(std::iter::once(Header::CacheControl(v)));
		self
	}

	pub fn clear_site_data(mut self, v: ClearSiteDataDirective) -> Self {
		self.0.extend(std::iter::once(Header::ClearSiteData(v)));
		self
	}

	pub fn connection(mut self, v: Connection) -> Self {
		self.0.extend(std::iter::once(Header::Connection(v)));
		self
	}

	pub fn content_disposition(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::ContentDisposition(v)));
		self
	}

	pub fn content_encoding(mut self, v: Encoding) -> Self {
		self.0.extend(std::iter::once(Header::ContentEncoding(v)));
		self
	}

	pub fn content_language(mut self, v: Language) -> Self {
		self.0.extend(std::iter::once(Header::ContentLanguage(v)));
		self
	}

	pub fn content_length(mut self, v: usize) -> Self {
		self.0.extend(std::iter::once(Header::ContentLength(v)));
		self
	}

	pub fn content_location(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::ContentLocation(v)));
		self
	}

	pub fn content_range(mut self, v: ContentRange) -> Self {
		self.0.extend(std::iter::once(Header::ContentRange(v)));
		self
	}

	pub fn content_type(mut self, v: Box<MediaType>) -> Self {
		self.0.extend(std::iter::once(Header::ContentType(v)));
		self
	}

	pub fn cookie(mut self, v: Vec<(String, String)>) -> Self {
		self.0.extend(std::iter::once(Header::Cookie(v)));
		self
	}

	pub fn dnt(mut self, v: bool) -> Self {
		self.0.extend(std::iter::once(Header::Dnt(v)));
		self
	}

	pub fn date(mut self, v: DateTime<Utc>) -> Self {
		self.0.extend(std::iter::once(Header::Date(v)));
		self
	}

	pub fn e_tag(mut self, v: ETag) -> Self {
		self.0.extend(std::iter::once(Header::ETag(v)));
		self
	}

	pub fn expires(mut self, v: DateTime<Utc>) -> Self {
		self.0.extend(std::iter::once(Header::Expires(v)));
		self
	}

	pub fn host(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Host(v)));
		self
	}

	pub fn if_match(mut self, v: Vec<String>) -> Self {
		self.0.extend(std::iter::once(Header::IfMatch(v)));
		self
	}

	pub fn if_modified_since(mut self, v: DateTime<Utc>) -> Self {
		self.0.extend(std::iter::once(Header::IfModifiedSince(v)));
		self
	}

	pub fn if_none_match(mut self, v: Vec<String>) -> Self {
		self.0.extend(std::iter::once(Header::IfNoneMatch(v)));
		self
	}

	pub fn if_range(mut self, v: ()) -> Self {
		self.0.extend(std::iter::once(Header::IfRange(v)));
		self
	}

	pub fn if_unmodified_since(mut self, v: DateTime<Utc>) -> Self {
		self.0.extend(std::iter::once(Header::IfUnmodifiedSince(v)));
		self
	}

	pub fn keep_alive(mut self, v: KeepAlive) -> Self {
		self.0.extend(std::iter::once(Header::KeepAlive(v)));
		self
	}

	pub fn last_modified(mut self, v: DateTime<Utc>) -> Self {
		self.0.extend(std::iter::once(Header::LastModified(v)));
		self
	}

	pub fn location(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Location(v)));
		self
	}

	pub fn method(mut self, v: Method) -> Self {
		self.0.extend(std::iter::once(Header::Method(v)));
		self
	}

	pub fn path(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Path(v)));
		self
	}

	pub fn range(mut self, v: Ranges) -> Self {
		self.0.extend(std::iter::once(Header::Range(v)));
		self
	}

	pub fn referer(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Referer(v)));
		self
	}

	pub fn retry_after(mut self, v: RetryAfter) -> Self {
		self.0.extend(std::iter::once(Header::RetryAfter(v)));
		self
	}

	pub fn save_data(mut self, v: bool) -> Self {
		self.0.extend(std::iter::once(Header::SaveData(v)));
		self
	}
	pub fn scheme(mut self, v: Scheme) -> Self {
		self.0.extend(std::iter::once(Header::Scheme(v)));
		self
	}

	pub fn server(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Server(v)));
		self
	}

	pub fn set_cookie(mut self, v: SetCookie) -> Self {
		self.0.extend(std::iter::once(Header::SetCookie(v)));
		self
	}

	pub fn status(mut self, v: Status) -> Self {
		self.0.extend(std::iter::once(Header::Status(v)));
		self
	}

	pub fn upgrade(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::Upgrade(v)));
		self
	}

	pub fn upgrade_insecure_requests(mut self, v: bool) -> Self {
		self.0.extend(std::iter::once(Header::UpgradeInsecureRequests(v)));
		self
	}

	pub fn user_agent(mut self, v: String) -> Self {
		self.0.extend(std::iter::once(Header::UserAgent(v)));
		self
	}

	pub fn vary(mut self, v: Vec<HeaderId>) -> Self {
		self.0.extend(std::iter::once(Header::Vary(v)));
		self
	}

	pub fn unknown(mut self, key: String, val: String) -> Self {
		self.0.extend(std::iter::once(Header::Custom(key, val)));
		self
	}
}