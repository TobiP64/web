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

use std::io;

pub const FIN_BIT:             u8 = 1 << 0;
pub const RSV1_BIT:            u8 = 1 << 1;
pub const RSV2_BIT:            u8 = 1 << 2;
pub const RSV3_BIT:            u8 = 1 << 3;
pub const OPCODE_MASK:         u8 = 0b00001111;
pub const OPCODE_SHIFT:        u8 = 4;
pub const MASK_BIT:            u8 = 1 << 1;
pub const OPCODE_CONTINUATION: u8 = 0x1;
pub const OPCODE_TEXT_FRAME:   u8 = 0x2;
pub const OPCODE_BINARY_FRAME: u8 = 0x8;
pub const OPCODE_CONN_CLOSE:   u8 = 0x8;
pub const OPCODE_PING:         u8 = 0x9;
pub const OPCODE_PONG:         u8 = 0xA;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct FrameHeader {
	pub flags:       u8,
	pub payload_len: u64,
	pub masking_key: Option<u32>,
}

impl FrameHeader {
	pub fn read(mut reader: impl io::Read) -> io::Result<Self> {
		let mut buf = [0u8; 14];
		reader.read_exact(&mut buf[..2])?;
		
		let (payload_len, masking_key) = match buf[1] {
			126 => {
				reader.read_exact(&mut buf[2..4])?;
				(u16::from_be_bytes([buf[2], buf[3]]) as _, None)
			}
			127 => {
				reader.read_exact(&mut buf[2..10])?;
				(
					u64::from_be_bytes([buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9]]),
					None
				)
			}
			254 => {
				reader.read_exact(&mut buf[2..8])?;
				(
					u16::from_be_bytes([buf[2], buf[3]]) as _,
					Some(u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]))
				)
			}
			255 => {
				reader.read_exact(&mut buf[2..])?;
				(
					u64::from_be_bytes([buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9]]),
					Some(u32::from_be_bytes([buf[10], buf[11], buf[12], buf[13]]))
				)
			}
			v if v & MASK_BIT == MASK_BIT => {
				reader.read_exact(&mut buf[2..6])?;
				(
					(v & !MASK_BIT) as _,
					Some(u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]))
				)
			}
			v => (v as _, None)
		};
		
		Ok(Self { flags: buf[0], payload_len, masking_key })
	}
	
	pub fn write(self, mut writer: impl io::Write) -> io::Result<()> {
		match self {
			Self { flags, payload_len, masking_key: None } if payload_len <= 125 => {
				writer.write_all(&[
					flags,
					payload_len as u8
				])
			}
			Self { flags, payload_len, masking_key: None } if payload_len <= u16::MAX as u64 => {
				let payload_len = payload_len.to_be_bytes();
				writer.write_all(&[
					flags,
					127,
					payload_len[0],
					payload_len[1]
				])
			}
			Self { flags, payload_len, masking_key: None }=> {
				let payload_len = payload_len.to_be_bytes();
				writer.write_all(&[
					flags,
					127,
					payload_len[0],
					payload_len[1],
					payload_len[2],
					payload_len[3],
					payload_len[4],
					payload_len[5],
					payload_len[6],
					payload_len[7]
				])
			}
			Self { flags, payload_len, masking_key: Some(masking_key) } if payload_len <= 125 => {
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					payload_len as u8 | MASK_BIT,
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				])
			}
			Self { flags, payload_len, masking_key: Some(masking_key) } if payload_len <= u16::MAX as u64 => {
				let payload_len = payload_len.to_be_bytes();
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					254,
					payload_len[0],
					payload_len[1],
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				])
			}
			Self { flags, payload_len, masking_key: Some(masking_key) }=> {
				let payload_len = payload_len.to_be_bytes();
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					255,
					payload_len[0],
					payload_len[1],
					payload_len[2],
					payload_len[3],
					payload_len[4],
					payload_len[5],
					payload_len[6],
					payload_len[7],
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				])
			}
		}
	}
	
	pub async fn read_async(mut reader: impl futures_lite::io::AsyncReadExt + Unpin) -> io::Result<Self> {
		let mut buf = [0u8; 14];
		reader.read_exact(&mut buf[..2]).await?;
		
		let (payload_len, masking_key) = match buf[1] {
			126 => {
				reader.read_exact(&mut buf[2..4]).await?;
				(u16::from_be_bytes([buf[2], buf[3]]) as _, None)
			}
			127 => {
				reader.read_exact(&mut buf[2..10]).await?;
				(
					u64::from_be_bytes([buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9]]),
					None
				)
			}
			254 => {
				reader.read_exact(&mut buf[2..8]).await?;
				(
					u16::from_be_bytes([buf[2], buf[3]]) as _,
					Some(u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]))
				)
			}
			255 => {
				reader.read_exact(&mut buf[2..]).await?;
				(
					u64::from_be_bytes([buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9]]),
					Some(u32::from_be_bytes([buf[10], buf[11], buf[12], buf[13]]))
				)
			}
			v if v & MASK_BIT == MASK_BIT => {
				reader.read_exact(&mut buf[2..6]).await?;
				(
					(v & !MASK_BIT) as _,
					Some(u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]))
				)
			}
			v => (v as _, None)
		};
		
		Ok(Self { flags: buf[0], payload_len, masking_key })
	}
	
	pub async fn write_async(self, mut writer: impl futures_lite::io::AsyncWriteExt + Unpin) -> io::Result<()> {
		match self {
			Self { flags, payload_len, masking_key: None } if payload_len <= 125 => {
				writer.write_all(&[
					flags,
					payload_len as u8
				]).await
			}
			Self { flags, payload_len, masking_key: None } if payload_len <= u16::MAX as u64 => {
				let payload_len = payload_len.to_be_bytes();
				writer.write_all(&[
					flags,
					127,
					payload_len[0],
					payload_len[1]
				]).await
			}
			Self { flags, payload_len, masking_key: None }=> {
				let payload_len = payload_len.to_be_bytes();
				writer.write_all(&[
					flags,
					127,
					payload_len[0],
					payload_len[1],
					payload_len[2],
					payload_len[3],
					payload_len[4],
					payload_len[5],
					payload_len[6],
					payload_len[7]
				]).await
			}
			Self { flags, payload_len, masking_key: Some(masking_key) } if payload_len <= 125 => {
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					payload_len as u8 | MASK_BIT,
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				]).await
			}
			Self { flags, payload_len, masking_key: Some(masking_key) } if payload_len <= u16::MAX as u64 => {
				let payload_len = payload_len.to_be_bytes();
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					254,
					payload_len[0],
					payload_len[1],
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				]).await
			}
			Self { flags, payload_len, masking_key: Some(masking_key) }=> {
				let payload_len = payload_len.to_be_bytes();
				let masking_key = masking_key.to_be_bytes();
				writer.write_all(&[
					flags,
					255,
					payload_len[0],
					payload_len[1],
					payload_len[2],
					payload_len[3],
					payload_len[4],
					payload_len[5],
					payload_len[6],
					payload_len[7],
					masking_key[0],
					masking_key[1],
					masking_key[2],
					masking_key[3]
				]).await
			}
		}
	}
}