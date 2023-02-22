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

use std::sync::atomic::AtomicU64;
use uuid::Uuid;

pub struct Topology {
	pub topology:  Vec<Node>,
	pub tables:    Vec<Table>
}

#[derive(Clone, Debug)]
pub struct Node {
	pub id:           Uuid,
	pub name:         String,
	pub addr:         String,
	pub version:      u32,
	pub labels:       Vec<String>,
	pub chunks:       Vec<(Uuid, TableChunk)>,
	pub status:       NodeStatus,
	pub ccn:          AtomicU64,
	pub last_update:  AtomicU64,
	pub last_contact: AtomicU64,
	pub rtt:          AtomicU64
}

#[derive(Copy, Clone, Debug)]
pub enum NodeStatus {
	Init,
	Ready,
	Outdated,
	Error,
	Lost
}

#[derive(Copy, Clone, Debug)]
pub enum ChunkStatus {
	Init,
	Ready,
	Outdated
}

#[derive(Clone, Debug)]
pub struct Table {
	pub id:             Uuid,
	pub database:       String,
	pub table:          String,
	pub chunks:         Vec<TableChunk>
}

#[derive(Copy, Clone, Debug)]
pub struct TableChunk {
	pub node:        Uuid,
	pub range_start: Uuid,
	pub range_end:   Uuid,
	pub status:      ChunkStatus,
	pub flags:       u32
}

impl TableChunk {
	pub const CLUSTER_READS:  u32 = 0x1;
	pub const CLUSTER_WRITES: u32 = 0x2;
	pub const DRIVER_READS:   u32 = 0x4;
	pub const DRIVER_WRITES:  u32 = 0x8;
	pub const INDEX_ONLY:     u32 = 0x10;
}