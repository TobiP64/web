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

//! RTP: A Transport Protocol for Real-Time Applications
//!
//! [RFC 3550](https://datatracker.ietf.org/doc/html/rfc3550)

use std::io;

pub const FLAGS_VERSION_MASK:      u8 = 0b11;
pub const FLAGS_PADDING_BIT:       u8 = 1 << 2;
pub const FLAGS_EXTENSION_BIT:     u8 = 1 << 3;
pub const FLAGS_CSRC_COUNT_MASK:   u8 = 0b00001111;
pub const FLAGS_CSRC_COUNT_SHIFT:  u8 = 4;
pub const PAYLOAD_TYPE_MARKER_BIT: u8 = 1;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Header {
	pub flags:           u8,
	pub payload_type:    u8,
	pub sequence_number: u16,
	pub timestamp:       u32,
	pub ssrc:            u32,
}

impl Header {
	pub fn read(mut reader: impl io::Read, csrc: &mut impl Extend<u32>) -> io::Result<(Self, impl Iterator<Item = u32>)> {
		let mut buf = [0u8; 12 + 4 * 15];
		reader.read_exact(&mut buf[..1])?;
		let cc = (buf[0] & FLAGS_CSRC_COUNT_MASK << FLAGS_CSRC_COUNT_SHIFT) as usize;
		reader.read_exact(&mut buf[1..12 + cc * 4])?;
		Ok((
			Self {
				flags:           buf[0],
				payload_type:    buf[1],
				sequence_number: u16::from_be_bytes([buf[2], buf[3]]),
				timestamp:       u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
				ssrc:            u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
			},
			(0..cc).map(move |i| u32::from_be_bytes([buf[12 + i * 4], buf[13 + i * 4], buf[14 + i * 4], buf[15 + i * 4]]))
		))
	}
	
	pub fn write(self, csrc: impl IntoIterator<Item = u32>, mut writer: impl io::Write) -> io::Result<()> {
		let mut buf = [0u8; 12 + 4 * 15];
		buf[0] = self.flags;
		buf[1] = self.payload_type;
		buf[2..=3].copy_from_slice(&self.sequence_number.to_be_bytes());
		buf[4..=7].copy_from_slice(&self.timestamp.to_be_bytes());
		buf[8..=11].copy_from_slice(&self.ssrc.to_be_bytes());
		
		for (i, csrc) in csrc.into_iter().enumerate() {
			buf[12 + i * 4..16 + i * 4].copy_from_slice(&csrc.to_be_bytes());
		}
		
		writer.write_all(&buf)
	}
}