// MIT License
//
// Copyright (c) 2022 Tobias Pfeiffer
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

use std::time::Duration;

pub struct Config {
	pub server_name: String,
	pub server_tags: Vec<String>,
	pub storage:     Storage,
	pub cluster:     Cluster,
	pub driver:      Driver,
	pub telemetry:   Telemetry,
}

pub struct Storage {
	pub dir:                Option<String>,
	pub in_memory:          bool,
	pub read_only:          bool,
	pub block_size:         usize,
	pub max_cache_size:     usize,
	pub max_queries:        usize,
	pub max_query_bytes:    usize,
	pub max_query_millis:   usize,
	pub max_sessions:       usize,
	pub cursor_timeout:     Duration,
	pub executor_pool_size: usize,
}

pub struct Cluster {
	pub bind:         String,
	pub port:         String,
	pub nodes:        Vec<String>,
	pub tls_key:      String,
	pub tls_cert:     String,
	pub tls_ca:       String,
	pub timeout:      Duration,
	pub allow_read:   bool,
	pub allow_write:  bool,
	pub mirror_reads: f32
}

pub struct Driver {
	pub bind:        String,
	pub port:        String,
	pub tls_key:     Option<String>,
	pub tls_cert:    Option<String>,
	pub tls_ca:      Option<String>,
	pub allow_read:  bool,
	pub allow_write: bool
}

pub struct Telemetry {
	pub enabled:  bool,
	pub name:     String,
	pub endpoint: Option<String>,
	pub protocol: Option<otel_mrt::Protocol>,
	pub timeout:  Option<Duration>,
	pub interval: Option<Duration>
}