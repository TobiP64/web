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

use {super::*, std::io::{self, Write}};

pub const QR_BIT:                u16 = 1 << 0;
pub const OPCODE_MASK:           u16 = 0b0111100000000000;
pub const OPCODE_SHIFT:          u16 = 1;
pub const OPCODE_QUERY:          u16 = 0;
pub const OPCODE_IQUERY:         u16 = 1;
pub const OPCODE_STATUS:         u16 = 2;
pub const OPCODE_NOTIFY:         u16 = 4;
pub const OPCODE_UPDATE:         u16 = 5;
pub const AA_BIT:                u16 = 1 << 5;
pub const TC_BIT:                u16 = 1 << 6;
pub const RD_BIT:                u16 = 1 << 7;
pub const RA_BIT:                u16 = 1 << 8;
pub const Z_BIT:                 u16 = 1 << 9;
pub const AD_BIT:                u16 = 1 << 10;
pub const CD_BIT:                u16 = 1 << 11;
pub const RCODE_MASK:            u16 = 0b0000000000001111;
pub const RCODE_SHIFT:           u16 = 12;

const DEFAULT_BUF_LEN: usize = 1024;

#[derive(Copy, Clone, Debug , Eq, PartialEq)]
pub struct Header {
	pub id:       u16,
	pub flags:    Flags,
	pub qd_count: u16,
	pub an_count: u16,
	pub ns_count: u16,
	pub ar_count: u16
}

#[derive(Clone, Debug, Default)]
pub struct Message {
	pub id:                 u16,
	pub flags:              Flags,
	pub questions:          Vec<Question>,
	pub answers:            Vec<ResourceRecord>,
	pub authority_records:  Vec<ResourceRecord>,
	pub additional_records: Vec<ResourceRecord>
}

impl Message {
	pub fn read(mut reader: impl io::Read) -> io::Result<Self> {
		let mut buf = [0u8; 12];
		reader.read_exact(&mut buf)?;
		
		Ok(Self {
			id:                 u16::from_le_bytes([buf[0], buf[1]]),
			flags:              Flags(u16::from_le_bytes([buf[2], buf[3]])),
			questions:          (0..u16::from_le_bytes([buf[4], buf[5]])).map(|_| Question::read(&mut reader)).collect::<io::Result<_>>()?,
			answers:            (0..u16::from_le_bytes([buf[6], buf[7]])).map(|_| ResourceRecord::read(&mut reader)).collect::<io::Result<_>>()?,
			authority_records:  (0..u16::from_le_bytes([buf[8], buf[9]])).map(|_| ResourceRecord::read(&mut reader)).collect::<io::Result<_>>()?,
			additional_records: (0..u16::from_le_bytes([buf[10], buf[11]])).map(|_| ResourceRecord::read(&mut reader)).collect::<io::Result<_>>()?
		})
	}
	
	pub async fn read_async(reader: impl futures_lite::AsyncReadExt + Unpin) -> io::Result<Self> {
		unimplemented!()
	}
	
	pub fn write(&self, mut writer: impl io::Write) -> io::Result<()> {
		let mut buf = [0u8; 12];
		buf[0..2].copy_from_slice(&self.id.to_le_bytes());
		buf[2..4].copy_from_slice(&self.flags.0.to_le_bytes());
		buf[4..6].copy_from_slice(&(self.questions.len() as u16).to_le_bytes());
		buf[6..8].copy_from_slice(&(self.answers.len() as u16).to_le_bytes());
		buf[8..10].copy_from_slice(&(self.authority_records.len() as u16).to_le_bytes());
		buf[10..12].copy_from_slice(&(self.additional_records.len() as u16).to_le_bytes());
		writer.write_all(&buf)?;
		
		for question in &self.questions {
			question.write(&mut writer)?;
		}
		
		for rr in &self.answers {
			rr.write(&mut writer)?;
		}
		
		for rr in &self.authority_records {
			rr.write(&mut writer)?;
		}
		
		for rr in &self.additional_records {
			rr.write(&mut writer)?;
		}
		
		Ok(())
	}
	
	pub async fn write_async(&self, mut writer: impl futures_lite::AsyncWriteExt + Unpin) -> io::Result<()> {
		let mut buf = [0u8; 12];
		buf[0..2].copy_from_slice(&self.id.to_le_bytes());
		buf[2..4].copy_from_slice(&self.flags.0.to_le_bytes());
		buf[4..6].copy_from_slice(&(self.questions.len() as u16).to_le_bytes());
		buf[6..8].copy_from_slice(&(self.answers.len() as u16).to_le_bytes());
		buf[8..10].copy_from_slice(&(self.authority_records.len() as u16).to_le_bytes());
		buf[10..12].copy_from_slice(&(self.additional_records.len() as u16).to_le_bytes());
		writer.write_all(&buf).await?;
		
		for question in &self.questions {
			question.write_async(&mut writer).await?;
		}
		
		for rr in &self.answers {
			rr.write_async(&mut writer).await?;
		}
		
		for rr in &self.authority_records {
			rr.write_async(&mut writer).await?;
		}
		
		for rr in &self.additional_records {
			rr.write_async(&mut writer).await?;
		}
		
		Ok(())
	}
	
	pub fn read_with_len(mut reader: impl io::Read) -> io::Result<Self> {
		let mut len = [0u8; 2];
		reader.read_exact(&mut len)?;
		let len = u16::from_be_bytes(len) as usize;
		let mut buf = Vec::with_capacity(len);
		unsafe { buf.set_len(len); }
		reader.read_exact(&mut buf)?;
		Self::read(&mut &*buf)
	}
	
	pub async fn read_async_with_len(mut reader: impl futures_lite::AsyncReadExt + Unpin) -> io::Result<Self> {
		let mut len = [0u8; 2];
		reader.read_exact(&mut len).await?;
		let len = u16::from_be_bytes(len) as usize;
		let mut buf = Vec::with_capacity(len);
		unsafe { buf.set_len(len); }
		reader.read_exact(&mut buf).await?;
		Self::read_async(&mut &*buf).await
	}
	
	pub fn write_with_len(&self, mut writer: impl io::Write) -> io::Result<()> {
		let mut buf = Vec::with_capacity(DEFAULT_BUF_LEN);
		buf.write_all(&[0u8; 2])?;
		self.write(&mut buf)?;
		let __buf_len__ = buf.len();
		buf[..2].copy_from_slice(&(__buf_len__ as u16 - 2).to_be_bytes());
		writer.write_all(&buf)
	}
	
	pub async fn write_async_with_len(&self, mut writer: impl futures_lite::AsyncWriteExt + Unpin) -> io::Result<()> {
		let mut buf = Vec::with_capacity(DEFAULT_BUF_LEN);
		buf.write_all(&[0u8; 2])?;
		self.write(&mut buf)?;
		let __buf_len__ = buf.len();
		buf[..2].copy_from_slice(&(__buf_len__ as u16 - 2).to_be_bytes());
		writer.write_all(&buf).await
	}
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Flags(pub u16);

impl Flags {
	pub fn qr(&self) -> bool {
		self.0 & QR_BIT == QR_BIT
	}
	
	pub fn set_qr(&mut self, v: bool) {
		if v {
			self.0 |= QR_BIT;
		} else {
			self.0 &= !QR_BIT;
		}
	}
	
	pub fn op_code(&self) -> u16 {
		self.0 & OPCODE_MASK << OPCODE_SHIFT
	}
	
	pub fn set_op_code(&mut self, v: u16) {
		self.0 |= v >> OPCODE_SHIFT;
	}
	
	pub fn aa(&self) -> bool {
		self.0 & AA_BIT == AA_BIT
	}
	
	pub fn set_aa(&mut self, v: bool) {
		if v {
			self.0 |= AA_BIT;
		} else {
			self.0 &= !AA_BIT;
		}
	}
	
	pub fn tc(&self) -> bool {
		self.0 & TC_BIT == TC_BIT
	}
	
	pub fn set_tc(&mut self, v: bool) {
		if v {
			self.0 |= TC_BIT;
		} else {
			self.0 &= !TC_BIT;
		}
	}
	
	pub fn rd(&self) -> bool {
		self.0 & QR_BIT == QR_BIT
	}
	
	pub fn set_rd(&mut self, v: bool) {
		if v {
			self.0 |= QR_BIT;
		} else {
			self.0 &= !QR_BIT;
		}
	}
	
	pub fn ra(&self) -> bool {
		self.0 & RA_BIT == RA_BIT
	}
	
	pub fn set_ra(&mut self, v: bool) {
		if v {
			self.0 |= RA_BIT;
		} else {
			self.0 &= !RA_BIT;
		}
	}
	
	pub fn z(&self) -> bool {
		self.0 & Z_BIT == Z_BIT
	}
	
	pub fn set_z(&mut self, v: bool) {
		if v {
			self.0 |= Z_BIT;
		} else {
			self.0 &= !Z_BIT;
		}
	}
	
	pub fn ad(&self) -> bool {
		self.0 & AD_BIT == AD_BIT
	}
	
	pub fn set_ad(&mut self, v: bool) {
		if v {
			self.0 |= AD_BIT;
		} else {
			self.0 &= !AD_BIT;
		}
	}
	
	pub fn cd(&self) -> bool {
		self.0 & CD_BIT == CD_BIT
	}
	
	pub fn set_cd(&mut self, v: bool) {
		if v {
			self.0 |= CD_BIT;
		} else {
			self.0 &= !CD_BIT;
		}
	}
	
	pub fn rcode(&self) -> Result<RCode, u16> {
		let v = self.0 & RCODE_MASK << RCODE_SHIFT;
		RCode::try_from(v).map_err(|_| v)
	}
	
	pub fn set_rcode(&mut self, v: Result<RCode, u16>) {
		self.0 |= crate::utils::unstable::_82223_into_ok_or_err(v.map(|v| v as u16)) >> RCODE_SHIFT;
	}
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RCode {
	NoError               = 0,
	FormatError           = 1,
	ServerFailure         = 2,
	NonExistentDomain     = 3,
	NotImplemented        = 4,
	QueryRefused          = 5,
	YXDomain              = 6,
	YXRRSet               = 7,
	NonExistentRRSet      = 8,
	NotAuthoritive        = 9,
	NotZone               = 10,
	DsoTypeNotImplemented = 11,
	BadVersOrSig          = 16,
	BadKey                = 17,
	BadTime               = 18,
	BadMode               = 19,
	BadName               = 20,
	BadAlg                = 21,
	BadTrunc              = 22,
	BadCookie             = 23
}

impl TryFrom<u16> for RCode {
	type Error = io::Error;
	
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			0..=11 | 16..=23 => Ok(unsafe { std::mem::transmute(value) }),
			_ => Err(io::Error::from(io::ErrorKind::InvalidData))
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Question {
	pub name:   String,
	pub r#type: Type,
	pub class:  Class
}

impl Question {
	fn read(mut reader: impl io::Read) -> io::Result<Self> {
		let name = read_domain_name(&mut reader)?;
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf)?;
		
		Ok(Self {
			name,
			r#type: Type::try_from(u16::from_le_bytes([buf[0], buf[1]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid type"))?,
			class:  Class::try_from(u16::from_le_bytes([buf[2], buf[3]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid class"))?
		})
	}
	
	async fn read_async(mut reader: impl futures_lite::AsyncReadExt + Unpin) -> io::Result<Self> {
		let name = read_domain_name_async(&mut reader).await?;
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf).await?;
		
		Ok(Self {
			name,
			r#type: Type::try_from(u16::from_le_bytes([buf[0], buf[1]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid type"))?,
			class:  Class::try_from(u16::from_le_bytes([buf[2], buf[3]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid class"))?
		})
	}
	
	fn write(&self, mut writer: impl io::Write) -> io::Result<()> {
		write_domain_name(&self.name, &mut writer)?;
		let mut buf = [0u8; 4];
		buf[0..2].copy_from_slice(&(self.r#type as u16).to_le_bytes());
		buf[2..4].copy_from_slice(&(self.class as u16).to_le_bytes());
		writer.write_all(&buf)
	}
	
	async fn write_async(&self, mut writer: impl futures_lite::AsyncWriteExt + Unpin) -> io::Result<()> {
		write_domain_name_async(&self.name, &mut writer).await?;
		let mut buf = [0u8; 4];
		buf[0..2].copy_from_slice(&(self.r#type as u16).to_le_bytes());
		buf[2..4].copy_from_slice(&(self.class as u16).to_le_bytes());
		writer.write_all(&buf).await
	}
}

impl ResourceRecord {
	fn read(mut reader: impl io::Read) -> io::Result<Self> {
		let name = read_domain_name(&mut reader)?;
		let mut buf = [0u8; 10];
		reader.read_exact(&mut buf)?;
		
		Ok(Self {
			name,
			class:  Class::try_from(u16::from_le_bytes([buf[2], buf[3]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid class"))?,
			ttl:    u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
			data:   {
				let len = u16::from_le_bytes([buf[8], buf[9]]) as usize;
				let mut buf = Vec::with_capacity(len);
				unsafe { buf.set_len(len); }
				reader.read_exact(&mut buf)?;
				ResourceRecordData::read(u16::from_le_bytes([buf[0], buf[1]]), &buf)?
			}
		})
	}
	
	async fn read_async(mut reader: impl futures_lite::AsyncReadExt + Unpin) -> io::Result<Self> {
		let name = read_domain_name_async(&mut reader).await?;
		let mut buf = [0u8; 10];
		reader.read_exact(&mut buf).await?;
		
		Ok(Self {
			name,
			class:  Class::try_from(u16::from_le_bytes([buf[2], buf[3]]))
				.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid class"))?,
			ttl:    u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
			data:   {
				let len = u16::from_le_bytes([buf[8], buf[9]]) as usize;
				let mut buf = Vec::with_capacity(len);
				unsafe { buf.set_len(len); }
				reader.read_exact(&mut buf).await?;
				ResourceRecordData::read(u16::from_le_bytes([buf[0], buf[1]]), &buf)?
			}
		})
	}
	
	fn write(&self, mut writer: impl io::Write) -> io::Result<()> {
		write_domain_name(&self.name, &mut writer)?;
		let mut buf = [0u8; 10];
		let mut data = Vec::with_capacity(1024);
		let ty = self.data.write(&mut data);
		buf[0..2].copy_from_slice(&ty.to_le_bytes());
		buf[2..4].copy_from_slice(&(self.class as u16).to_le_bytes());
		buf[4..8].copy_from_slice(&self.ttl.to_le_bytes());
		buf[8..10].copy_from_slice(&(data.len() as u16).to_le_bytes());
		writer.write_all(&buf)?;
		writer.write_all(&data)
	}
	
	async fn write_async(&self, mut writer: impl futures_lite::AsyncWriteExt + Unpin) -> io::Result<()> {
		write_domain_name_async(&self.name, &mut writer).await?;
		let mut buf = [0u8; 10];
		let mut data = Vec::with_capacity(1024);
		let ty = self.data.write(&mut data);
		buf[0..2].copy_from_slice(&ty.to_le_bytes());
		buf[2..4].copy_from_slice(&(self.class as u16).to_le_bytes());
		buf[4..8].copy_from_slice(&self.ttl.to_le_bytes());
		buf[8..10].copy_from_slice(&(data.len() as u16).to_le_bytes());
		writer.write_all(&buf).await?;
		writer.write_all(&data).await
	}
}

fn read_domain_name(mut reader: impl io::Read) -> io::Result<String> {
	let mut len  = [0];
	let mut name = Vec::new();
	
	loop {
		reader.read_exact(&mut len[..1])?;
		let len = len[0] as usize;
		
		if len == 0 {
			break;
		}
		
		let off = name.len();
		name.reserve(len + 1);
		unsafe { name.set_len(off + len + 1); }
		reader.read_exact(&mut name[off..off + len])?;
		name[off + len] = b'.';
	}
	
	String::from_utf8(name).map_err(
		|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))
}

async fn read_domain_name_async(mut reader: impl futures_lite::AsyncReadExt + Unpin) -> io::Result<String> {
	let mut len  = [0];
	let mut name = Vec::new();
	
	loop {
		reader.read_exact(&mut len[..1]).await?;
		let len = len[0] as usize;
		
		if len == 0 {
			break;
		}
		
		let off = name.len();
		name.reserve(len + 1);
		unsafe { name.set_len(off + len + 1); }
		reader.read_exact(&mut name[off..off + len]).await?;
		name[off + len] = b'.';
	}
	
	String::from_utf8(name).map_err(
		|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))
}

fn write_domain_name(name: &str, mut writer: impl io::Write) -> io::Result<()> {
	for name in name.split('.') {
		writer.write_all(&[name.len() as u8])?;
		writer.write_all(name.as_bytes())?;
	}
	
	Ok(())
}

async fn write_domain_name_async(name: &str, mut writer: impl futures_lite::AsyncWriteExt + Unpin) -> io::Result<()> {
	for name in name.split('.') {
		writer.write_all(&[name.len() as u8]).await?;
		writer.write_all(name.as_bytes()).await?;
	}
	
	Ok(())
}