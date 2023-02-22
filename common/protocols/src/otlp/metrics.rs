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

//! https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/metrics/v1/metrics.proto
//! https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/collector/metrics/v1/metrics_service.proto

pub use super::*;

pub const HTTP_PATH: &str = "/v1/metrics";

pub type ExportMetricsServiceRequest  = MetricsData;
pub type ExportMetricsServiceResponse = ();

pub trait MetricsService {
	fn export(&self, _: ExportMetricsServiceRequest) -> std::io::Result<ExportMetricsServiceResponse>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MetricsData {
	pub resource_metrics: Vec<ResourceMetrics>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceMetrics {
	pub resource:   resource::Resource,
	pub metrics:    Vec<Metric>,
	pub schema_url: String
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metric {
	pub name:        String,
	pub description: String,
	pub unit:        String,
	pub data:        MetricData
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MetricData {
	Gauge(Gauge),
	Sum(Sum),
	Histogram(Histogram),
	ExponentialHistogram(ExponentialHistogram),
	Summary(Summary)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gauge {
	pub data_points: Vec<NumberDataPoint>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sum {
	pub data_points:             Vec<NumberDataPoint>,
	pub aggregation_temporality: AggregationTemporality,
	pub is_monotonic:            bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Histogram {
	pub data_points:             Vec<HistogramDataPoint>,
	pub aggregation_temporality: AggregationTemporality
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExponentialHistogram {
	pub data_points:             Vec<ExponentialHistogramDataPoint>,
	pub aggregation_temporality: AggregationTemporality
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summary {
	pub data_points: Vec<SummaryDataPoint>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AggregationTemporality {
	Unspecified = 0,
	Delta       = 1,
	Cumulative  = 2
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataPointFlags {
	None,
	NoRecordedValue
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NumberDataPoint {
	pub attributes:           Vec<KeyValue>,
	pub start_time_unix_nano: u64,
	pub time_unix_nano:       u64,
	pub value:                NumberDataPointValue,
	pub exemplars:            Vec<Exemplar>,
	pub flags:                u32
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NumberDataPointValue {
	F64(f64),
	I64(i64)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HistogramDataPoint {
	pub attributes:           Vec<KeyValue>,
	pub start_time_unix_nano: u64,
	pub time_unix_nano:       u64,
	pub count:                u64,
	pub sum:                  f64,
	pub bucket_counts:        Vec<u64>,
	pub explicit_bounds:      Vec<f64>,
	pub exemplars:            Vec<Exemplar>,
	pub flags:                u32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExponentialHistogramDataPoint {
	pub attributes:           Vec<KeyValue>,
	pub start_time_unix_nano: u64,
	pub time_unix_nano:       u64,
	pub count:                u64,
	pub sum:                  f64,
	pub scale:                u32,
	pub zero_count:           u64,
	pub positive:             Buckets,
	pub negative:             Buckets,
	pub exemplars:            Vec<Exemplar>,
	pub flags:                u32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Buckets {
	offset:        i32,
	bucket_counts: Vec<u64>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SummaryDataPoint {
	pub attributes:           Vec<KeyValue>,
	pub start_time_unix_nano: u64,
	pub time_unix_nano:       u64,
	pub count:                u64,
	pub sum:                  f64,
	pub quantile_values:      Vec<ValueAtQuantile>,
	pub flags:                u32
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValueAtQuantile {
	pub quantile: f64,
	pub value:    f64
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Exemplar {
	pub filtered_attributes: Vec<KeyValue>,
	pub time_unix_nano:      u64,
	pub value:               NumberDataPointValue,
	pub span_id:             Option<SpanId>,
	pub trace_id:            Option<TraceId>
}