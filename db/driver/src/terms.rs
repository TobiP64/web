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

use uuid::Uuid;

extern "C" {
	pub fn insert(db: Uuid, table: Uuid) -> u32;

	pub fn lookup_key(db: Uuid, table: Uuid, index: Uuid, key: *mut [u8]) -> u32;

	pub fn lookup_range(db: Uuid, table: Uuid, index: Uuid, from: *mut [u8], to: *mut [u8]) -> u32;

	pub fn lookup_all(db: Uuid, table: Uuid, index: Uuid) -> u32;

	pub fn cursor_fetch(cursor: u32) -> bool;

	pub fn cursor_fetch_concurrent(cursor: u32, function: fn()) -> bool;

	pub fn cursor_close(cursor: u32);

	pub fn cursor_delete(cursor: u32);

	pub fn cursor_replace(cursor: u32);

	pub fn cursor_read_column(cursor: u32, column: Uuid) -> *mut [u8];

	pub fn cursor_read_next_column(cursor: u32) -> (Uuid, *mut [u8]);

	pub fn cursor_write_column(cursor: u32, column: Uuid, len: Option<usize>) -> *mut u8;
}

