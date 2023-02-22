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

//! https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/trace/v1/trace.proto
//! https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/collector/logs/v1/logs_service.proto

use super::*;

pub const HTTP_PATH: &str = "/v1/traces";

pub type ExportTraceServiceRequest  = TracesData;
pub type ExportTraceServiceResponse = ();

pub trait TraceService {
	fn export(&self, _: ExportTraceServiceRequest) -> std::io::Result<ExportTraceServiceResponse>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TracesData {
	pub resource_spans: Vec<ResourceSpans>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceSpans {
	pub resource:                      resource::Resource,
	pub instrumentation_library_spans: Vec<InstrumentationLibrarySpans>,
	pub schema_url:                    String
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct InstrumentationLibrarySpans {
	pub instrumentation_library: InstrumentationLibrary,
	pub spans:                   Vec<Span>,
	pub schema_url:              String
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Span {
	pub trace_id:                 TraceId,
	pub span_id:                  SpanId,
	pub trace_state:              String,
	pub parent_span_id:           SpanId,
	pub name:                     String,
	pub kind:                     SpanKind,
	pub start_time_unix_nano:     u64,
	pub end_time_unix_nano:       u64,
	pub attributes:               Vec<KeyValue>,
	pub dropped_attributes_count: u32,
	pub events:                   Vec<Event>,
	pub dropped_events_count:     u32,
	pub links:                    Vec<Link>,
	pub dropped_links_count:      u32,
	pub status:                   Status
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SpanKind {
	Unspecified,
	Internal,
	Server,
	Client,
	Producer,
	Consumer
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Event {
	pub time_unix_nano:           u64,
	pub name:                     String,
	pub attributes:               Vec<KeyValue>,
	pub dropped_attributes_count: u32
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Link {
	pub trace_id:                 TraceId,
	pub span_id:                  SpanId,
	pub trace_state:              String,
	pub attributes:               Vec<KeyValue>,
	pub dropped_attributes_count: u32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Status {
	pub message: String,
	pub code:    StatusCode
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StatusCode {
	Unset,
	Ok,
	Error
}