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

pub mod wire;
pub mod terms;
pub mod rawops;
pub mod monitor;
pub mod executor;
pub mod metadata;

pub struct Request {
	pub ccn:              u128,
	pub query:            Vec<u8>,
	pub flags:            u32,
	pub min_batch_rows:   usize,
	pub max_batch_rows:   usize,
	pub min_batch_bytes:  usize,
	pub max_batch_bytes:  usize,
	pub min_batch_millis: usize,
	pub max_batch_millis: usize,
	pub total_rows:       usize,
	pub total_bytes:      usize,
	pub total_millis:     usize,
}

pub const QUERY_FLAG_NO_REPLY:        u32 = 0x1;
pub const QUERY_FLAG_PROFILE:         u32 = 0x2;
pub const QUERY_FLAG_READ_SOFT:       u32 = 0x4;
pub const QUERY_FLAG_READ_HARD:       u32 = 0x8;
pub const QUERY_FLAG_READ_SINGLE:     u32 = 0x10;
pub const QUERY_FLAG_READ_MAJORITY:   u32 = 0x20;
pub const QUERY_FLAG_READ_COMMITTED:  u32 = 0x40;
pub const QUERY_FLAG_READ_SNAPSHOT:   u32 = 0x80;
pub const QUERY_FLAG_WRITE_SOFT:      u32 = 0x100;
pub const QUERY_FLAG_WRITE_HARD:      u32 = 0x200;
pub const QUERY_FLAG_WRITE_SINGLE:    u32 = 0x400;
pub const QUERY_FLAG_WRITE_MAJORITY:  u32 = 0x800;
pub const QUERY_FLAG_WRITE_COMMITTED: u32 = 0x1000;
pub const QUERY_FLAG_WRITE_SNAPSHOT:  u32 = 0x2000;

pub struct Result {
	pub status:    Status,
	pub error:     Option<String>,
	pub backtrace: Option<Vec<u32>>,
	pub profile:   Option<()>,
}

pub enum Status {
	Success,
	InvalidCCN,
	ResourceLimitSoft,
	ResourceLimitHard,
	TimeoutSoft,
	TimeoutHard,
	Authentication,
	Authorization,
	QueryLogic,
	QueryNonExistence,
	ClusterUnknown,
	ClusterRecoverable,
	ClusterFatal,
	InternalUnknown,
	InternalRecoverable,
	InternalFatal,
}
