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

use super::*;

#[derive(Clone, Debug, Default)]
pub struct LogParams<'a> {
	pub severity_number:          Option<SeverityNumber>,
	pub severity_text:            Option<Cow<'a, str>>,
	pub name:                     Option<Cow<'a, str>>,
	pub body:                     Option<AnyValue>,
	pub attributes:               Option<Vec<KeyValue>>,
	pub dropped_attributes_count: Option<u32>,
	pub flags:                    Option<u32>,
	pub span:                     Option<&'a Span>
}

impl<'a> LogParams<'a> {
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn severity_number(mut self, v: SeverityNumber) -> Self {
		self.severity_number = Some(v);
		self
	}
	
	pub fn severity_text(mut self, v: Cow<'a, str>) -> Self {
		self.severity_text = Some(v);
		self
	}
	
	pub fn name(mut self, v: Cow<'a, str>) -> Self {
		self.name = Some(v);
		self
	}
	
	pub fn body(mut self, v: AnyValue) -> Self {
		self.body = Some(v);
		self
	}
	
	pub fn attributes(mut self, v: Vec<KeyValue>) -> Self {
		self.attributes = Some(v);
		self
	}
	
	pub fn attribute(mut self, v: KeyValue) -> Self {
		self.attributes.get_or_insert_with(Default::default).push(v);
		self
	}
	
	pub fn dropped_attributes_count(mut self, v: u32) -> Self {
		self.dropped_attributes_count = Some(v);
		self
	}
	
	pub fn flags(mut self, v: u32) -> Self {
		self.flags = Some(v);
		self
	}
	
	pub fn span(mut self, v: &'a Span) -> Self {
		self.span = Some(v);
		self
	}
}

pub struct Logger<T: log::Log> {
	inner: T,
	rt:    Runtime
}

impl<T: log::Log> Logger<T> {
	pub fn new(runtime: &Runtime, inner: T) -> Self {
		Self { inner, rt: runtime.clone() }
	}
}

impl<T: log::Log> log::Log for Logger<T> {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		self.inner.enabled(metadata)
	}
	
	fn log(&self, record: &log::Record) {
		self.inner.log(record);
		self.rt.log(LogParams::new()
			.severity_number(match record.level() {
				log::Level::Error => SeverityNumber::Error,
				log::Level::Warn  => SeverityNumber::Warn,
				log::Level::Info  => SeverityNumber::Info,
				log::Level::Debug => SeverityNumber::Debug,
				log::Level::Trace => SeverityNumber::Trace,
			})
			.body(AnyValue::String(record.args().to_string()))
			.attributes([
				record.file().map(|v| KeyValue {
					key:   "file".to_string(),
					value: AnyValue::String(v.to_string())
				}),
				record.module_path().map(|v| KeyValue {
					key:   "module_path".to_string(),
					value: AnyValue::String(v.to_string())
				}),
				Some(KeyValue {
					key:   "target".to_string(),
					value: AnyValue::String(record.target().to_string())
				}),
				record.line().map(|v| KeyValue {
					key:   "line".to_string(),
					value: AnyValue::Int(v as _)
				})
			].into_iter()
				.flatten()
				.collect()));
	}
	
	fn flush(&self) {
		self.inner.flush();
	}
}