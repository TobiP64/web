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

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields, from = "ConfigEnum<Config>")]
pub struct Config {
	pub enabled:  bool,
	pub name:     String,
	pub endpoint: Option<String>,
	pub protocol: Option<ConfigProtocol>,
	pub timeout:  Option<Duration>,
	pub interval: Option<Duration>
}

impl From<ConfigEnum<Config>> for Config {
	fn from(v: ConfigEnum<Config>) -> Self {
		match v {
			ConfigEnum::Bool(false) => Self::default(),
			ConfigEnum::Bool(true)  => Self { enabled: true, ..Self::default() },
			ConfigEnum::Config(cfg) => cfg
		}
	}
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigProtocol {
	Grpc,
	HttpJson,
	HttpProto,
}

#[allow(clippy::from_over_into)]
impl Into<otel_mrt::Protocol> for ConfigProtocol {
	fn into(self) -> otel_mrt::Protocol {
		match self {
			Self::Grpc      => otel_mrt::Protocol::Grpc,
			Self::HttpJson  => otel_mrt::Protocol::HttpJson,
			Self::HttpProto => otel_mrt::Protocol::HttpProto
		}
	}
}