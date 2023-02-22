// MIT License
//
// Copyright (c) 2019-2023  Tobias Pfeiffer
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

use {std::io::*, futures_lite::{AsyncRead, AsyncReadExt}};

pub const BER_BOOLEAN:      u8 = 0x01;
pub const BER_INTEGER:      u8 = 0x02;
pub const BER_OCTET_STRING: u8 = 0x04;
pub const BER_NULL:         u8 = 0x05;
pub const BER_ENUMERATED:   u8 = 0x0A;
pub const BER_SEQUENCE:     u8 = 0x30;
pub const BER_SET:          u8 = 0x31;

pub type BerTag = u8;

pub trait BerRead: Read + Sized {
	fn read_ber(&mut self) -> Result<(BerTag, usize)> {
		let mut buf = [0u8; 10];
		self.read_exact(&mut buf[..2])?;
		
		let len = if buf[1] & 0x80 == 0 {
			buf[1] as _
		} else if buf[1] & !0x80 > 8 {
			return Err(Error::new(ErrorKind::InvalidData, "integer too long"));
		} else {
			let __tmp_buf_1__ = buf[1];
			self.read_exact(&mut buf[10 - (__tmp_buf_1__ & !0x80) as usize..])?;
			u64::from_be_bytes([buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9]]) as _
		};
		
		Ok((buf[0], len))
	}
	
	fn ber_reader(mut self) -> Result<(BerTag, std::io::Take<Self>)> {
		let (tag, len) = self.read_ber()?;
		Ok((tag, self.take(len as _)))
	}
	
	fn read_ber_to_buf(&mut self, buf: &mut Vec<u8>) -> Result<(BerTag, usize)> {
		let (tag, len) = self.read_ber()?;
		let off = buf.len();
		buf.reserve(len);
		unsafe { buf.set_len(off + len); }
		self.read_exact(&mut buf[off..off + len])?;
		Ok((tag, len))
	}
	
	fn read_ber_data(&mut self, tag: BerTag) -> Result<Vec<u8>> {
		let (read_tag, len) = self.read_ber()?;
		
		if read_tag != tag {
			return Err(Error::new(ErrorKind::InvalidData, "invalid tag"))
		}
		
		let mut buf = Vec::with_capacity(len);
		unsafe { buf.set_len(len); }
		self.read_exact(&mut buf)?;
		Ok(buf)
	}
	
	fn read_ber_null(&mut self, tag: BerTag) -> Result<()> {
		if self.read_ber()? != (tag, 0) {
			Err(Error::new(ErrorKind::InvalidData, "invalid tag or length"))
		} else {
			Ok(())
		}
	}
	
	fn read_ber_bool(&mut self, tag: BerTag) -> Result<bool> {
		if self.read_ber()? != (tag, 1) {
			Err(Error::new(ErrorKind::InvalidData, "invalid tag or length"))
		} else {
			let mut buf = [0];
			self.read_exact(&mut buf)?;
			Ok(buf[0] != 0x00)
		}
	}
	
	fn read_ber_int(&mut self, tag: BerTag) -> Result<i64> {
		let (read_tag, len) = self.read_ber()?;
		
		if read_tag != tag || len > 8 {
			return Err(Error::new(ErrorKind::InvalidData, "invalid tag or integer too long"));
		}
		
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf[8 - len..])?;
		Ok(i64::from_be_bytes(buf))
	}
	
	fn read_ber_str(&mut self, tag: BerTag) -> Result<String> {
		String::from_utf8(self.read_ber_data(tag)?)
			.map_err(|e| Error::new(ErrorKind::InvalidData, e))
	}
}

impl<T: Read> BerRead for T {}

pub trait BerWrite: Write {
	fn write_ber(&mut self, tag: BerTag, len: usize) -> Result<()> {
		if len == 0 {
			self.write_all(&[tag, 0x00])
		} else if len < 128 {
			self.write_all(&[tag, len as u8])
		} else {
			self.write_all(&[tag, 0x88])?;
			self.write_all(&len.to_be_bytes())
		}
	}
	
	fn write_ber_data(&mut self, tag: BerTag, data: &[u8]) -> Result<()> {
		self.write_ber(tag, data.len())?;
		
		if !data.is_empty() {
			self.write_all(data)?;
		}
		
		Ok(())
	}
	
	fn write_ber_null(&mut self, tag: BerTag) -> Result<()> {
		self.write_ber_data(tag, &[])
	}
	
	fn write_ber_bool(&mut self, tag: BerTag, v: bool) -> Result<()> {
		self.write_ber_data(tag, &[if v { 0xFF } else { 0x00 }])
	}
	
	fn write_ber_int(&mut self, tag: BerTag, v: i64) -> Result<()> {
		self.write_ber_data(tag, &(v as i32).to_be_bytes())
	}
	
	fn write_ber_str(&mut self, tag: BerTag, v: &str) -> Result<()> {
		self.write_ber_data(tag, v.as_bytes())
	}
}

impl<T: Write> BerWrite for T {}

pub struct BerWriter<'a>(&'a mut Vec<u8>, usize);

impl<'a> BerWriter<'a> {
	pub fn new(buf: &'a mut Vec<u8>, tag: BerTag) -> Self {
		buf.write_all(&[tag, 0x82, 0, 0]).unwrap();
		Self(buf, buf.len())
	}
}

impl<'a> std::ops::Deref for BerWriter<'a> {
	type Target = Vec<u8>;
	
	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

impl<'a> std::ops::DerefMut for BerWriter<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut*self.0
	}
}

impl<'a> Write for BerWriter<'a> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.0.write(buf)
	}
	
	fn flush(&mut self) -> Result<()> {
		self.0.flush()
	}
	
	fn write_all(&mut self, buf: &[u8]) -> Result<()> {
		self.0.write_all(buf)
	}
	
	fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> Result<()> {
		self.0.write_fmt(fmt)
	}
}

impl<'a> Drop for BerWriter<'a> {
	fn drop(&mut self) {
		let Self(buf, off) = self;
		let len = buf.len() - *off;
		buf[*off - 2..*off].copy_from_slice(&(len as u16).to_be_bytes());
	}
}

pub async fn ber_read_async(mut reader: impl AsyncRead + Unpin) -> Result<(BerTag, usize)> {
	let mut buf = [0u8; 9];
	reader.read_exact(&mut buf[..2]).await?;
	
	let len = if buf[1] & 0x80 == 0 {
		buf[1] as _
	} else if buf[1] & 0x80 > 8 {
		return Err(Error::new(ErrorKind::InvalidData, "integer too long"));
	} else {
		let __tmp_buf_1__ = buf[1];
		reader.read_exact(&mut buf[9 - (__tmp_buf_1__ & 0x80) as usize..]).await?;
		u64::from_be_bytes([buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8]]) as _
	};
	
	Ok((buf[0], len))
}

pub async fn ber_read_to_buf_async(mut reader: impl AsyncRead + Unpin, buf: &mut Vec<u8>) -> Result<(BerTag, usize)> {
	let (tag, len) = ber_read_async(&mut reader).await?;
	let off = buf.len();
	buf.reserve(len);
	unsafe { buf.set_len(off + len); }
	reader.read_exact(&mut buf[off..off + len]).await?;
	Ok((tag, len))
}

pub fn unexp_tag_err<T>(tag: BerTag) -> Error {
	Error::new(ErrorKind::InvalidData, format!("unexpected tag {:#X} in {}", tag, std::any::type_name::<T>()))
}

pub struct BerSeqIter<F: FnMut() -> Result<T>, T>(pub F);

impl<F: FnMut() -> Result<T>, T> BerSeqIter<F, T> {
	pub fn next<'a>(&'a mut self) -> Result<Option<T>>  where T: 'a {
		match (self.0)() {
			Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
			Err(e) => Err(e),
			Ok(v)  => Ok(Some(v))
		}
	}
}

impl<F: FnMut() -> Result<T>, T> Iterator for BerSeqIter<F, T> {
	type Item = Result<T>;
	
	fn next(&mut self) -> Option<Self::Item> {
		match (self.0)() {
			Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => None,
			Err(e) => Some(Err(e)),
			Ok(v)  => Some(Ok(v))
		}
	}
}