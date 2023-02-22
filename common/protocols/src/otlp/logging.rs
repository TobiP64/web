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

//! https://github.com/open-telemetry/opentelemetry-proto/tree/main/opentelemetry/proto/logs/v1
//! https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/collector/logs/v1/logs_service.proto

use super::*;

pub const HTTP_PATH: &str = "/v1/logs";

pub type ExportLogsServiceRequest  = LogsData;
pub type ExportLogsServiceResponse = ();

pub trait LogsService {
	fn export(&self, _: ExportLogsServiceRequest) -> std::io::Result<ExportLogsServiceResponse>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LogsData {
	pub resource_logs: Vec<ResourceLogs>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceLogs {
	pub resource:                     resource::Resource,
	pub instrumentation_library_logs: Vec<InstrumentationLibraryLogs>,
	pub schema_url:                   String
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct InstrumentationLibraryLogs {
	pub instrumentation_library: InstrumentationLibrary,
	pub logs:                    Vec<LogRecord>,
	pub schema_url:              String
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SeverityNumber {
	Unspecified,
	Trace,
	Trace2,
	Trace3,
	Trace4,
	Debug,
	Debug2,
	Debug3,
	Debug4,
	Info,
	Info2,
	Info3,
	Info4,
	Warn,
	Warn2,
	Warn3,
	Warn4,
	Error,
	Error2,
	Error3,
	Error4,
	Fatal,
	Fatal2,
	Fatal3,
	Fatal4
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogRecordFlags {
	Unspecified = 0
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LogRecord {
	pub time_unix_nano:           u64,
	pub severity_number:          Option<SeverityNumber>,
	pub severity_text:            Option<String>,
	pub name:                     Option<String>,
	pub body:                     Option<AnyValue>,
	pub attributes:               Option<Vec<KeyValue>>,
	pub dropped_attributes_count: Option<u32>,
	pub flags:                    Option<u32>,
	pub trace_id:                 Option<TraceId>,
	pub span_id:                  Option<SpanId>
}