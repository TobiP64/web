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

pub const DEFAULT_PROBABILITY: f32 = 1.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Sampler {
	AlwaysOn,
	AlwaysOff,
	TraceIdRatio { probability: f32 },
	ParentBasedAlwaysOn,
	ParentBasedAlwaysOff,
	ParentBasedTraceIdRatio { probability: f32 },
}

impl Sampler {
	pub fn from_env() -> Option<Self> {
		Some(match &*std::env::var("OTEL_TRACES_SAMPLER").ok()? {
			"always_on" => Self::AlwaysOn,
			"always_off" => Self::AlwaysOff,
			"traceidration" => Self::TraceIdRatio {
				probability: std::env::var("OTEL_TRACES_SAMPLER_ARG")
					.ok()
					.and_then(|v| v.parse().ok())
					.unwrap_or(DEFAULT_PROBABILITY)
			},
			"parentbased_always_on" => Self::ParentBasedAlwaysOn,
			"parentbased_always_off" => Self::ParentBasedAlwaysOff,
			"parentbased_traceidration" => Self::ParentBasedTraceIdRatio {
				probability: std::env::var("OTEL_TRACES_SAMPLER_ARG")
					.ok()
					.and_then(|v| v.parse().ok())
					.unwrap_or(DEFAULT_PROBABILITY)
			},
			_ => return None,
		})
	}
}

#[derive(Debug)]
pub struct Span {
	trace_id:       u128,
	span_id:        u128,
	span_kind:      SpanKind,
	parent:         Option<Arc<Self>>,
	name:           Cow<'static, str>,
	start:          u128,
	end:            u128,
	attributes:     Vec<(Cow<'static, str>, AnyValue)>,
	links:          Vec<Link>,
	events:         Vec<Event>,
	status_code:    Option<StatusCode>,
	status_message: Option<Cow<'static, str>>,
	runtime:        Runtime
}

impl Span {
	pub fn is_recording(&self) -> bool {
		unimplemented!()
	}
	
	pub fn set_attribute(&self, key: Cow<'static, str>, val: AnyValue) {
		unimplemented!()
	}
	
	pub fn set_attributes(&self, key: Cow<'static, str>, val: AnyValue) {
		unimplemented!()
	}
	
	pub fn add_event(&self, name: Cow<'static, str>, attributes: impl IntoIterator<Item = (Cow<'static, str>, AnyValue)>, timestamp: Option<u128>) {
		unimplemented!()
	}
	
	pub fn set_status(&self) {
		unimplemented!()
	}
	
	pub fn update_name(&self, name: Cow<'static, str>) {
		unimplemented!()
	}
	
	pub fn end(self) {
		self.runtime.0.clone()
			.sync
			.lock().unwrap()
			.spans
			.push(self);
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default)]
pub struct Link {
	
}

#[derive(Clone, Debug, Default)]
pub struct Event {
	
}