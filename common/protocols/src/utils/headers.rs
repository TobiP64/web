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

use std::{str::FromStr, fmt::{self, Write}};

pub const DATE_FORMAT: &str = "%a, %d %b %Y %T GMT";

pub fn parse_date(v: &str) -> Result<chrono::DateTime<chrono::Utc>, ()> {
	Ok(chrono::DateTime::from_utc(chrono::NaiveDateTime::parse_from_str(v, DATE_FORMAT)
		.map_err(|_| ())?, chrono::Utc))
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Encoding {
	GZip,
	Deflate,
	Compress,
	Identity,
	Br,
	Other(Box<str>)
}

impl FromStr for Encoding {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"gzip"     => Self::GZip,
			"deflate"  => Self::Deflate,
			"compress" => Self::Compress,
			"identity" => Self::Identity,
			"br"       => Self::Br,
			v          => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl fmt::Display for Encoding {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Self::GZip     => "gzip",
			Self::Deflate  => "deflate",
			Self::Compress => "compress",
			Self::Identity => "identity",
			Self::Br       => "br",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Language {
	En,
	EnUS,
	EnGB,
	EnAU,
	De,
	DeDE,
	DeAT,
	DeCH,
	Other(Box<str>)
}

impl FromStr for Language {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.split(';').next().ok_or(())? {
			"en"    => Self::En,
			"en-US" => Self::EnUS,
			"en-GB" => Self::EnGB,
			"en-AU" => Self::EnAU,
			"de"    => Self::De,
			"de-DE" => Self::DeDE,
			"de-AT" => Self::DeAT,
			"de-CH" => Self::DeCH,
			v       => Self::Other(v.to_string().into_boxed_str())
		})
	}
}

impl fmt::Display for Language {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Self::En       => "en",
			Self::EnUS     => "en-US",
			Self::EnGB     => "en-GB",
			Self::EnAU     => "en-AU",
			Self::De       => "de",
			Self::DeDE     => "de-DE",
			Self::DeAT     => "de-AT",
			Self::DeCH     => "de-CH",
			Self::Other(v) => v
		})
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaType {
	pub r#type:  MainType,
	pub subtype: String,
	pub suffix:  Option<String>,
	pub params:  Option<Vec<String>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainType {
	Application,
	Audio,
	Font,
	Example,
	Image,
	Message,
	Model,
	Multipart,
	Text,
	Video
}

impl FromStr for MediaType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self {
			r#type: match &s[..s.find('/').unwrap_or(s.len())] {
				"application" => MainType::Application,
				"audio"       => MainType::Audio,
				"font"        => MainType::Font,
				"example"     => MainType::Example,
				"image"       => MainType::Image,
				"message"     => MainType::Message,
				"model"       => MainType::Model,
				"multipart"   => MainType::Multipart,
				"text"        => MainType::Text,
				"video"       => MainType::Video,
				_ => return Err(())
			},
			subtype: s[s.find('/').ok_or(())? + 1..
				s.find(|ch| ch == '+' || ch == ';')
					.unwrap_or(s.len())]
				.to_string(),
			suffix: s.find('+').map(|off| s[off + 1..
				s.find(';').unwrap_or(s.len())].to_string()),
			params: s.find(';').map(|off| s[off + 1..]
				.split(';')
				.map(String::from)
				.collect())
		})
	}
}

impl fmt::Display for MediaType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self.r#type {
			MainType::Application => "application/",
			MainType::Audio       => "audio/",
			MainType::Font        => "font/",
			MainType::Example     => "example/",
			MainType::Image       => "image/",
			MainType::Message     => "message/",
			MainType::Model       => "model/",
			MainType::Multipart   => "multipart/",
			MainType::Text        => "text/",
			MainType::Video       => "video/",
		})?;

		f.write_str(&self.subtype)?;

		if let Some(suffix) = &self.suffix {
			f.write_char('+')?;
			f.write_str(suffix)?;
		}

		if let Some(params) = &self.params {
			for param in params {
				f.write_char(';')?;
				f.write_str(param)?;
			}
		}

		Ok(())
	}
}

pub fn parse_list<T: FromStr>(value: &str) -> Result<Vec<T>, ()> where <T as FromStr>::Err: fmt::Debug {
	Ok(value.split(',')
		.filter_map(|v| v.trim().parse().ok())
		.collect())
}

pub fn parse_map(value: &str) -> Vec<(String, String)> {
	value.split(';')
		.map(str::trim)
		.map(|s| match s.split_once('=') {
			None             => (s.to_string(), String::new()),
			Some((key ,val)) => (key.to_string(), val.to_string())
		}).collect()
}

pub fn fmt_list<I: IntoIterator>(f: &mut fmt::Formatter, iter: I) -> fmt::Result where I::Item: fmt::Display {
	for e in iter {
		write!(f, "{}, ", e)?;
	}
	Ok(())
}

pub fn fmt_map<I: IntoIterator<Item = (K, V)>, K: fmt::Display, V: fmt::Display>(f: &mut fmt::Formatter, iter: I) -> fmt::Result {
	for (k, v) in iter {
		let v = v.to_string();

		if v.is_empty() {
			write!(f, "{};", k)?;
		} else {
			write!(f, "{}={};", k, v)?;
		}
	}
	Ok(())
}