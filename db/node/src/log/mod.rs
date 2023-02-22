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

use std::io;

pub struct LogService {
	file: smol::lock::Mutex<smol::fs::File>
}

impl LogService {
	pub async fn checkpoint_start(&self, ccns: impl Iterator<Item = u128>) -> io::Result<()> {
		unimplemented!()
	}

	pub async fn checkpoint_end(&self) -> io::Result<()> {
		unimplemented!()
	}

	pub async fn begin(&self, ccn: u128, ops: &[u8]) -> io::Result<()> {
		unimplemented!()
	}

	pub async fn update(&self, ccn: u128, table: u128, field: u128, document: u128, old: &[u8], new: &[u8]) -> io::Result<()> {
		unimplemented!()
	}

	pub async fn commit(&self, ccn: u128) -> io::Result<()> {
		unimplemented!()
	}

	pub async fn abort(&self, ccn: u128, code: u64) -> io::Result<()> {
		unimplemented!()
	}

	async fn write(&self, buf: &[u8]) -> io::Result<()> {
		use smol::io::AsyncWriteExt;
		let mut file = self.file.lock().await;
		file.write_all(buf).await?;
		file.flush().await?;
		Ok(())
	}
}

#[repr(C)]
pub struct Record {
	ccn:  u128,
	len:  u32,
	ty:   RecordType,
	_pad: u8,
	data: RecordData
}

#[repr(u8)]
pub enum RecordType {
	CheckpointStart,
	CheckpointEnd,
	Ops,
	Data,
	Commit,
	Abort
}

pub union RecordData {
	pub checkpoint_start: [u128; 0],
	pub checkpoint_end:   (),
	pub ops:              [u8; 0],
	pub data:             Data,
	pub commit:           (),
	pub abort:            u64
}

pub struct Data {
	pub table:    u128,
	pub field:    u128,
	pub document: u128,
	pub old_len:  u32,
	pub new_len:  u32,
	pub old_new:  [u8; 0]
}