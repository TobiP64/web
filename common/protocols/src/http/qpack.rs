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


use std::{
	mem::size_of,
	borrow::Cow
};
use crate::{
	http::headers::{*},
	utils::RingBuffer
};

static HUFFMAN: [(usize, usize); 257] = [
	(0x0000_1ff8, 13),
	(0x007f_ffd8, 23),
	(0x0fff_ffe2, 28),
	(0x0fff_ffe3, 28),
	(0x0fff_ffe4, 28),
	(0x0fff_ffe5, 28),
	(0x0fff_ffe6, 28),
	(0x0fff_ffe7, 28),
	(0x0fff_ffe8, 28),
	(0x00ff_ffea, 24),
	(0x3fff_fffc, 30),
	(0x0fff_ffe9, 28),
	(0x0fff_ffea, 28),
	(0x3fff_fffd, 30),
	(0x0fff_ffeb, 28),
	(0x0fff_ffec, 28),
	(0x0fff_ffed, 28),
	(0x0fff_ffee, 28),
	(0x0fff_ffef, 28),
	(0x0fff_fff0, 28),
	(0x0fff_fff1, 28),
	(0x0fff_fff2, 28),
	(0x3fff_fffe, 30),
	(0x0fff_fff3, 28),
	(0x0fff_fff4, 28),
	(0x0fff_fff5, 28),
	(0x0fff_fff6, 28),
	(0x0fff_fff7, 28),
	(0x0fff_fff8, 28),
	(0x0fff_fff9, 28),
	(0x0fff_fffa, 28),
	(0x0fff_fffb, 28),
	(0x0000_0014, 6),
	(0x0000_03f8, 10),
	(0x0000_03f9, 10),
	(0x0000_0ffa, 12),
	(0x0000_1ff9, 13),
	(0x0000_0015, 6),
	(0x0000_00f8, 8),
	(0x0000_07fa, 11),
	(0x0000_03fa, 10),
	(0x0000_03fb, 10),
	(0x0000_00f9, 8),
	(0x0000_07fb, 11),
	(0x0000_00fa, 8),
	(0x0000_0016, 6),
	(0x0000_0017, 6),
	(0x0000_0018, 6),
	(0x0000_0000, 5),
	(0x0000_0001, 5),
	(0x0000_0002, 5),
	(0x0000_0019, 6),
	(0x0000_001a, 6),
	(0x0000_001b, 6),
	(0x0000_001c, 6),
	(0x0000_001d, 6),
	(0x0000_001e, 6),
	(0x0000_001f, 6),
	(0x0000_005c, 7),
	(0x0000_00fb, 8),
	(0x0000_7ffc, 15),
	(0x0000_0020, 6),
	(0x0000_0ffb, 12),
	(0x0000_03fc, 10),
	(0x0000_1ffa, 13),
	(0x0000_0021, 6),
	(0x0000_005d, 7),
	(0x0000_005e, 7),
	(0x0000_005f, 7),
	(0x0000_0060, 7),
	(0x0000_0061, 7),
	(0x0000_0062, 7),
	(0x0000_0063, 7),
	(0x0000_0064, 7),
	(0x0000_0065, 7),
	(0x0000_0066, 7),
	(0x0000_0067, 7),
	(0x0000_0068, 7),
	(0x0000_0069, 7),
	(0x0000_006a, 7),
	(0x0000_006b, 7),
	(0x0000_006c, 7),
	(0x0000_006d, 7),
	(0x0000_006e, 7),
	(0x0000_006f, 7),
	(0x0000_0070, 7),
	(0x0000_0071, 7),
	(0x0000_0072, 7),
	(0x0000_00fc, 8),
	(0x0000_0073, 7),
	(0x0000_00fd, 8),
	(0x0000_1ffb, 13),
	(0x0007_fff0, 19),
	(0x0000_1ffc, 13),
	(0x0000_3ffc, 14),
	(0x0000_0022, 6),
	(0x0000_7ffd, 15),
	(0x0000_0003, 5),
	(0x0000_0023, 6),
	(0x0000_0004, 5),
	(0x0000_0024, 6),
	(0x0000_0005, 5),
	(0x0000_0025, 6),
	(0x0000_0026, 6),
	(0x0000_0027, 6),
	(0x0000_0006, 5),
	(0x0000_0074, 7),
	(0x0000_0075, 7),
	(0x0000_0028, 6),
	(0x0000_0029, 6),
	(0x0000_002a, 6),
	(0x0000_0007, 5),
	(0x0000_002b, 6),
	(0x0000_0076, 7),
	(0x0000_002c, 6),
	(0x0000_0008, 5),
	(0x0000_0009, 5),
	(0x0000_002d, 6),
	(0x0000_0077, 7),
	(0x0000_0078, 7),
	(0x0000_0079, 7),
	(0x0000_007a, 7),
	(0x0000_007b, 7),
	(0x0000_7ffe, 15),
	(0x0000_07fc, 11),
	(0x0000_3ffd, 14),
	(0x0000_1ffd, 13),
	(0x0fff_fffc, 28),
	(0x000f_ffe6, 20),
	(0x003f_ffd2, 22),
	(0x000f_ffe7, 20),
	(0x000f_ffe8, 20),
	(0x003f_ffd3, 22),
	(0x003f_ffd4, 22),
	(0x003f_ffd5, 22),
	(0x007f_ffd9, 23),
	(0x003f_ffd6, 22),
	(0x007f_ffda, 23),
	(0x007f_ffdb, 23),
	(0x007f_ffdc, 23),
	(0x007f_ffdd, 23),
	(0x007f_ffde, 23),
	(0x00ff_ffeb, 24),
	(0x007f_ffdf, 23),
	(0x00ff_ffec, 24),
	(0x00ff_ffed, 24),
	(0x003f_ffd7, 22),
	(0x007f_ffe0, 23),
	(0x00ff_ffee, 24),
	(0x007f_ffe1, 23),
	(0x007f_ffe2, 23),
	(0x007f_ffe3, 23),
	(0x007f_ffe4, 23),
	(0x001f_ffdc, 21),
	(0x003f_ffd8, 22),
	(0x007f_ffe5, 23),
	(0x003f_ffd9, 22),
	(0x007f_ffe6, 23),
	(0x007f_ffe7, 23),
	(0x00ff_ffef, 24),
	(0x003f_ffda, 22),
	(0x001f_ffdd, 21),
	(0x000f_ffe9, 20),
	(0x003f_ffdb, 22),
	(0x003f_ffdc, 22),
	(0x007f_ffe8, 23),
	(0x007f_ffe9, 23),
	(0x001f_ffde, 21),
	(0x007f_ffea, 23),
	(0x003f_ffdd, 22),
	(0x003f_ffde, 22),
	(0x00ff_fff0, 24),
	(0x001f_ffdf, 21),
	(0x003f_ffdf, 22),
	(0x007f_ffeb, 23),
	(0x007f_ffec, 23),
	(0x001f_ffe0, 21),
	(0x001f_ffe1, 21),
	(0x003f_ffe0, 22),
	(0x001f_ffe2, 21),
	(0x007f_ffed, 23),
	(0x003f_ffe1, 22),
	(0x007f_ffee, 23),
	(0x007f_ffef, 23),
	(0x000f_ffea, 20),
	(0x003f_ffe2, 22),
	(0x003f_ffe3, 22),
	(0x003f_ffe4, 22),
	(0x007f_fff0, 23),
	(0x003f_ffe5, 22),
	(0x003f_ffe6, 22),
	(0x007f_fff1, 23),
	(0x03ff_ffe0, 26),
	(0x03ff_ffe1, 26),
	(0x000f_ffeb, 20),
	(0x0007_fff1, 19),
	(0x003f_ffe7, 22),
	(0x007f_fff2, 23),
	(0x003f_ffe8, 22),
	(0x01ff_ffec, 25),
	(0x03ff_ffe2, 26),
	(0x03ff_ffe3, 26),
	(0x03ff_ffe4, 26),
	(0x07ff_ffde, 27),
	(0x07ff_ffdf, 27),
	(0x03ff_ffe5, 26),
	(0x00ff_fff1, 24),
	(0x01ff_ffed, 25),
	(0x0007_fff2, 19),
	(0x001f_ffe3, 21),
	(0x03ff_ffe6, 26),
	(0x07ff_ffe0, 27),
	(0x07ff_ffe1, 27),
	(0x03ff_ffe7, 26),
	(0x07ff_ffe2, 27),
	(0x00ff_fff2, 24),
	(0x001f_ffe4, 21),
	(0x001f_ffe5, 21),
	(0x03ff_ffe8, 26),
	(0x03ff_ffe9, 26),
	(0x0fff_fffd, 28),
	(0x07ff_ffe3, 27),
	(0x07ff_ffe4, 27),
	(0x07ff_ffe5, 27),
	(0x000f_ffec, 20),
	(0x00ff_fff3, 24),
	(0x000f_ffed, 20),
	(0x001f_ffe6, 21),
	(0x003f_ffe9, 22),
	(0x001f_ffe7, 21),
	(0x001f_ffe8, 21),
	(0x007f_fff3, 23),
	(0x003f_ffea, 22),
	(0x003f_ffeb, 22),
	(0x01ff_ffee, 25),
	(0x01ff_ffef, 25),
	(0x00ff_fff4, 24),
	(0x00ff_fff5, 24),
	(0x03ff_ffea, 26),
	(0x007f_fff4, 23),
	(0x03ff_ffeb, 26),
	(0x07ff_ffe6, 27),
	(0x03ff_ffec, 26),
	(0x03ff_ffed, 26),
	(0x07ff_ffe7, 27),
	(0x07ff_ffe8, 27),
	(0x07ff_ffe9, 27),
	(0x07ff_ffea, 27),
	(0x07ff_ffeb, 27),
	(0x0fff_fffe, 28),
	(0x07ff_ffec, 27),
	(0x07ff_ffed, 27),
	(0x07ff_ffee, 27),
	(0x07ff_ffef, 27),
	(0x07ff_fff0, 27),
	(0x03ff_ffee, 26),
	(0x3fff_ffff, 30)
];

static STATIC_TABLE: std::sync::Once<[HeaderOption; 61]> = std::sync::Once::new();

fn init_static_table() -> [HeaderOption; 61] {
	[
		HeaderId::Authority.into(),
		Header::Method(Method::Get).into(),
		Header::Method(Method::Post).into(),
		Header::Path("/".to_string()).into(),
		Header::Path("/index.html".to_string()).into(),
		Header::Scheme(Scheme::Http).into(),
		Header::Scheme(Scheme::Https).into(),
		Header::Status(StatusCode::Ok).into(),
		Header::Status(StatusCode::NoContent).into(),
		Header::Status(StatusCode::PartialContent).into(),
		Header::Status(StatusCode::NotModified).into(),
		Header::Status(StatusCode::BadRequest).into(),
		Header::Status(StatusCode::NotFound).into(),
		Header::Status(StatusCode::InternalServerError).into(),
		HeaderId::AcceptCharset.into(),
		Header::AcceptEncoding(vec![Encoding::GZip, Encoding::Deflate]).into(),
		HeaderId::AcceptLanguage.into(),
		HeaderId::AcceptRanges.into(),
		HeaderId::AcceptTypes.into(),
		HeaderId::AccessControlAllowOrigin.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Allow.into(),
		HeaderId::Authorization.into(),
		HeaderId::CacheControl.into(),
		HeaderId::ContentDisposition.into(),
		HeaderId::ContentEncoding.into(),
		HeaderId::ContentLanguage.into(),
		HeaderId::ContentLength.into(),
		HeaderId::ContentLocation.into(),
		HeaderId::ContentRange.into(),
		HeaderId::ContentType.into(),
		HeaderId::Cookie.into(),
		HeaderId::Date.into(),
		HeaderId::ETag.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Expires.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Host.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::LastModified.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Location.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Referer.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::Server.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderId::UserAgent.into(),
		HeaderId::Vary.into(),
		HeaderId::__NonExhaustive.into(),
		HeaderOption::Id(HeaderId::__NonExhaustive)
	]
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
	///
	IndexOutOfBounds,
	/// an variable length integer would overflow `usize`
	IntegerOverflow,
	/// the passed buffer was to short for encoding
	BufferOverflow,
	/// the passed buffer was to short for decoding
	BufferUnderflow,
	/// a huffman-encoded string had a padding that was greater than 1 byte
	PaddingTooLong,
	/// a huffman-encoded string had a padding that did not correspond to
	/// the most significatnt bits of the EOS symbol
	PaddingNotEOS,
	/// an index was 0, which is a reserved value
	InvalidIndex,
	/// the new max limit was higher than the limit set by the application protocol
	MaxLimitTooHigh,
	/// a huffman-encoded string contained the EOS code
	UnexpectedEOS,
	NoValueInStaticTable,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EncodeType {
	IncrementalIndexed,
	WithoutIndexing,
	NeverIndexed
}

#[derive(Clone, Debug)]
pub struct EncodingOptions {
	r#type:  EncodeType,
	huffman: bool,
	header:  Header
}

pub struct Encoder {
	max_size_proto: usize,
	max_size:       usize,
	estimated_size: usize,
	table:          RingBuffer<Header>,
	buf:            String
}

impl Encoder {
	pub fn new(max_size: usize) -> Self {
		Self {
			max_size_proto: max_size,
			max_size,
			estimated_size:   0,
			table:  RingBuffer::new(max_size / size_of::<Header>()),
			buf:    String::new()
		}
	}
	
	pub fn encode_max_size(&mut self,
						   block:    &mut [u8],
						   i:        &mut usize,
						   max_size: usize) -> Result<(), Error> {
		if max_size > self.max_size_proto {
			return Err(Error::MaxLimitTooHigh);
		}
		
		encode_integer(3, block, i, max_size, 0b001)
	}
	
	pub fn encode(&mut self,
				  block:   &mut [u8],
				  i:       &mut usize,
				  header:  Header,
				  r#type:  EncodeType,
				  huffman: bool) -> Result<(), Error> {
		use ::std::fmt::Write;
		let idx = self.find_entry(&header);
		
		if idx != 0 {
			encode_integer(1, block, i, idx, 0b1)?;
			return Ok(());
		}
		
		let idx = self.find_name(&header);
		
		if idx != 0 {
			let (prefix, prefix_bits) = match r#type {
				EncodeType::IncrementalIndexed => (2, 0b1),
				EncodeType::WithoutIndexing    => (4, 0b0),
				EncodeType::NeverIndexed       => (4, 0b1)
			};
			encode_integer(prefix, block, i, idx, prefix_bits)?;
		} else {
			block[*i] = match r#type {
				EncodeType::IncrementalIndexed => 0b0100_0000,
				EncodeType::WithoutIndexing    => 0b0000_0000,
				EncodeType::NeverIndexed       => 0b0001_0000
			};
			*i += 1;
			encode_string(block, i, header.id().http2_name(), huffman)?;
		}
		
		self.buf.clear();
		write!(&mut self.buf, "{}", header).unwrap_or(());
		encode_string(block, i, &self.buf, huffman)?;
		
		if let EncodeType::IncrementalIndexed = r#type {
			self.add(header);
		}
		Ok(())
	}
	
	fn add(&mut self, header: Header) {
		if header.estimated_size() > self.max_size {
			self.estimated_size = 0;
			return;
		}
		
		while self.max_size - self.estimated_size < header.estimated_size() {
			self.estimated_size -= self.table.pop()
				.unwrap_or_else(|| unreachable!())
				.estimated_size();
		}
		
		self.estimated_size += header.estimated_size();
		if self.table.push(header).is_some() {
			eprintln!("[HPACK] table overflow");
		}
	}
	
	fn find_entry(&self, header: &Header) -> usize {
		let iter = self.table.iter()
			.enumerate()
			.map(|(i, h)| (i + STATIC_TABLE.len(), h));
		
		STATIC_TABLE.iter()
			.enumerate()
			.filter_map(|(i, h)| match h {
				HeaderOption::Id(_) => None,
				HeaderOption::Header(header) => Some((i, header)),
			})
			.chain(iter)
			.find(|(_, h)| *h == header)
			.map(|(i, _)| i + 1)
			.unwrap_or(0)
	}
	
	fn find_name(&self, header: &Header) -> usize {
		STATIC_TABLE.iter()
			.map(HeaderOption::id)
			.chain(vec![].iter().map(|h: &Header| h.id()))
			.enumerate()
			.find(|(_, id)| *id == header.id())
			.map(|(i, _)| i + 1)
			.unwrap_or(0)
	}
}

pub struct Decoder {
	max_size_proto: usize,
	max_size:       usize,
	/// an estimated size of the table if the headers would be stored in plain text
	estimated_size: usize,
	table:          RingBuffer<Header>,
	buf:            String
}

impl Decoder {
	pub fn new(max_size: usize) -> Self {
		Self {
			max_size_proto: max_size,
			max_size,
			estimated_size: 0,
			table:          RingBuffer::new(max_size / size_of::<Header>()),
			buf:            String::new()
		}
	}
	
	pub fn decode(&mut self, block: &[u8], i: &mut usize) -> Option<Result<Cow<Header>, Error>> {
		let b = block[*i];
		if b & 0b1000_0000 == 0b1000_0000 {          // Indexed Header Field
			let index = match decode_integer(1, block, i) {
				Ok(v) => v,
				Err(e) => return Some(Err(e))
			};
			
			let header = match self.get(index) {
				Ok(v) => v,
				Err(e) => return Some(Err(e))
			};
			
			match header {
				HeaderOptionRef::Id(_) => Some(Err(Error::NoValueInStaticTable)),
				HeaderOptionRef::Header(header) => Some(Ok(Cow::Borrowed(header)))
			}
		} else if b == 0b0100_0000
			|| b == 0b0000_0000
			|| b == 0b0001_0000 {
			
			*i += 1;
			self.buf.clear();
			
			if let Err(e) = decode_string(block, i, &mut self.buf) {
				return Some(Err(e))
			};
			
			let split = self.buf.len();
			
			if let Err(e) = decode_string(block, i, &mut self.buf) {
				return Some(Err(e))
			};
			
			let (name, value) = self.buf.split_at(split);
			
			let header = match Header::from_s(name, value) {
				Ok(v) => v,
				Err(_) => return self.decode(block, i)
			};
			
			Some(Ok(if b == 0b0100_0000 {                   // Literal Header Field with Incremental Indexing -- New Name
				self.add(header)
			} else {                                        // Literal Header Field without Indexing -- New Name / Literal Header Field Never Indexed -- New Name
				Cow::Owned(header)
			}))
		} else if b & 0b1100_0000 == 0b0100_0000
			|| b & 0b1111_0000 == 0b0000_0000
			|| b & 0b1111_0000 == 0b0001_0000 {
			
			self.buf.clear();
			
			let index = match decode_integer(if b & 0b1100_0000 == 0b0100_0000 { 2 } else { 4 }, block, i) {
				Ok(v) => v,
				Err(e) => return Some(Err(e))
			};
			
			let header = match self.get(index) {
				Ok(v) => v,
				Err(e) => return Some(Err(e))
			}.id();
			
			if let Err(e) = decode_string(block, i, &mut self.buf) {
				return Some(Err(e))
			};
			
			let header = match Header::from(header, &self.buf) {
				Ok(h) => h,
				Err(_) => return self.decode(block, i)
			};
			
			Some(Ok(if b & 0b1100_0000 == 0b0100_0000 {      // Literal Header Field with Incremental Indexing -- Indexed Name
				self.add(header)
			} else {                                        // Literal Header Field without Indexing -- Indexed Name / Literal Header Field Never Indexed -- Indexed Name
				Cow::Owned(header)
			}))
		} else if b & 0b1110_0000 == 0b0010_0000 {           // Maximum Dynamic Table Size Change
			
			let max_size = match decode_integer(3, block, i) {
				Ok(v) => v,
				Err(e) => return Some(Err(e))
			};
			
			if max_size > self.max_size_proto {
				return Some(Err(Error::MaxLimitTooHigh))
			}
			
			while self.estimated_size > self.max_size {
				match self.table.pop() {
					Some(header) => self.estimated_size -= header.estimated_size(),
					None => break
				}
			}
			self.decode(block, i)
		} else {
			unreachable!()
		}
	}
	
	fn get(&self, mut index: usize) -> Result<HeaderOptionRef, Error> {
		if index == 0 { return Err(Error::InvalidIndex) }
		
		index -= 1;
		if index < STATIC_TABLE.len() {
			return Ok(STATIC_TABLE[index].as_ref());
		}
		
		index -= STATIC_TABLE.len();
		if index < self.table.len() {
			return Ok(HeaderOptionRef::Header(&self.table[index]));
		}
		
		Err(Error::IndexOutOfBounds)
	}
	
	fn add(&mut self, header: Header) -> Cow<Header> {
		if header.estimated_size() > self.max_size {
			self.estimated_size = 0;
			self.table.clear();
			return Cow::Owned(header);
		}
		
		while self.max_size - self.estimated_size < header.estimated_size() {
			self.estimated_size -= self.table.pop()
				.unwrap_or_else(|| unreachable!())
				.estimated_size();
		}
		
		self.estimated_size += header.estimated_size();
		if self.table.push(header).is_some() {
			eprintln!("[HPACK] table overflow");
		}
		Cow::Borrowed(&self.table[0])
	}
}

fn encode_integer(prefix:      usize,
				  octets:      &mut [u8],
				  i:           &mut usize,
				  mut value:   usize,
				  prefix_bits: u8) -> Result<(), Error> {
	debug_assert!(prefix < 8);
	let mask = !0 >> prefix as u8;
	if value < mask as usize {
		octets[*i] = value as u8 | ((prefix_bits << (8 - prefix)) & (!mask));
		*i += 1;
		return Ok(());
	}
	
	octets[*i] = mask | ((prefix_bits << (8 - prefix)) & (!mask));
	*i += 1;
	value -= mask as usize;
	
	while value > 127 {
		octets[*i] = (value % 128 + 128) as u8;
		*i += 1;
		value /= 128;
		
		if octets.len() <= *i {
			return Err(Error::BufferUnderflow)
		}
	}
	octets[*i] = value as u8;
	*i += 1;
	Ok(())
}

fn decode_integer(prefix: usize,
				  octets: &[u8],
				  i:      &mut usize) -> Result<usize, Error> {
	debug_assert!(prefix < 8);
	let mask = !0 >> prefix as u8;
	let mut value = (octets[*i] & mask) as usize;
	*i += 1;
	
	if value < mask as usize {
		return Ok(value);
	}
	
	let mut m = 0;
	for _ in 0..=5 {
		value += (octets[*i] as usize & 127) << m;
		m += 7;
		*i += 1;
		
		if octets[*i - 1] & 128 == 0 {
			return Ok(value);
		} else if octets.len() <= *i {
			return Err(Error::BufferUnderflow)
		}
	}
	Err(Error::IntegerOverflow)
}

fn encode_string(octets:  &mut [u8],
				 i:       &mut usize,
				 value:   &str,
				 huffman: bool) -> Result<(), Error> {
	if !huffman {
		encode_integer(1, octets, i, value.len(), 0)?;
		octets[*i..].clone_from_slice(value.as_bytes());
		*i += value.len();
	} else {
		let len: usize = (value.chars()
			.map(|ch| HUFFMAN[ch as usize].1)
			.sum::<usize>() + 7) & 0xFFFFFFFFFFFFFFF8;
		
		encode_integer(1, octets, i, len / 8, 1)?;
		
		let mut bit = 0; // bit index in the current byte
		for ch in value.chars() {
			let (code, mut bits) = HUFFMAN[ch as usize];
			let mut code = (code as u32) << (32 - bits as u32); // shift the code to the least significant bits
			
			loop {
				octets[*i] |= (code >> (bit + 24) as u32) as u8;     // move code to the beginning of the first byte
				*i += 1;
				code <<= 8 - bit as u32;                             // delete written bits
				
				if bits < 8 - bit {
					bit += bits;                                     // set bit offset to number of written bits
					*i -= 1;                                         // decrease i by one, because the current bit was not fully filled
					break;
				} else {
					bits -= 8 - bit;                                 // decrease number of remaining bits
					bit = 0;                                         // set bit offset to 0, because we wrote to the next byte boundary
				}
			}
		}
		octets[*i] |= ((HUFFMAN[256].0 as u32) >> (HUFFMAN[256].1 - (8 - bit)) as u32) as u8; // add the EOS symbol as padding
		*i += 1;
	}
	Ok(())
}

pub fn decode_string(octets: &[u8],
					 i:      &mut usize,
					 str:    &mut String) -> Result<(), Error> {
	let huffman = octets[*i] & 128 != 0;
	let length = decode_integer(1, octets, i)?;
	let limit = *i + length;
	if !huffman {
		str.push_str(::std::str::from_utf8(&octets[*i..limit]).unwrap_or(""));
	} else {
		let mut tmp = 0u32;
		octets[*i..*o + 4].copy_from_nonoverlapping(&tmp.to_le_bytes());
		tmp = tmp.swap_bytes();
		let mut bit = 0; // bit index in the current byte
		*i += 4;
		'outer: loop {
			for (ch, (code, mut bits)) in HUFFMAN.iter().copied().enumerate() {
				let mask = !(!0 >> bits as u32);
				let code = (code as u32) << (32 - bits as u32); // shift the code to the least significant bits
				if tmp & mask != code || (*i >= limit && bit < bits) { // check if the mask fits and we have enough bits remaining for this char
					continue;
				} else if ch == 256 {
					return Err(Error::UnexpectedEOS);
				}
				
				str.push(ch as u8 as char);
				tmp <<= bits as u32;                              // remove decoded bits
				
				// read next bits or decrease number of remaining bits
				if *i < limit {                                   // check if there are any bytes to decode
					if 8 - bits as isize >= 0 {                   // do we have to shift right or left to append the new bits correctly?
						tmp |= ((octets[*i] << bit as u8) as u32) // extract next bits
							>> (8 - bits as u32);                 // append bits
					} else {
						tmp |= ((octets[*i] << bit as u8) as u32) // extract next bits
							<< (bits as u32 - 8);                 // append bits
					}
				} else {                                          // check if there are any bits to decode
					bit -= bits;
					continue 'outer;                              // there are still some bits to decode
				}
				
				// check we have enough bits in the current byte
				if bits < 8 - bit {
					bit += bits;                                  // there are enough bits remaining so we do not need to read more bytes
					continue 'outer;
				}
				
				// `bits` is now the number of bits that need to be read to fill `tmp`
				bits -= 8 - bit;                                  // subtract already appended bits
				
				// we do not have enough bits in the current byte, so we have to read the next bytes
				loop {
					*i += 1;
					
					if *i >= limit {
						bit = 32 - bits;                          // `bit` is now the number of remaining bits to decode
						continue 'outer;
					}
					
					// if `bits` is 0 because we read to a byte boundary, we do `tmp |= 0`, so nothing bad happens
					tmp |= (octets[*i] as u32)                    // read next byte
						>> (8 - bits as u32);                     // append bits
					
					if bits < 8 {
						break;                                    // we don't need to read another byte
					} else {
						bits -= 8;
					}
				}
				
				bit = bits;                                       // set how many bits we read from the new current byte
				continue 'outer;
			}
			break;
		}
		if *i < limit {
			return Err(Error::PaddingTooLong);
		}
	}
	*i = limit;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn integer1() {
		let mut buf = [0u8; 1024];
		let mut i = 0;
		assert_eq!(Ok(()), encode_integer(3, &mut buf, &mut i, 12, 0));
		assert_eq!(1, i);
		let mut i = 0;
		assert_eq!(Ok(12), decode_integer(3, &buf, &mut i));
		assert_eq!(1, i);
	}
	
	#[test]
	fn integer2() {
		let mut buf = [0u8; 1024];
		let mut i = 0;
		assert_eq!(Ok(()), encode_integer(5, &mut buf, &mut i, 123, 0));
		assert_eq!(2, i);
		let mut i = 0;
		assert_eq!(Ok(123), decode_integer(5, &buf, &mut i));
		assert_eq!(2, i);
	}
	
	#[test]
	fn integer3() {
		let mut buf = [0u8; 1024];
		let mut i = 0;
		assert_eq!(Ok(()), encode_integer(5, &mut buf, &mut i, 12345, 0));
		assert_eq!(3, i);
		let mut i = 0;
		assert_eq!(Ok(12345), decode_integer(5, &buf, &mut i));
		assert_eq!(3, i);
	}
	
	#[test]
	fn string1() {
		let str = "www.example.com";
		let mut octets = [0u8; 128];
		let mut buf = String::new();
		let mut i = 0;
		assert_eq!(Ok(()), encode_string(&mut octets, &mut i, str, true));
		assert_eq!(13, i);
		i = 0;
		assert_eq!(Ok(()), decode_string(&octets, &mut i, &mut buf));
		assert_eq!(13, i);
		assert_eq!(str, buf.as_str());
	}
	
	#[test]
	fn string2() {
		let str = "no-cache";
		let mut octets = [0u8; 128];
		let mut buf = String::new();
		let mut i = 0;
		assert_eq!(Ok(()), encode_string(&mut octets, &mut i, str, true));
		assert_eq!(7, i);
		i = 0;
		assert_eq!(Ok(()), decode_string(&octets, &mut i, &mut buf));
		assert_eq!(7, i);
		assert_eq!(str, buf.as_str());
	}
	
	#[test]
	fn string3() {
		let str = "ihfgrsiw38747394ndksjhfdiufz-.,-..++SDI98405T3ER";
		let mut octets = [0u8; 128];
		let mut buf = String::new();
		let mut i = 0;
		assert_eq!(Ok(()), encode_string(&mut octets, &mut i, str, true));
		i = 0;
		assert_eq!(Ok(()), decode_string(&octets, &mut i, &mut buf));
		assert_eq!(str, buf.as_str());
	}
	
	#[test]
	fn hpack1() {
		let mut encoder = Encoder::new(1024);
		let mut decoder = Decoder::new(1024);
		let mut buffer = [0u8; 0x1000];
		let mut i = 0;
		
		encoder.encode(&mut buffer, &mut i, Header::Method(Method::Get),
					   EncodeType::IncrementalIndexed, true).unwrap();
		encoder.encode(&mut buffer, &mut i, Header::Path(String::from("/")),
					   EncodeType::IncrementalIndexed, true).unwrap();
		encoder.encode(&mut buffer, &mut i, Header::UserAgent(String::from("some browser")),
					   EncodeType::IncrementalIndexed, true).unwrap();
		
		let len = i;
		i = 0;
		assert_eq!(Some(Ok(Header::Method(Method::Get))), decoder.decode(&mut buffer[0..len], &mut i).map(|r| r.map(|r| r.into_owned())));
		assert_eq!(Some(Ok(Header::Path(String::from("/")))), decoder.decode(&mut buffer[0..len], &mut i).map(|r| r.map(|r| r.into_owned())));
		assert_eq!(Some(Ok(Header::UserAgent(String::from("some browser")))), decoder.decode(&mut buffer[0..len], &mut i).map(|r| r.map(|r| r.into_owned())));
		assert_eq!(len, i);
	}
}