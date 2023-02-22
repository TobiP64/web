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
	std::{str::FromStr, fmt::{Display, Formatter, Result as FmtResult}},
	crate::utils::*
};

static NAMES: [&str; 22] = [
	"Bcc",
	"Cc",
	"Comments",
	"Keywords",
	"Date",
	"From",
	"In-Reply-To",
	"Message-Id",
	"Received",
	"References",
	"Reply-To",
	"Resent-Bcc",
	"Resent-Cc",
	"Resent-Date",
	"Resent-From",
	"Resent-Message-Id",
	"Resent-Sender",
	"Resent-To",
	"Return-Path",
	"Sender",
	"Subject",
	"To"
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HeaderId {
	Bcc,
	Cc,
	Comments,
	Keywords,
	Date,
	From,
	InReplyTo,
	MessageId,
	Received,
	References,
	ReplyTo,
	ResentBcc,
	ResentCc,
	ResentDate,
	ResentFrom,
	ResentMessageId,
	ResentSender,
	ResentTo,
	ReturnPath,
	Sender,
	Subject,
	To
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
			"Bcc"               => Self::Bcc,
			"Cc"                => Self::Cc,
			"Comments"          => Self::Comments,
			"Keywords"          => Self::Keywords,
			"Date"              => Self::Date,
			"From"              => Self::From,
			"In-Reply-To"       => Self::InReplyTo,
			"Message-Id"        => Self::MessageId,
			"Received"          => Self::Received,
			"References"        => Self::References,
			"Reply-To"          => Self::ReplyTo,
			"Resent-Bcc"        => Self::ResentBcc,
			"Resent-Cc"         => Self::ResentCc,
			"Resent-Date"       => Self::ResentDate,
			"Resent-From"       => Self::ResentFrom,
			"Resent-Message-Id" => Self::ResentMessageId,
			"Resent-Sender"     => Self::ResentSender,
			"Resent-To"         => Self::ResentTo,
			"Return-Path"       => Self::ReturnPath,
			"Sender"            => Self::Sender,
			"Subject"           => Self::Subject,
			"To"                => Self::To,
			_                   => return Err(())
		})
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Header {
	Bcc(Vec<String>),
	Cc(Vec<String>),
	Comments(String),
	Keywords(Vec<String>),
	Date(chrono::DateTime<chrono::Utc>),
	From(Vec<String>),
	InReplyTo(String),
	MessageId(String),
	Received(Received),
	References(String),
	ReplyTo(Vec<String>),
	ResentBcc(Vec<String>),
	ResentCc(Vec<String>),
	ResentDate(chrono::DateTime<chrono::Utc>),
	ResentFrom(Vec<String>),
	ResentMessageId(String),
	ResentSender(String),
	ResentTo(Vec<String>),
	ReturnPath(String),
	Sender(String),
	Subject(String),
	To(Vec<String>),
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
			HeaderId::Bcc             => Self::Bcc(parse_list(v)?),
			HeaderId::Cc              => Self::Cc(parse_list(v)?),
			HeaderId::Comments        => Self::Comments(v.to_string()),
			HeaderId::Keywords        => Self::Keywords(parse_list(v)?),
			HeaderId::Date            => Self::Date(v.parse().map_err(|_| ())?),
			HeaderId::From            => Self::From(parse_list(v)?),
			HeaderId::InReplyTo       => Self::InReplyTo(v.to_string()),
			HeaderId::MessageId       => Self::MessageId(v.to_string()),
			HeaderId::Received        => Self::Received(v.parse()?),
			HeaderId::References      => Self::References(v.to_string()),
			HeaderId::ReplyTo         => Self::ReplyTo(parse_list(v)?),
			HeaderId::ResentBcc       => Self::ResentBcc(parse_list(v)?),
			HeaderId::ResentCc        => Self::ResentCc(parse_list(v)?),
			HeaderId::ResentDate      => Self::ResentDate(v.parse().map_err(|_| ())?),
			HeaderId::ResentFrom      => Self::ResentFrom(parse_list(v)?),
			HeaderId::ResentMessageId => Self::ResentMessageId(v.to_string()),
			HeaderId::ResentSender    => Self::ResentSender(v.to_string()),
			HeaderId::ResentTo        => Self::ResentTo(parse_list(v)?),
			HeaderId::ReturnPath      => Self::ReturnPath(v.to_string()),
			HeaderId::Sender          => Self::Sender(v.to_string()),
			HeaderId::Subject         => Self::Subject(v.to_string()),
			HeaderId::To              => Self::To(parse_list(v)?)
		})
	}
	
	pub fn id(&self) -> Result<HeaderId, &str> {
		Ok(match self {
			
			Self::Bcc(_)             => HeaderId::Bcc,
			Self::Cc(_)              => HeaderId::Cc,
			Self::Comments(_)        => HeaderId::Comments,
			Self::Keywords(_)        => HeaderId::Keywords,
			Self::Date(_)            => HeaderId::Date,
			Self::From(_)            => HeaderId::From,
			Self::InReplyTo(_)       => HeaderId::InReplyTo,
			Self::MessageId(_)       => HeaderId::MessageId,
			Self::Received(_)        => HeaderId::Received,
			Self::References(_)      => HeaderId::References,
			Self::ReplyTo(_)         => HeaderId::ReplyTo,
			Self::ResentBcc(_)       => HeaderId::ResentBcc,
			Self::ResentCc(_)        => HeaderId::ResentCc,
			Self::ResentDate(_)      => HeaderId::ResentDate,
			Self::ResentFrom(_)      => HeaderId::ResentFrom,
			Self::ResentMessageId(_) => HeaderId::ResentMessageId,
			Self::ResentSender(_)    => HeaderId::ResentSender,
			Self::ResentTo(_)        => HeaderId::ResentTo,
			Self::ReturnPath(_)      => HeaderId::ReturnPath,
			Self::Sender(_)          => HeaderId::Sender,
			Self::Subject(_)         => HeaderId::Subject,
			Self::To(_)              => HeaderId::To,
			Self::Custom(name, _)    => return Err(name),
		})
	}
	
	pub fn name(&self) -> &str {
		crate::utils::unstable::_82223_into_ok_or_err(self.id().map(HeaderId::name))
	}
}

impl Display for Header {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::Bcc(v)             => fmt_list(f, v),
			Self::Cc(v)              => fmt_list(f, v),
			Self::Comments(v)        => Display::fmt(v, f),
			Self::Keywords(v)        => fmt_list(f, v),
			Self::Date(v)            => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::From(v)            => fmt_list(f, v),
			Self::InReplyTo(v)       => Display::fmt(v, f),
			Self::MessageId(v)       => Display::fmt(v, f),
			Self::Received(v)        => Display::fmt(v, f),
			Self::References(v)      => Display::fmt(v, f),
			Self::ReplyTo(v)         => fmt_list(f, v),
			Self::ResentBcc(v)       => fmt_list(f, v),
			Self::ResentCc(v)        => fmt_list(f, v),
			Self::ResentDate(v)      => write!(f, "{}", v.format("%a, %d %b %Y %T GMT")),
			Self::ResentFrom(v)      => fmt_list(f, v),
			Self::ResentMessageId(v) => Display::fmt(v, f),
			Self::ResentSender(v)    => Display::fmt(v, f),
			Self::ResentTo(v)        => fmt_list(f, v),
			Self::ReturnPath(v)      => Display::fmt(v, f),
			Self::Sender(v)          => Display::fmt(v, f),
			Self::Subject(v)         => Display::fmt(v, f),
			Self::To(v)              => fmt_list(f, v),
			Self::Custom(_, v)       => Display::fmt(v, f),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Received {
	pub token: String,
	pub date:  chrono::DateTime<chrono::Utc>
}

impl FromStr for Received {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

impl Display for Received {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		todo!()
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
	
	/*pub fn send(&mut self, stream: &mut (impl traits::Stream + ?Sized)) -> std::io::Result<()> where H: AsRef<[Header]>, D: AsRef<[u8]> {
		stream.write_headers(self.0.as_ref())?;
		stream.write_all(self.1.as_ref())?;
		Ok(())
	}
	
	pub async fn send_async(&mut self, stream: &mut (impl traits::AsyncStream + Unpin + ?Sized)) -> std::io::Result<()> where H: AsRef<[Header]>, D: AsRef<[u8]> {
		use {futures_lite::io::AsyncWriteExt, traits::AsyncStreamExt};
		stream.write_headers(self.0.as_ref()).await?;
		stream.write_all(self.1.as_ref()).await?;
		Ok(())
	}*/
	
	// TODO
}