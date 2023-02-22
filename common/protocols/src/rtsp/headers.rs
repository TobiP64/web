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
		fmt::{Display, Formatter, Result as FmtResult},
		convert::TryFrom,
		net::IpAddr
	},
	chrono::{DateTime, Utc}
};

pub use crate::utils::headers::*;

pub type Url = String;

static NAMES: [&str; 60] = [
	"Method",
	"Status",
	"Accept",
	"Accept-Credentials",
	"Accept-Encoding",
	"Accept-Language",
	"Accept-Ranges",
	"Allow",
	"Authentication-Info",
	"Authorization",
	"Bandwidth",
	"Block-Size",
	"Cache-Control",
	"Connection",
	"Connection-Credentials",
	"Content-Base",
	"Content-Encoding",
	"Content-Language",
	"Content-Length",
	"Content-Location",
	"Content-Type",
	"CSeq",
	"Date",
	"Expires",
	"From",
	"If-Match",
	"If-Modified-Since",
	"If-None-Match",
	"Last-Modified",
	"Location",
	"Media-Properties",
	"Media-Range",
	"MTag",
	"Notify-Reason",
	"Pipeline-Requests",
	"Proxy-Authentication",
	"Proxy-AuthenticationInfo",
	"Proxy-Authorization",
	"Proxy-Require",
	"Proxy-Supported",
	"Public",
	"Range",
	"Referrer",
	"Request-Status",
	"Require",
	"Retry-After",
	"RTP-Info",
	"Scale",
	"Seek-Style",
	"Server",
	"Session",
	"Speed",
	"Supported",
	"Terminate-Reason",
	"Timestamp",
	"Transport",
	"Unsupported",
	"User-Agent",
	"Via",
	"WWW-Authenticate"
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HeaderId {
	Method,
	Status,
	Accept,
	AcceptCredentials,
	AcceptEncoding,
	AcceptLanguage,
	AcceptRanges,
	Allow,
	AuthenticationInfo,
	Authorization,
	Bandwidth,
	BlockSize,
	CacheControl,
	Connection,
	ConnectionCredentials,
	ContentBase,
	ContentEncoding,
	ContentLanguage,
	ContentLength,
	ContentLocation,
	ContentType,
	CSeq,
	Date,
	Expires,
	From,
	IfMatch,
	IfModifiedSince,
	IfNoneMatch,
	LastModified,
	Location,
	MediaProperties,
	MediaRange,
	MTag,
	NotifyReason,
	PipelineRequests,
	ProxyAuthentication,
	ProxyAuthenticationInfo,
	ProxyAuthorization,
	ProxyRequire,
	ProxySupported,
	Public,
	Range,
	Referrer,
	RequestStatus,
	Require,
	RetryAfter,
	RtpInfo,
	Scale,
	SeekStyle,
	Server,
	Session,
	Speed,
	Supported,
	TerminateReason,
	Timestamp,
	Transport,
	Unsupported,
	UserAgent,
	Via,
	WwwAuthenticate
}

impl HeaderId {
	pub fn parse_value(self, v: &str) -> Result<Header, ()> {
		Header::parse_id_value(self, v)
	}
	
	pub fn name(self) -> &'static str {
		NAMES[self as usize]
	}
}

impl FromStr for HeaderId {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Method"                   => Self::Method,
			"Status"                   => Self::Status,
			"Accept"                   => Self::Accept,
			"Accept-Credentials"       => Self::AcceptCredentials,
			"Accept-Encoding"          => Self::AcceptEncoding,
			"Accept-Language"          => Self::AcceptLanguage,
			"Accept-Ranges"            => Self::AcceptRanges,
			"Allow"                    => Self::Allow,
			"Authentication-Info"      => Self::AuthenticationInfo,
			"Authorization"            => Self::Authorization,
			"Bandwidth"                => Self::Bandwidth,
			"Block-Size"               => Self::BlockSize,
			"Cache-Control"            => Self::CacheControl,
			"Connection"               => Self::Connection,
			"Connection-Credentials"   => Self::ConnectionCredentials,
			"Content-Base"             => Self::ContentBase,
			"Content-Encoding"         => Self::ContentEncoding,
			"Content-Language"         => Self::ContentLanguage,
			"Content-Length"           => Self::ContentLength,
			"Content-Location"         => Self::ContentLocation,
			"Content-Type"             => Self::ContentType,
			"CSeq"                     => Self::CSeq,
			"Date"                     => Self::Date,
			"Expires"                  => Self::Expires,
			"From"                     => Self::From,
			"If-Match"                 => Self::IfMatch,
			"If-Modified-Since"        => Self::IfModifiedSince,
			"If-None-Match"            => Self::IfNoneMatch,
			"Last-Modified"            => Self::LastModified,
			"Location"                 => Self::Location,
			"Media-Properties"         => Self::MediaProperties,
			"Media-Range"              => Self::MediaRange,
			"MTag"                     => Self::MTag,
			"Notify-Reason"            => Self::NotifyReason,
			"Pipeline-Requests"        => Self::PipelineRequests,
			"Proxy-Authentication"     => Self::ProxyAuthentication,
			"Proxy-AuthenticationInfo" => Self::ProxyAuthenticationInfo,
			"Proxy-Authorization"      => Self::ProxyAuthorization,
			"Proxy-Require"            => Self::ProxyRequire,
			"Proxy-Supported"          => Self::ProxySupported,
			"Public"                   => Self::Public,
			"Range"                    => Self::Range,
			"Referrer"                 => Self::Referrer,
			"Request-Status"           => Self::RequestStatus,
			"Require"                  => Self::Require,
			"Retry-After"              => Self::RetryAfter,
			"RTP-Info"                 => Self::RtpInfo,
			"Scale"                    => Self::Scale,
			"Seek-Style"               => Self::SeekStyle,
			"Server"                   => Self::Server,
			"Session"                  => Self::Session,
			"Speed"                    => Self::Speed,
			"Supported"                => Self::Supported,
			"Terminate-Reason"         => Self::TerminateReason,
			"Timestamp"                => Self::Timestamp,
			"Transport"                => Self::Transport,
			"Unsupported"              => Self::Unsupported,
			"User-Agent"               => Self::UserAgent,
			"Via"                      => Self::Via,
			"WWW-Authenticate"         => Self::WwwAuthenticate,
			_                          => return Err(())
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Header {
	Method(Method),
	Status(Status),
	Accept(Vec<MediaType>),
	AcceptCredentials(AcceptCredentials),
	AcceptEncoding(Vec<Encoding>),
	AcceptLanguage(Vec<Language>),
	AcceptRanges(Vec<MediaTypeRanges>),
	Allow(Vec<Method>),
	AuthenticationInfo(String),
	Authorization(String),
	Bandwidth(usize),
	BlockSize(usize),
	CacheControl(CacheControl),
	Connection(Vec<Connection>),
	ConnectionCredentials(ConnectionCredentials),
	ContentBase(Url),
	ContentEncoding(Encoding),
	ContentLanguage(Language),
	ContentLength(usize),
	ContentLocation(Url),
	ContentType(MediaType),
	CSeq(usize),
	Date(DateTime<Utc>),
	Expires(DateTime<Utc>),
	From(String),
	IfMatch(String),
	IfModifiedSince(DateTime<Utc>),
	IfNoneMatch(String),
	LastModified(DateTime<Utc>),
	Location(Url),
	MediaProperties(Vec<MediaProperty>),
	MediaRange(Vec<String>),
	MTag(MTag),
	NotifyReason(NotifyReason),
	PipelineRequests(u32),
	ProxyAuthentication(String),
	ProxyAuthenticationInfo(String),
	ProxyAuthorization(String),
	ProxyRequire(String),
	ProxySupported(Vec<String>),
	Public(Vec<Method>),
	Range(String),
	Referrer(Url),
	RequestStatus(RequestStatus),
	Require(String),
	RetryAfter(RetryAfter),
	RtpInfo(Vec<RtpInfo>),
	Scale(f32),
	SeekStyle(SeekStyle),
	Server(String),
	Session(String),
	Speed(Speed),
	Supported(Vec<String>),
	TerminateReason(TerminateReason),
	Timestamp(Timestamp),
	Transport(Vec<Transport>),
	Unsupported(Vec<String>),
	UserAgent(String),
	Via(Vec<Via>),
	WwwAuthenticate(String),
	Custom(String, String)
}

impl Header {
	pub fn parse_name_value(name: &str, value: &str) -> Self {
		match name.parse::<HeaderId>().and_then(|id| Self::parse_id_value(id, value)) {
			Ok(v)  => v,
			Err(_) => Self::Custom(
				name.to_string(),
				value.to_string()
			)
		}
	}
	
	pub fn parse_id_value(id: HeaderId, v: &str) -> Result<Self, ()> {
		Ok(match id {
			HeaderId::Method                  => Self::Method(v.parse()?),
			HeaderId::Status                  => Self::Status(v.parse()?),
			HeaderId::Accept                  => Self::Accept(parse_list(v)?),
			HeaderId::AcceptCredentials       => Self::AcceptCredentials(v.parse()?),
			HeaderId::AcceptEncoding          => Self::AcceptEncoding(parse_list(v)?),
			HeaderId::AcceptLanguage          => Self::AcceptLanguage(parse_list(v)?),
			HeaderId::AcceptRanges            => Self::AcceptRanges(parse_list(v)?),
			HeaderId::Allow                   => Self::Allow(parse_list(v)?),
			HeaderId::AuthenticationInfo      => Self::AuthenticationInfo(v.to_string()),
			HeaderId::Authorization           => Self::Authorization(v.to_string()),
			HeaderId::Bandwidth               => Self::Bandwidth(v.parse().map_err(|_| ())?),
			HeaderId::BlockSize               => Self::BlockSize(v.parse().map_err(|_| ())?),
			HeaderId::CacheControl            => Self::CacheControl(v.parse()?),
			HeaderId::Connection              => Self::Connection(parse_list(v)?),
			HeaderId::ConnectionCredentials   => Self::ConnectionCredentials(v.parse()?),
			HeaderId::ContentBase             => Self::ContentBase(v.to_string()),
			HeaderId::ContentEncoding         => Self::ContentEncoding(v.parse()?),
			HeaderId::ContentLanguage         => Self::ContentLanguage(v.parse()?),
			HeaderId::ContentLength           => Self::ContentLength(v.parse().map_err(|_| ())?),
			HeaderId::ContentLocation         => Self::ContentLocation(v.to_string()),
			HeaderId::ContentType             => Self::ContentType(v.parse()?),
			HeaderId::CSeq                    => Self::CSeq(v.parse().map_err(|_| ())?),
			HeaderId::Date                    => Self::Date(v.parse().map_err(|_| ())?),
			HeaderId::Expires                 => Self::Expires(v.parse().map_err(|_| ())?),
			HeaderId::From                    => Self::From(v.to_string()),
			HeaderId::IfMatch                 => Self::IfMatch(v.to_string()),
			HeaderId::IfModifiedSince         => Self::IfModifiedSince(v.parse().map_err(|_| ())?),
			HeaderId::IfNoneMatch             => Self::IfNoneMatch(v.to_string()),
			HeaderId::LastModified            => Self::LastModified(v.parse().map_err(|_| ())?),
			HeaderId::Location                => Self::Location(v.to_string()),
			HeaderId::MediaProperties         => Self::MediaProperties(parse_list(v)?),
			HeaderId::MediaRange              => Self::MediaRange(parse_list(v)?),
			HeaderId::MTag                    => Self::MTag(v.parse()?),
			HeaderId::NotifyReason            => Self::NotifyReason(v.parse()?),
			HeaderId::PipelineRequests        => Self::PipelineRequests(v.parse().map_err(|_| ())?),
			HeaderId::ProxyAuthentication     => Self::ProxyAuthentication(v.to_string()),
			HeaderId::ProxyAuthenticationInfo => Self::ProxyAuthenticationInfo(v.to_string()),
			HeaderId::ProxyAuthorization      => Self::ProxyAuthorization(v.to_string()),
			HeaderId::ProxyRequire            => Self::ProxyRequire(v.to_string()),
			HeaderId::ProxySupported          => Self::ProxySupported(parse_list(v)?),
			HeaderId::Public                  => Self::Public(parse_list(v)?),
			HeaderId::Range                   => Self::Range(v.to_string()),
			HeaderId::Referrer                => Self::Referrer(v.to_string()),
			HeaderId::RequestStatus           => Self::RequestStatus(v.parse()?),
			HeaderId::Require                 => Self::Require(v.to_string()),
			HeaderId::RetryAfter              => Self::RetryAfter(v.parse()?),
			HeaderId::RtpInfo                 => Self::RtpInfo(parse_list(v)?),
			HeaderId::Scale                   => Self::Scale(v.parse().map_err(|_| ())?),
			HeaderId::SeekStyle               => Self::SeekStyle(v.parse()?),
			HeaderId::Server                  => Self::Server(v.to_string()),
			HeaderId::Session                 => Self::Session(v.to_string()),
			HeaderId::Speed                   => Self::Speed(v.parse()?),
			HeaderId::Supported               => Self::Supported(parse_list(v)?),
			HeaderId::TerminateReason         => Self::TerminateReason(v.parse()?),
			HeaderId::Timestamp               => Self::Timestamp(v.parse()?),
			HeaderId::Transport               => Self::Transport(parse_list(v)?),
			HeaderId::Unsupported             => Self::Unsupported(parse_list(v)?),
			HeaderId::UserAgent               => Self::UserAgent(v.to_string()),
			HeaderId::Via                     => Self::Via(parse_list(v)?),
			HeaderId::WwwAuthenticate         => Self::WwwAuthenticate(v.to_string())
		})
	}
	
	pub fn id(&self) -> Result<HeaderId, &str> {
		Ok(match self {
			Self::Method(_)                  => HeaderId::Method,
			Self::Status(_)                  => HeaderId::Status,
			Self::Accept(_)                  => HeaderId::Accept,
			Self::AcceptCredentials(_)       => HeaderId::AcceptCredentials,
			Self::AcceptEncoding(_)          => HeaderId::AcceptEncoding,
			Self::AcceptLanguage(_)          => HeaderId::AcceptLanguage,
			Self::AcceptRanges(_)            => HeaderId::AcceptRanges,
			Self::Allow(_)                   => HeaderId::Allow,
			Self::AuthenticationInfo(_)      => HeaderId::AuthenticationInfo,
			Self::Authorization(_)           => HeaderId::Authorization,
			Self::Bandwidth(_)               => HeaderId::Bandwidth,
			Self::BlockSize(_)               => HeaderId::BlockSize,
			Self::CacheControl(_)            => HeaderId::CacheControl,
			Self::Connection(_)              => HeaderId::Connection,
			Self::ConnectionCredentials(_)   => HeaderId::ConnectionCredentials,
			Self::ContentBase(_)             => HeaderId::ContentBase,
			Self::ContentEncoding(_)         => HeaderId::ContentEncoding,
			Self::ContentLanguage(_)         => HeaderId::ContentLanguage,
			Self::ContentLength(_)           => HeaderId::ContentLength,
			Self::ContentLocation(_)         => HeaderId::ContentLocation,
			Self::ContentType(_)             => HeaderId::ContentType,
			Self::CSeq(_)                    => HeaderId::CSeq,
			Self::Date(_)                    => HeaderId::Date,
			Self::Expires(_)                 => HeaderId::Expires,
			Self::From(_)                    => HeaderId::From,
			Self::IfMatch(_)                 => HeaderId::IfMatch,
			Self::IfModifiedSince(_)         => HeaderId::IfModifiedSince,
			Self::IfNoneMatch(_)             => HeaderId::IfNoneMatch,
			Self::LastModified(_)            => HeaderId::LastModified,
			Self::Location(_)                => HeaderId::Location,
			Self::MediaProperties(_)         => HeaderId::MediaProperties,
			Self::MediaRange(_)              => HeaderId::MediaRange,
			Self::MTag(_)                    => HeaderId::MTag,
			Self::NotifyReason(_)            => HeaderId::NotifyReason,
			Self::PipelineRequests(_)        => HeaderId::PipelineRequests,
			Self::ProxyAuthentication(_)     => HeaderId::ProxyAuthentication,
			Self::ProxyAuthenticationInfo(_) => HeaderId::ProxyAuthenticationInfo,
			Self::ProxyAuthorization(_)      => HeaderId::ProxyAuthorization,
			Self::ProxyRequire(_)            => HeaderId::ProxyRequire,
			Self::ProxySupported(_)          => HeaderId::ProxySupported,
			Self::Public(_)                  => HeaderId::Public,
			Self::Range(_)                   => HeaderId::Range,
			Self::Referrer(_)                => HeaderId::Referrer,
			Self::RequestStatus(_)           => HeaderId::RequestStatus,
			Self::Require(_)                 => HeaderId::Require,
			Self::RetryAfter(_)              => HeaderId::RetryAfter,
			Self::RtpInfo(_)                 => HeaderId::RtpInfo,
			Self::Scale(_)                   => HeaderId::Scale,
			Self::SeekStyle(_)               => HeaderId::SeekStyle,
			Self::Server(_)                  => HeaderId::Server,
			Self::Session(_)                 => HeaderId::Session,
			Self::Speed(_)                   => HeaderId::Speed,
			Self::Supported(_)               => HeaderId::Supported,
			Self::TerminateReason(_)         => HeaderId::TerminateReason,
			Self::Timestamp(_)               => HeaderId::Timestamp,
			Self::Transport(_)               => HeaderId::Transport,
			Self::Unsupported(_)             => HeaderId::Unsupported,
			Self::UserAgent(_)               => HeaderId::UserAgent,
			Self::Via(_)                     => HeaderId::Via,
			Self::WwwAuthenticate(_)         => HeaderId::WwwAuthenticate,
			Self::Custom(name, _)            => return Err(name)
		})
	}
	
	pub fn name(&self) -> &str {
		crate::utils::unstable::_82223_into_ok_or_err(self.id().map(HeaderId::name))
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
	
	pub fn as_accept(&self) -> Option<&Vec<MediaType>> {
		match self {
			Self::Accept(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_accept(&mut self) -> Option<&mut Vec<MediaType>> {
		match self {
			Self::Accept(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_accept(self) -> Option<Vec<MediaType>> {
		match self {
			Self::Accept(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_accept_credentials(&self) -> Option<&AcceptCredentials> {
		match self {
			Self::AcceptCredentials(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_accept_credentials(&mut self) -> Option<&mut AcceptCredentials> {
		match self {
			Self::AcceptCredentials(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_accept_credentials(self) -> Option<AcceptCredentials> {
		match self {
			Self::AcceptCredentials(v) => Some(v),
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
	
	pub fn as_accept_ranges(&self) -> Option<&Vec<MediaTypeRanges>> {
		match self {
			Self::AcceptRanges(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_accept_ranges(&mut self) -> Option<&mut Vec<MediaTypeRanges>> {
		match self {
			Self::AcceptRanges(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_accept_ranges(self) -> Option<Vec<MediaTypeRanges>> {
		match self {
			Self::AcceptRanges(v) => Some(v),
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
	
	pub fn as_authentication_info(&self) -> Option<&String> {
		match self {
			Self::AuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_authentication_info(&mut self) -> Option<&mut String> {
		match self {
			Self::AuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_authentication_info(self) -> Option<String> {
		match self {
			Self::AuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_authorization(&self) -> Option<&String> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_authorization(&mut self) -> Option<&mut String> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_authorization(self) -> Option<String> {
		match self {
			Self::Authorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_bandwidth(&self) -> Option<&usize> {
		match self {
			Self::Bandwidth(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_bandwidth(&mut self) -> Option<&mut usize> {
		match self {
			Self::Bandwidth(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_bandwidth(self) -> Option<usize> {
		match self {
			Self::Bandwidth(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_block_size(&self) -> Option<&usize> {
		match self {
			Self::BlockSize(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_block_size(&mut self) -> Option<&mut usize> {
		match self {
			Self::BlockSize(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_block_size(self) -> Option<usize> {
		match self {
			Self::BlockSize(v) => Some(v),
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
	
	pub fn as_connection(&self) -> Option<&Vec<Connection>> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_connection(&mut self) -> Option<&mut Vec<Connection>> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_connection(self) -> Option<Vec<Connection>> {
		match self {
			Self::Connection(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_connection_credentials(&self) -> Option<&ConnectionCredentials> {
		match self {
			Self::ConnectionCredentials(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_connection_credentials(&mut self) -> Option<&mut ConnectionCredentials> {
		match self {
			Self::ConnectionCredentials(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_connection_credentials(self) -> Option<ConnectionCredentials> {
		match self {
			Self::ConnectionCredentials(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_content_base(&self) -> Option<&Url> {
		match self {
			Self::ContentBase(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_content_base(&mut self) -> Option<&mut Url> {
		match self {
			Self::ContentBase(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_content_base(self) -> Option<Url> {
		match self {
			Self::ContentBase(v) => Some(v),
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
	
	pub fn as_content_location(&self) -> Option<&Url> {
		match self {
			Self::ContentLocation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_content_location(&mut self) -> Option<&mut Url> {
		match self {
			Self::ContentLocation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_content_location(self) -> Option<Url> {
		match self {
			Self::ContentLocation(v) => Some(v),
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
	
	pub fn into_content_type(self) -> Option<MediaType> {
		match self {
			Self::ContentType(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_c_seq(&self) -> Option<&usize> {
		match self {
			Self::CSeq(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_c_seq(&mut self) -> Option<&mut usize> {
		match self {
			Self::CSeq(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_c_seq(self) -> Option<usize> {
		match self {
			Self::CSeq(v) => Some(v),
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
	
	pub fn as_from(&self) -> Option<&String> {
		match self {
			Self::From(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_from(&mut self) -> Option<&mut String> {
		match self {
			Self::From(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_from(self) -> Option<String> {
		match self {
			Self::From(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_if_match(&self) -> Option<&String> {
		match self {
			Self::IfMatch(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_if_match(&mut self) -> Option<&mut String> {
		match self {
			Self::IfMatch(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_if_match(self) -> Option<String> {
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
	
	pub fn as_if_none_match(&self) -> Option<&String> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_if_none_match(&mut self) -> Option<&mut String> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_if_none_match(self) -> Option<String> {
		match self {
			Self::IfNoneMatch(v) => Some(v),
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
	
	pub fn as_location(&self) -> Option<&Url> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_location(&mut self) -> Option<&mut Url> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_location(self) -> Option<Url> {
		match self {
			Self::Location(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_media_properties(&self) -> Option<&Vec<MediaProperty>> {
		match self {
			Self::MediaProperties(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_media_properties(&mut self) -> Option<&mut Vec<MediaProperty>> {
		match self {
			Self::MediaProperties(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_media_properties(self) -> Option<Vec<MediaProperty>> {
		match self {
			Self::MediaProperties(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_media_range(&self) -> Option<&Vec<String>> {
		match self {
			Self::MediaRange(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_media_range(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::MediaRange(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_media_range(self) -> Option<Vec<String>> {
		match self {
			Self::MediaRange(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_m_tag(&self) -> Option<&MTag> {
		match self {
			Self::MTag(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_m_tag(&mut self) -> Option<&mut MTag> {
		match self {
			Self::MTag(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_m_tag(self) -> Option<MTag> {
		match self {
			Self::MTag(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_notify_reason(&self) -> Option<&NotifyReason> {
		match self {
			Self::NotifyReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_notify_reason(&mut self) -> Option<&mut NotifyReason> {
		match self {
			Self::NotifyReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_notify_reason(self) -> Option<NotifyReason> {
		match self {
			Self::NotifyReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_pipeline_requests(&self) -> Option<&u32> {
		match self {
			Self::PipelineRequests(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_pipeline_requests(&mut self) -> Option<&mut u32> {
		match self {
			Self::PipelineRequests(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_pipeline_requests(self) -> Option<u32> {
		match self {
			Self::PipelineRequests(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_proxy_authentication(&self) -> Option<&String> {
		match self {
			Self::ProxyAuthentication(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_proxy_authentication(&mut self) -> Option<&mut String> {
		match self {
			Self::ProxyAuthentication(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_proxy_authentication(self) -> Option<String> {
		match self {
			Self::ProxyAuthentication(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_proxy_authentication_info(&self) -> Option<&String> {
		match self {
			Self::ProxyAuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_proxy_authentication_info(&mut self) -> Option<&mut String> {
		match self {
			Self::ProxyAuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_proxy_authentication_info(self) -> Option<String> {
		match self {
			Self::ProxyAuthenticationInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_proxy_authorization(&self) -> Option<&String> {
		match self {
			Self::ProxyAuthorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_proxy_authorization(&mut self) -> Option<&mut String> {
		match self {
			Self::ProxyAuthorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_proxy_authorization(self) -> Option<String> {
		match self {
			Self::ProxyAuthorization(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_proxy_require(&self) -> Option<&String> {
		match self {
			Self::ProxyRequire(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_proxy_require(&mut self) -> Option<&mut String> {
		match self {
			Self::ProxyRequire(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_proxy_require(self) -> Option<String> {
		match self {
			Self::ProxyRequire(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_proxy_supported(&self) -> Option<&Vec<String>> {
		match self {
			Self::ProxySupported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_proxy_supported(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::ProxySupported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_proxy_supported(self) -> Option<Vec<String>> {
		match self {
			Self::ProxySupported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_public(&self) -> Option<&Vec<Method>> {
		match self {
			Self::Public(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_public(&mut self) -> Option<&mut Vec<Method>> {
		match self {
			Self::Public(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_public(self) -> Option<Vec<Method>> {
		match self {
			Self::Public(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_range(&self) -> Option<&String> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_range(&mut self) -> Option<&mut String> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_range(self) -> Option<String> {
		match self {
			Self::Range(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_referrer(&self) -> Option<&Url> {
		match self {
			Self::Referrer(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_referrer(&mut self) -> Option<&mut Url> {
		match self {
			Self::Referrer(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_referrer(self) -> Option<Url> {
		match self {
			Self::Referrer(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_request_status(&self) -> Option<&RequestStatus> {
		match self {
			Self::RequestStatus(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_request_status(&mut self) -> Option<&mut RequestStatus> {
		match self {
			Self::RequestStatus(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_request_status(self) -> Option<RequestStatus> {
		match self {
			Self::RequestStatus(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_require(&self) -> Option<&String> {
		match self {
			Self::Require(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_require(&mut self) -> Option<&mut String> {
		match self {
			Self::Require(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_require(self) -> Option<String> {
		match self {
			Self::Require(v) => Some(v),
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
	
	pub fn as_rtp_info(&self) -> Option<&Vec<RtpInfo>> {
		match self {
			Self::RtpInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_rtp_info(&mut self) -> Option<&mut Vec<RtpInfo>> {
		match self {
			Self::RtpInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_rtp_info(self) -> Option<Vec<RtpInfo>> {
		match self {
			Self::RtpInfo(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_scale(&self) -> Option<&f32> {
		match self {
			Self::Scale(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_scale(&mut self) -> Option<&mut f32> {
		match self {
			Self::Scale(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_scale(self) -> Option<f32> {
		match self {
			Self::Scale(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_seek_style(&self) -> Option<&SeekStyle> {
		match self {
			Self::SeekStyle(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_seek_style(&mut self) -> Option<&mut SeekStyle> {
		match self {
			Self::SeekStyle(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_seek_style(self) -> Option<SeekStyle> {
		match self {
			Self::SeekStyle(v) => Some(v),
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
	
	pub fn as_session(&self) -> Option<&String> {
		match self {
			Self::Session(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_session(&mut self) -> Option<&mut String> {
		match self {
			Self::Session(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_session(self) -> Option<String> {
		match self {
			Self::Session(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_speed(&self) -> Option<&Speed> {
		match self {
			Self::Speed(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_speed(&mut self) -> Option<&mut Speed> {
		match self {
			Self::Speed(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_speed(self) -> Option<Speed> {
		match self {
			Self::Speed(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_supported(&self) -> Option<&Vec<String>> {
		match self {
			Self::Supported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_supported(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::Supported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_supported(self) -> Option<Vec<String>> {
		match self {
			Self::Supported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_terminate_reason(&self) -> Option<&TerminateReason> {
		match self {
			Self::TerminateReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_terminate_reason(&mut self) -> Option<&mut TerminateReason> {
		match self {
			Self::TerminateReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_terminate_reason(self) -> Option<TerminateReason> {
		match self {
			Self::TerminateReason(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_timestamp(&self) -> Option<&Timestamp> {
		match self {
			Self::Timestamp(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_timestamp(&mut self) -> Option<&mut Timestamp> {
		match self {
			Self::Timestamp(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_timestamp(self) -> Option<Timestamp> {
		match self {
			Self::Timestamp(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_transport(&self) -> Option<&Vec<Transport>> {
		match self {
			Self::Transport(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_transport(&mut self) -> Option<&mut Vec<Transport>> {
		match self {
			Self::Transport(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_transport(self) -> Option<Vec<Transport>> {
		match self {
			Self::Transport(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_unsupported(&self) -> Option<&Vec<String>> {
		match self {
			Self::Unsupported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_unsupported(&mut self) -> Option<&mut Vec<String>> {
		match self {
			Self::Unsupported(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_unsupported(self) -> Option<Vec<String>> {
		match self {
			Self::Unsupported(v) => Some(v),
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
	
	pub fn as_via(&self) -> Option<&Vec<Via>> {
		match self {
			Self::Via(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_via(&mut self) -> Option<&mut Vec<Via>> {
		match self {
			Self::Via(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_via(self) -> Option<Vec<Via>> {
		match self {
			Self::Via(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_www_authenticate(&self) -> Option<&String> {
		match self {
			Self::WwwAuthenticate(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_www_authenticate(&mut self) -> Option<&mut String> {
		match self {
			Self::WwwAuthenticate(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_www_authenticate(self) -> Option<String> {
		match self {
			Self::WwwAuthenticate(v) => Some(v),
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
			Self::Method(v)                  => Display::fmt(v, f),
			Self::Status(v)                  => Display::fmt(v, f),
			Self::Accept(v)                  => fmt_list(f, v),
			Self::AcceptCredentials(v)       => Display::fmt(v, f),
			Self::AcceptEncoding(v)          => fmt_list(f, v),
			Self::AcceptLanguage(v)          => fmt_list(f, v),
			Self::AcceptRanges(v)            => fmt_list(f, v),
			Self::Allow(v)                   => fmt_list(f, v),
			Self::AuthenticationInfo(v)      => Display::fmt(v, f),
			Self::Authorization(v)           => Display::fmt(v, f),
			Self::Bandwidth(v)               => Display::fmt(v, f),
			Self::BlockSize(v)               => Display::fmt(v, f),
			Self::CacheControl(v)            => Display::fmt(v, f),
			Self::Connection(v)              => fmt_list(f, v),
			Self::ConnectionCredentials(v)   => Display::fmt(v, f),
			Self::ContentBase(v)             => Display::fmt(v, f),
			Self::ContentEncoding(v)         => Display::fmt(v, f),
			Self::ContentLanguage(v)         => Display::fmt(v, f),
			Self::ContentLength(v)           => Display::fmt(v, f),
			Self::ContentLocation(v)         => Display::fmt(v, f),
			Self::ContentType(v)             => Display::fmt(v, f),
			Self::CSeq(v)                    => Display::fmt(v, f),
			Self::Date(v)                    => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::Expires(v)                 => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::From(v)                    => Display::fmt(v, f),
			Self::IfMatch(v)                 => Display::fmt(v, f),
			Self::IfModifiedSince(v)         => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::IfNoneMatch(v)             => Display::fmt(v, f),
			Self::LastModified(v)            => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::Location(v)                => Display::fmt(v, f),
			Self::MediaProperties(v)         => fmt_list(f, v),
			Self::MediaRange(v)              => fmt_list(f, v),
			Self::MTag(v)                    => Display::fmt(v, f),
			Self::NotifyReason(v)            => Display::fmt(v, f),
			Self::PipelineRequests(v)        => Display::fmt(v, f),
			Self::ProxyAuthentication(v)     => Display::fmt(v, f),
			Self::ProxyAuthenticationInfo(v) => Display::fmt(v, f),
			Self::ProxyAuthorization(v)      => Display::fmt(v, f),
			Self::ProxyRequire(v)            => Display::fmt(v, f),
			Self::ProxySupported(v)          => fmt_list(f, v),
			Self::Public(v)                  => fmt_list(f, v),
			Self::Range(v)                   => Display::fmt(v, f),
			Self::Referrer(v)                => Display::fmt(v, f),
			Self::RequestStatus(v)           => Display::fmt(v, f),
			Self::Require(v)                 => Display::fmt(v, f),
			Self::RetryAfter(v)              => Display::fmt(v, f),
			Self::RtpInfo(v)                 => fmt_list(f, v),
			Self::Scale(v)                   => Display::fmt(v, f),
			Self::SeekStyle(v)               => Display::fmt(v, f),
			Self::Server(v)                  => Display::fmt(v, f),
			Self::Session(v)                 => Display::fmt(v, f),
			Self::Speed(v)                   => Display::fmt(v, f),
			Self::Supported(v)               => fmt_list(f, v),
			Self::TerminateReason(v)         => Display::fmt(v, f),
			Self::Timestamp(v)               => Display::fmt(v, f),
			Self::Transport(v)               => fmt_list(f, v),
			Self::Unsupported(v)             => fmt_list(f, v),
			Self::UserAgent(v)               => Display::fmt(v, f),
			Self::Via(v)                     => fmt_list(f, v),
			Self::WwwAuthenticate(v)         => Display::fmt(v, f),
			Self::Custom(_, v)               => f.write_str(v),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Method {
	Options,
	Describe,
	Setup,
	Play,
	PlayNotify,
	Pause,
	TearDown,
	GetParam,
	SetParam,
	Redirect,
	Other(Box<str>)
}

impl FromStr for Method {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"OPTIONS"     => Self::Options,
			"DESCRIBE"    => Self::Describe,
			"SETUP"       => Self::Setup,
			"PLAY"        => Self::Play,
			"PLAY_NOTIFY" => Self::PlayNotify,
			"PAUSE"       => Self::Pause,
			"TEARDOWN"    => Self::TearDown,
			"GET_PARAM"   => Self::GetParam,
			"SET_PARAM"   => Self::SetParam,
			"REDIRECT"    => Self::Redirect,
			v             => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for Method {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Options    => "OPTIONS",
			Self::Describe   => "DESCRIBE",
			Self::Setup      => "SETUP",
			Self::Play       => "PLAY",
			Self::PlayNotify => "PLAY_NOTIFY",
			Self::Pause      => "PAUSE",
			Self::TearDown   => "TEARDOWN",
			Self::GetParam   => "GET_PARAM",
			Self::SetParam   => "SET_PARAM",
			Self::Redirect   => "REDIRECT",
			Self::Other(v)   => v
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
	Continue                           = 100,
	Ok                                 = 200,
	MovedPermanently                   = 301,
	Found                              = 302,
	SeeOther                           = 303,
	NotModified                        = 304,
	UseProxy                           = 305,
	BadRequest                         = 400,
	Unauthorized                       = 401,
	PaymentRequired                    = 402,
	Forbidden                          = 403,
	NotFound                           = 404,
	MethodNotAllowed                   = 405,
	NotAcceptable                      = 406,
	ProxyAuthenticationRequired        = 407,
	RequestTimeout                     = 408,
	Gone                               = 410,
	PreconditionFailed                 = 412,
	RequestBodyTooLarge                = 413,
	RequestUriTooLong                  = 414,
	UnsupportedMediaType               = 415,
	ParameterNotUnderstood             = 451,
	IllegalConferenceIdentifier        = 452,
	NotEnoughBandwidth                 = 453,
	SessionNotFound                    = 454,
	MethodNotValidInThisState          = 455,
	HeaderFieldNotValidForResource     = 456,
	InvalidRange                       = 457,
	ParameterIsReadOnly                = 458,
	AggregateOperationNotAllowed       = 459,
	OnlyAggregateOperationAllowed      = 460,
	UnsupportedTransport               = 461,
	DestinationUnreachable             = 462,
	DestinationProhibited              = 463,
	DataTransportNotReadyYet           = 464,
	NotificationReasonUnknown          = 465,
	KeyManagementError                 = 466,
	ConnectionAuthorizationRequired    = 470,
	ConnectionCredentialsNotAccepted   = 471,
	FailureToEstablishSecureConnection = 472,
	InternalServerError                = 500,
	NotImplemented                     = 501,
	BadGateway                         = 502,
	ServiceUnavailable                 = 503,
	GatewayTimeout                     = 504,
	RtspVersionNotSupported            = 505,
	OptionNotSupported                 = 551,
	ProxyUnavailable                   = 553
}

impl std::error::Error for Status {}

impl TryFrom<usize> for Status {
	type Error = ();
	
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		Ok(match value {
			100 => Self::Continue,
			200 => Self::Ok,
			301 => Self::MovedPermanently,
			302 => Self::Found,
			303 => Self::SeeOther,
			304 => Self::NotModified,
			305 => Self::UseProxy,
			400 => Self::BadRequest,
			401 => Self::Unauthorized,
			402 => Self::PaymentRequired,
			403 => Self::Forbidden,
			404 => Self::NotFound,
			405 => Self::MethodNotAllowed,
			406 => Self::NotAcceptable,
			407 => Self::ProxyAuthenticationRequired,
			408 => Self::RequestTimeout,
			410 => Self::Gone,
			412 => Self::PreconditionFailed,
			413 => Self::RequestBodyTooLarge,
			414 => Self::RequestUriTooLong,
			415 => Self::UnsupportedMediaType,
			451 => Self::ParameterNotUnderstood,
			452 => Self::IllegalConferenceIdentifier,
			453 => Self::NotEnoughBandwidth,
			454 => Self::SessionNotFound,
			455 => Self::MethodNotValidInThisState,
			456 => Self::HeaderFieldNotValidForResource,
			457 => Self::InvalidRange,
			458 => Self::ParameterIsReadOnly,
			459 => Self::AggregateOperationNotAllowed,
			460 => Self::OnlyAggregateOperationAllowed,
			461 => Self::UnsupportedTransport,
			462 => Self::DestinationUnreachable,
			463 => Self::DestinationProhibited,
			464 => Self::DataTransportNotReadyYet,
			465 => Self::NotificationReasonUnknown,
			466 => Self::KeyManagementError,
			470 => Self::ConnectionAuthorizationRequired,
			471 => Self::ConnectionCredentialsNotAccepted,
			472 => Self::FailureToEstablishSecureConnection,
			500 => Self::InternalServerError,
			501 => Self::NotImplemented,
			502 => Self::BadGateway,
			503 => Self::ServiceUnavailable,
			504 => Self::GatewayTimeout,
			505 => Self::RtspVersionNotSupported,
			551 => Self::OptionNotSupported,
			553 => Self::ProxyUnavailable,
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
			Self::Continue                           => "CONTINUE",
			Self::Ok                                 => "OK",
			Self::MovedPermanently                   => "MOVED PERMANENTLY",
			Self::Found                              => "FOUND",
			Self::SeeOther                           => "SEE OTHER",
			Self::NotModified                        => "NOT MODIFIED",
			Self::UseProxy                           => "USE PROXY",
			Self::BadRequest                         => "BAD REQUEST",
			Self::Unauthorized                       => "UNAUTHORIZED",
			Self::PaymentRequired                    => "PAYMENT REQUIRED",
			Self::Forbidden                          => "FORBIDDEN",
			Self::NotFound                           => "NOT FOUND",
			Self::MethodNotAllowed                   => "METHOD NOT ALLOWED",
			Self::NotAcceptable                      => "NOT ACCEPTABLE",
			Self::ProxyAuthenticationRequired        => "PROXY AUTHENTICATION REQUIRED",
			Self::RequestTimeout                     => "REQUEST TIMEOUT",
			Self::Gone                               => "GONE",
			Self::PreconditionFailed                 => "PRECONDITION FAILED",
			Self::RequestBodyTooLarge                => "REQUEST BODY TOO LARGE",
			Self::RequestUriTooLong                  => "REQUEST URI TOO LONG",
			Self::UnsupportedMediaType               => "UNSUPPORTED MEDIA TYPE",
			Self::ParameterNotUnderstood             => "PARAMETER NOT UNDERSTOOD",
			Self::IllegalConferenceIdentifier        => "ILLEGAL CONFERENCE IDENTIFIER",
			Self::NotEnoughBandwidth                 => "NOT ENOUGH BANDWIDTH",
			Self::SessionNotFound                    => "SESSION NOT FOUND",
			Self::MethodNotValidInThisState          => "METHOD NOT VALID IN THIS STATE",
			Self::HeaderFieldNotValidForResource     => "HEADER FIELD NOT VALID FOR RESOURCE",
			Self::InvalidRange                       => "INVALID RANGE",
			Self::ParameterIsReadOnly                => "PARAMETER IS READONLY",
			Self::AggregateOperationNotAllowed       => "AGGREGATE OPERATION NOT ALLOWED",
			Self::OnlyAggregateOperationAllowed      => "ONLY AGGREGATE OPERATION ALLOWED",
			Self::UnsupportedTransport               => "UNSUPPORTED TRANSPORT",
			Self::DestinationUnreachable             => "DESTINATION UNREACHABLE",
			Self::DestinationProhibited              => "DESTINATION PROHIBITED",
			Self::DataTransportNotReadyYet           => "DATA TRANSPORT NOT READY YET",
			Self::NotificationReasonUnknown          => "NOTIFICATION REASON UNKNOWN",
			Self::KeyManagementError                 => "KEY MANAGEMENT ERROR",
			Self::ConnectionAuthorizationRequired    => "CONNECTION AUTHORIZATION REQUIRED",
			Self::ConnectionCredentialsNotAccepted   => "CONNECTION CREDENTIALS NOT ACCEPTED",
			Self::FailureToEstablishSecureConnection => "FAILURE TO ESTABLISH SECURE CONNECTION",
			Self::InternalServerError                => "INTERNAL SERVER ERROR",
			Self::NotImplemented                     => "NOT IMPLEMENTED",
			Self::BadGateway                         => "BAD GATEWAY",
			Self::ServiceUnavailable                 => "SERVICE UNAVAILABLE",
			Self::GatewayTimeout                     => "GATEWAY TIMEOUT",
			Self::RtspVersionNotSupported            => "RTSP VERSION NOT SUPPORTED",
			Self::OptionNotSupported                 => "OPTION NOT SUPPORTED",
			Self::ProxyUnavailable                   => "PROXY UNAVAILABLE",
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcceptCredentials {
	User(Vec<AcceptCredentialsInfo>),
	Proxy,
	Any,
	Ext(String, Option<String>)
}

impl FromStr for AcceptCredentials {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"User"  => Self::User(Vec::new()),
			"Proxy" => Self::Proxy,
			"Any"   => Self::Any,
			v if v.starts_with("User ") => Self::User(v.strip_prefix("User ")
				.unwrap()
				.split(',')
				.map(str::trim)
				.map(AcceptCredentialsInfo::from_str)
				.collect::<Result<Vec<_>, _>>()?),
			v       => {
				let (key, val) = v.split_once('=')
					.map_or((v, None), |(k, v)| (k, Some(v)));
				Self::Ext(key.to_string(), val.map(str::to_string))
			}
		})
	}
}

impl Display for AcceptCredentials {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::User(v) if v.is_empty() => f.write_str("User"),
			Self::Proxy                   => f.write_str("Proxy"),
			Self::Any                     => f.write_str("Any"),
			Self::User(v)                 => {
				f.write_str("User ")?;
				
				Display::fmt(&v[0], f)?;
				
				for info in v {
					f.write_str(",")?;
					Display::fmt(info, f)?;
				}
				
				Ok(())
			}
			Self::Ext(key, None)      => f.write_str(key),
			Self::Ext(key, Some(val)) => write!(f, "{} {}", key, val)
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptCredentialsInfo {
	pub uri:  Url,
	pub algo: AcceptCredentialsCredentialHashAlgorithm,
	pub hash: Box<[u8]>
}

impl FromStr for AcceptCredentialsInfo {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.as_bytes()[0] != b'"' {
			return Err(());
		}
		
		let uri_end = s.find("\";").ok_or(())?;
		let mut values = s[uri_end + 2..].split(';');
		
		Ok(Self {
			uri:  s[1..uri_end + 1].to_string(),
			algo: AcceptCredentialsCredentialHashAlgorithm::from_str(values.next().ok_or(())?)?,
			hash: base64::decode(values.next().ok_or(())?).map_err(|_| ())?.into_boxed_slice()
		})
	}
}

impl Display for AcceptCredentialsInfo {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "\"{}\";{};{}", &self.uri, &self.algo, base64::encode(&self.hash))
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcceptCredentialsCredentialHashAlgorithm {
	Sha256,
	Other(Box<str>)
}

impl FromStr for AcceptCredentialsCredentialHashAlgorithm {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"sha-256" => Self::Sha256,
			v         => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for AcceptCredentialsCredentialHashAlgorithm {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Sha256   => "sha-256",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MediaTypeRanges {
	Npt,
	Smpte,
	Smpte30Drop,
	Smpte25,
	Clock,
	Ext(String)
}

impl FromStr for MediaTypeRanges {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"npt"           => Self::Npt,
			"smpte"         => Self::Smpte,
			"smpte-30-drop" => Self::Smpte30Drop,
			"smpte-25"      => Self::Smpte25,
			"clock"         => Self::Clock,
			v               => Self::Ext(v.to_string())
		})
	}
}

impl Display for MediaTypeRanges {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Npt         => "npt",
			Self::Smpte       => "smpte",
			Self::Smpte30Drop => "smpte-30-drop",
			Self::Smpte25     => "smpte-25",
			Self::Clock       => "clock",
			Self::Ext(v)      => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CacheControl {
	MaxStale(Option<usize>),
	MinFresh(usize),
	OnlyIfCached,
	Public,
	Private,
	NoCache,
	NoTransform,
	MustRevalidate,
	ProxyRevalidate,
	MaxAge(usize),
	Ext(String, Option<String>)
}

impl FromStr for CacheControl {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.split_once('=')
			.map_or((s, None), |(k, v)| (k, Some(v)))
		{
			("max-stale",        v)       => Self::MaxStale(v.map(str::parse).transpose().map_err(|_| ())?),
			("min-fresh",        Some(v)) => Self::MinFresh(v.parse().map_err(|_| ())?),
			("only-if-cached",   None)    => Self::OnlyIfCached,
			("public",           None)    => Self::Public,
			("private",          None)    => Self::Private,
			("no-cache",         None)    => Self::NoCache,
			("no-transform",     None)    => Self::NoTransform,
			("must-revalidate",  None)    => Self::MustRevalidate,
			("proxy-revalidate", None)    => Self::ProxyRevalidate,
			("max-age",          Some(v)) => Self::MaxAge(v.parse().map_err(|_| ())?),
			(key, val)                    => Self::Ext(key.to_string(), val.map(str::to_string)),
		})
	}
}

impl Display for CacheControl {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::MaxStale(None)      => f.write_str("max-stale"),
			Self::MaxStale(Some(v))   => write!(f, "max-stale={}", v),
			Self::MinFresh(v)         => write!(f, "min-fresh={}", v),
			Self::OnlyIfCached        => f.write_str("only-if-cached"),
			Self::Public              => f.write_str("public"),
			Self::Private             => f.write_str("private"),
			Self::NoCache             => f.write_str("no-cache"),
			Self::NoTransform         => f.write_str("no-transform"),
			Self::MustRevalidate      => f.write_str("must-revalidate"),
			Self::ProxyRevalidate     => f.write_str("max-age"),
			Self::MaxAge(v)           => write!(f, "max-age={}", v),
			Self::Ext(key, None)      => f.write_str(key),
			Self::Ext(key, Some(val)) => write!(f, "{}={}", key, val),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Connection {
	Close,
	Ext(String)
}

impl FromStr for Connection {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"close" => Self::Close,
			v       => Self::Ext(v.to_string())
		})
	}
}

impl Display for Connection {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Close  => "close",
			Self::Ext(v) => v
		})
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConnectionCredentials {
	pub url:  Url,
	pub cred: Vec<u8>
}

impl FromStr for ConnectionCredentials {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if !s.starts_with('"') {
			return Err(());
		}
		
		let i = s[1..].find('"').ok_or(())? + 1;
		let url = s[1..i].to_string();
		
		if !s[i + 1..].starts_with(';') {
			return Err(());
		}
		
		let cred = base64::decode(&s[i + 2..]).map_err(|_| ())?;
		Ok(Self { url, cred })
	}
}

impl Display for ConnectionCredentials {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "\"{}\";{}", &self.url, base64::encode(&self.cred))
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaProperty {
	RandomAccess(Option<f32>),
	BeginningOnly,
	NoSeeking,
	Immutable,
	Dynamic,
	TimeProgressing,
	Unlimited,
	TimeLimited(DateTime<Utc>),
	TimeDuration(f32),
	Scales(Vec<(f32, Option<f32>)>),
	Ext(String, Option<String>)
}

impl FromStr for MediaProperty {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.split_once('=')
			.map_or((s, None), |(k, v)| (k, Some(v)))
		{
			("Random-Access",    v)       => Self::RandomAccess(v.map(str::parse).transpose().map_err(|_| ())?),
			("Beginning-Only",   None)    => Self::BeginningOnly,
			("No-Seeking",       None)    => Self::NoSeeking,
			("Immutable",        None)    => Self::Immutable,
			("Dynamic",          None)    => Self::Dynamic,
			("Time-Progressing", None)    => Self::TimeProgressing,
			("Unlimited",        None)    => Self::Unlimited,
			("Time-Limited",     Some(v)) => Self::TimeLimited(v.parse().map_err(|_| ())?),
			("Time-Duration",    Some(v)) => Self::TimeDuration(v.parse().map_err(|_| ())?),
			("Scales",           Some(v)) => {
				if !v.starts_with('"') || !v.ends_with('"') {
					return Err(());
				}
				
				Self::Scales(v[1..v.len() - 1].split(',')
					.map(|v| {
						let v = v.split_once(':')
							.map_or((v, None), |(k, v)| (k, Some(v)));
						
						Ok((v.0.parse().map_err(|_| ())?, v.1.map(str::parse).transpose().map_err(|_| ())?))
					})
					.collect::<Result<_, _>>()?)
			},
			(key, val)                    => Self::Ext(key.to_string(), val.map(str::to_string)),
	   })
	}
}

impl Display for MediaProperty {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::RandomAccess(None)    => f.write_str("Random-Access"),
			Self::RandomAccess(Some(v)) => write!(f, "Random-Access={}", v),
			Self::BeginningOnly         => f.write_str("Beginning-Only"),
			Self::NoSeeking             => f.write_str("No-Seeking"),
			Self::Immutable             => f.write_str("Immutable"),
			Self::Dynamic               => f.write_str("Dynamic"),
			Self::TimeProgressing       => f.write_str("Time-Progressing"),
			Self::Unlimited             => f.write_str("Unlimited"),
			Self::TimeLimited(v)        => write!(f, "Time-Limited={}", v.format("%a, %d %b %Y %T GMT")),
			Self::TimeDuration(v)       => write!(f, "Time-Duration={}", v),
			Self::Scales(v)             => {
				f.write_str("Scales=\"")?;
				
				match v[0] {
					(v0, None)     => write!(f, "{}", v0)?,
					(v0, Some(v1)) => write!(f, "{}:{}", v0, v1)?
				}
				
				for scale in &v[1..] {
					match scale {
						(v0, None)     => write!(f, ",{}", v0)?,
						(v0, Some(v1)) => write!(f, ",{}:{}", v0, v1)?
					}
				}
				
				f.write_str("\"")
			},
			Self::Ext(key, None)        => f.write_str(key),
			Self::Ext(key, Some(val))   => write!(f, "{}={}", key, val),
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MTag {
	pub weak_validator: bool,
	pub value:          String
}

impl FromStr for MTag {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self {
			weak_validator: s.starts_with("W/"),
			value: s[s.find('"').ok_or(())? + 1 .. s.rfind('"').ok_or(())?].to_string()
		})
	}
}

impl Display for MTag {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}\"{}\"", if self.weak_validator { "W/" } else { "" }, self.value)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotifyReason {
	EndOfStream,
	MediaPropertiesUpdate,
	ScaleChange,
	Ext(String)
}

impl FromStr for NotifyReason {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"end-of-stream"           => Self::EndOfStream,
			"media-properties-update" => Self::MediaPropertiesUpdate,
			"scale-change"            => Self::ScaleChange,
			v                         => Self::Ext(v.to_string())
		})
	}
}

impl Display for NotifyReason {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::EndOfStream           => "end-of-stream",
			Self::MediaPropertiesUpdate => "media-properties-update",
			Self::ScaleChange           => "scale-change",
			Self::Ext(v)                => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestStatus {
	pub cseq:   usize,
	pub status: Status,
	pub reason: String
}

impl FromStr for RequestStatus {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = match s.strip_prefix("cseq=") {
			Some(v) => v,
			None => return Err(())
		};
		
		let i = s.find(' ').ok_or(())?;
		let cseq = s[..i].parse().map_err(|_| ())?;
		
		let s = match  s[i + 1..].strip_prefix("status=") {
			Some(v) => v,
			None => return Err(())
		};
		
		let i = s.find(' ').ok_or(())?;
		let status = s[..i].parse::<usize>().map_err(|_| ())
			.and_then(Status::try_from)?;
		
		let s = match  s[i + 1..].strip_prefix("reason=\"") {
			Some(v) => v,
			None => return Err(())
		};
		
		let i = s.find('"').ok_or(())?;
		let reason = s[..1].to_string();
		
		if i != s.len() - 1 {
			return Err(());
		}
		
		Ok(Self { cseq, status, reason })
	}
}

impl Display for RequestStatus {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "cseq={} status={} reason=\"{}\"", self.cseq, self.status, &self.reason)
	}
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
			s.parse().map(Self::Date).map_err(|_| ())
		}
	}
}

impl Display for RetryAfter {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::Date(date)       => write!(f, "{}", date.format("%a, %d %b %Y %T GMT")),
			Self::Seconds(seconds) => write!(f, "{}", seconds)
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RtpInfo {
	url: Url,
	ssrc: u32,
	params: Vec<String>
}

impl FromStr for RtpInfo {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = match s.strip_prefix("url=\"") {
			Some(v) => v,
			None => return Err(())
		};
		
		let i = s.find('"').ok_or(())?;
		let url = s[..i].to_string();
		
		let s = match s[i..].strip_prefix("\" ssrc=") {
			Some(v) => v,
			None => return Err(())
		};
		
		let i      = s.find(':').ok_or(())?;
		let ssrc   = s[..i].parse::<u32>().map_err(|_| ())?;
		let params = s[i + 1..].split(';')
			.map(str::to_string)
			.collect::<Vec<_>>();
		
		Ok(Self { url, ssrc, params })
	}
}

impl Display for RtpInfo {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "url=\"{}\" ssrc={:X}:", &self.url, self.ssrc)?;
		write!(f, "{}", &self.params[0])?;
		
		for param in &self.params[1..] {
			write!(f, ";{}", param)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeekStyle {
	Rap,
	CoRap,
	FirstPrior,
	Next,
	Ext(String)
}

impl FromStr for SeekStyle {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"RAP"         => Self::Rap,
			"CoRAP"       => Self::CoRap,
			"First-Prior" => Self::FirstPrior,
			"Next"        => Self::Next,
			v             => Self::Ext(v.to_string())
		})
	}
}

impl Display for SeekStyle {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Rap        => "RAP",
			Self::CoRap      => "CoRAP",
			Self::FirstPrior => "First-Prior",
			Self::Next       => "Next",
			Self::Ext(v)     => v
		})
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Speed {
	pub lower: f64,
	pub upper: f64
}

impl FromStr for Speed {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (lower, upper) = s.split_once('-').ok_or(())?;
		Ok(Self {
			lower: lower.parse().map_err(|_| ())?,
			upper: upper.parse().map_err(|_| ())?
		})
	}
}

impl Display for Speed {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}-{}", self.lower, self.upper)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminateReason {
	pub reason: TerminateReasonReason,
	pub params: Vec<TerminateReasonParam>
}

impl FromStr for TerminateReason {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut s = s.split(';');
		
		Ok(Self {
			reason: s.next().ok_or(())?.parse()?,
			params: s.map(str::parse)
				.collect::<Result<Vec<_>, _>>()?
		})
	}
}

impl Display for TerminateReason {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", &self.reason)?;
		
		for param in &self.params {
			write!(f, ";{}", param)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminateReasonReason {
	ServerAdmin,
	SessionTimeout,
	InternalError,
	Ext(String)
}

impl FromStr for TerminateReasonReason {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Session-Timeout" => Self::SessionTimeout,
			"Server-Admin"    => Self::ServerAdmin,
			"Internal-Error"  => Self::InternalError,
			v                 => Self::Ext(v.to_string())
		})
	}
}

impl Display for TerminateReasonReason {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::SessionTimeout => "Session-Timeout",
			Self::ServerAdmin    => "Server-Admin",
			Self::InternalError  => "Internal-Error",
			Self::Ext(v)         => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminateReasonParam {
	Time(DateTime<Utc>),
	UserMsg(String),
	Ext(String, Option<String>)
}

impl FromStr for TerminateReasonParam {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.split_once('=')
			.map_or((s, None), |(k, v)| (k, Some(v)))
		{
			("time",     Some(v)) => Self::Time(v.parse().map_err(|_| ())?),
			("user-msg", Some(v)) => Self::UserMsg(v.to_string()),
			(key,        val)     => Self::Ext(key.to_string(), val.map(str::to_string))
		})
	}
}

impl Display for TerminateReasonParam {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			TerminateReasonParam::Time(v)             => write!(f, "time={}", v.format("%Y%m%dT%H%M%SZ")),
			TerminateReasonParam::UserMsg(v)          => write!(f, "user-msg={}", v),
			TerminateReasonParam::Ext(key, None)      => f.write_str(key),
			TerminateReasonParam::Ext(key, Some(val)) => write!(f, "{}={}", key, val)
		}
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Timestamp {
	pub value: f64,
	pub delay: Option<f64>
}

impl FromStr for Timestamp {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (value, delay) = s.split_once(' ')
			.map_or((s, None), |(k, v)| (k, Some(v)));
		
		Ok(Self {
			value: value.parse().map_err(|_| ())?,
			delay: delay.map(str::parse).transpose().map_err(|_| ())?
		})
	}
}

impl Display for Timestamp {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}", self.value)?;
		
		if let Some(delay) = self.delay {
			write!(f, " {}", delay)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Transport {
	pub id:     String,
	pub params: Vec<TransportParam>
}

impl FromStr for Transport {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut s = s.split(';');
		
		Ok(Self {
			id:     s.next().ok_or(())?.to_string(),
			params: s.map(str::parse)
				.collect::<Result<Vec<_>, _>>()?
		})
	}
}

impl Display for Transport {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.id)?;
		
		for param in &self.params {
			write!(f, ";{}", param)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportParam {
	Unicast,
	Multicast,
	Interleaved(u8, Option<u8>),
	Ttl(u8),
	Layers(usize),
	Ssrc(u32),
	Mode(Vec<Mode>),
	DstAddr(Vec<String>),
	SrcAddr(Vec<String>),
	Setup(ConTransSetup),
	Connection(ConTransCon),
	RtcpMux,
	MiKey(Vec<u8>),
	Ext(String, Option<String>)
}

impl FromStr for TransportParam {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		fn parse_addr_list(v: &str) -> Result<Vec<String>, ()> {
			v.split('/')
				.map(|s| if !s.starts_with('"') && !s.ends_with('"') {
					Err(())
				} else {
					Ok(s.to_string())
				})
				.collect()
		}
		
		Ok(match s.split_once('=')
			.map_or((s, None), |(k, v)| (k, Some(v)))
		{
			("unicast", None)        => Self::Unicast,
			("multicast", None)      => Self::Multicast,
			("interleaved", Some(v)) => {
				let (v0, v1) = v.split_once('-')
					.map_or((s, None), |(k, v)| (k, Some(v)));
				
				Self::Interleaved(
					v0.parse().map_err(|_| ())?,
					v1.map(str::parse).transpose().map_err(|_| ())?
				)
			}
			("ttl", Some(v)) => Self::Ttl(v.parse().map_err(|_| ())?),
			("layers", Some(v)) => Self::Layers(v.parse().map_err(|_| ())?),
			("ssrc", Some(v)) => Self::Layers(v.parse().map_err(|_| ())?),
			("mode", Some(v)) => {
				if !v.starts_with('"') || !v.ends_with('"') {
					return Err(());
				}
				
				Self::Mode(v[1..v.len() - 1].split(',')
					.map(str::parse)
					.collect::<Result<Vec<_>, _>>()?)
			}
			("dest_addr", Some(v))  => Self::DstAddr(parse_addr_list(v)?),
			("src_addr", Some(v))   => Self::SrcAddr(parse_addr_list(v)?),
			("setup",      Some(v)) => Self::Setup(v.parse().map_err(|_| ())?),
			("connection", Some(v)) => Self::Connection(v.parse().map_err(|_| ())?),
			("RTCP-mux",   None)    => Self::RtcpMux,
			("MIKEY",      Some(v)) => Self::MiKey(base64::decode(v).map_err(|_| ())?),
			(key, val) => Self::Ext(key.to_string(), val.map(str::to_string))
		})
	}
}

impl Display for TransportParam {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		fn fmt_addr_list(f: &mut Formatter, v: &[String]) -> FmtResult {
			write!(f, "\"{}\"", &v[0])?;
			
			for e in &v[1..] {
				write!(f, "/\"{}\"", e)?;
			}
			
			Ok(())
		}
		
		match self {
			TransportParam::Unicast                  => f.write_str("unicast"),
			TransportParam::Multicast                => f.write_str("multicast"),
			TransportParam::Interleaved(v, None)     => write!(f, "interleaved={}", v),
			TransportParam::Interleaved(v, Some(v1)) => write!(f, "interleaved={}-{}", v, v1),
			TransportParam::Ttl(v)                   => write!(f, "ttl={}", v),
			TransportParam::Layers(v)                => write!(f, "layers={}", v),
			TransportParam::Ssrc(v)                  => write!(f, "ssrc={}", v),
			TransportParam::Mode(v)                  =>  {
				write!(f, "\"{}", v[0])?;
				
				for mode in &v[1..] {
					write!(f, ",{}", mode)?;
				}
				
				f.write_str("\"")
			}
			TransportParam::DstAddr(v)               => fmt_addr_list(f, v),
			TransportParam::SrcAddr(v)               => fmt_addr_list(f, v),
			TransportParam::Setup(v)                 => write!(f, "setup={}", v),
			TransportParam::Connection(v)            => write!(f, "connection={}", v),
			TransportParam::RtcpMux                  => f.write_str("RTCP-mux"),
			TransportParam::MiKey(v)                 => write!(f, "MIKEY={}", base64::encode(v)),
			TransportParam::Ext(key, None)           => f.write_str(key),
			TransportParam::Ext(key, Some(val))      => write!(f, "{}={}", key, val)
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConTransSetup {
	Active,
	Passive,
	ActPass
}

impl FromStr for ConTransSetup {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"active"  => Self::Active,
			"passive" => Self::Passive,
			"actpass" => Self::ActPass,
			_         => return Err(())
		})
	}
}

impl Display for ConTransSetup {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Active  => "active",
			Self::Passive => "passive",
			Self::ActPass => "actpass"
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConTransCon {
	New,
	Existing
}

impl FromStr for ConTransCon {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"new"      => Self::New,
			"existing" => Self::Existing,
			_          => return Err(())
		})
	}
}

impl Display for ConTransCon {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::New      => "new",
			Self::Existing => "existing"
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mode {
	Play,
	Ext(String)
}

impl FromStr for Mode {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"PLAY" => Self::Play,
			v      => Self::Ext(v.to_string())
		})
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::Play   => "PLAY",
			Self::Ext(v) => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Via {
	pub protocol_name:      String,
	pub protocol_version:   String,
	pub protocol_transport: TransportProtocol,
	pub sent_by:            String,
	pub params:             Vec<ViaParam>
}

impl FromStr for Via {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (sent_protocol, s) = s.split_once(' ').ok_or(())?;
		let (protocol_name, sent_protocol) = sent_protocol.split_once('/').ok_or(())?;
		let (protocol_version, protocol_transport) = sent_protocol.split_once('/').ok_or(())?;
		let mut params = s.split(';');
		
		Ok(Self {
			protocol_name:      protocol_name.to_string(),
			protocol_version:   protocol_version.to_string(),
			protocol_transport: protocol_transport.parse()?,
			sent_by:            params.next().ok_or(())?.to_string(),
			params:             params.map(str::parse)
				.collect::<Result<Vec<_>, _>>()?
		})
	}
}

impl Display for Via {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}/{}/{} {}", &self.protocol_name, &self.protocol_version, &self.protocol_transport, &self.sent_by)?;
		
		for param in &self.params {
			write!(f, ";{}", param)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportProtocol {
	Udp,
	Tcp,
	Tls,
	Quic,
	Other(Box<str>)
}

impl FromStr for TransportProtocol {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"UDP"  => Self::Udp,
			"TCP"  => Self::Tcp,
			"TLS"  => Self::Tls,
			"QUIC" => Self::Quic,
			v      => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl Display for TransportProtocol {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Udp      => "UDP",
			Self::Tcp      => "TCP",
			Self::Tls      => "TLS",
			Self::Quic     => "QUIC",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViaParam {
	Ttl(u8),
	MAddr(String),
	Received(IpAddr),
	Ext(String, Option<String>)
}

impl FromStr for ViaParam {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.split_once(' ')
			.map_or((s, None), |(k, v)| (k, Some(v)))
		{
			("ttl",      Some(val)) => Self::Ttl(val.parse().map_err(|_| ())?),
			("maddr",    Some(val)) => Self::MAddr(val.to_string()),
			("received", Some(val)) => Self::Received(val.parse().map_err(|_| ())?),
			(key, val) => Self::Ext(key.to_string(), val.map(str::to_string))
		})
	}
}

impl Display for ViaParam {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			ViaParam::Ttl(v)              => write!(f, "ttl={}", v),
			ViaParam::MAddr(v)            => write!(f, "maddr={}", v),
			ViaParam::Received(v)         => write!(f, "received={}", v),
			ViaParam::Ext(key, None)      => f.write_str(key),
			ViaParam::Ext(key, Some(val)) => write!(f, "{}={}", key, val)
		}
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MediaTime(String);

impl FromStr for MediaTime {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.to_string()))
	}
}

impl Display for MediaTime {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(&self.0)
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
	
	pub async fn send_async(&mut self, stream: &mut (impl traits::AsyncStream + Unpin + ?Sized)) -> std::io::Result<()> where H: AsRef<[Header]>, D: AsRef<[u8]> {
		use {futures_lite::io::AsyncWriteExt, traits::AsyncStreamExt};
		stream.write_headers(self.0.as_ref()).await?;
		stream.write_all(self.1.as_ref()).await?;
		Ok(())
	}
	
	// TODO
}