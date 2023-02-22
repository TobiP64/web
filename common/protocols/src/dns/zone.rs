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

use {super::*, std::{str::FromStr, fmt}};

#[derive(Clone, Debug, Default)]
pub struct Zone {
	pub origin:  DomainName,
	pub ttl:     usize,
	pub records: Vec<ResourceRecord>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ZoneParseError {
	pub ty:   ZoneParseErrorType,
	pub line: usize,
	pub row:  usize
}

impl ZoneParseError {
	pub fn new(ty: ZoneParseErrorType, line: usize, row: usize) -> Self {
		Self { ty, line, row }
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ZoneParseErrorType {
	Directive,
	DirectiveValue,
	RecordClass,
	RecordType,
	RecordValue
}

impl FromStr for Zone {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let lines = s.lines().enumerate();
		let mut self_ = Self::default();
		
		for (line_idx, line) in lines {
			let end   = line.find(';').unwrap_or(line.len());
			let mut s = line[..end].split_ascii_whitespace();
			
			match s.next() {
				Some("$ORIGIN") => self_.origin = s.next()
					.ok_or_else(|| ZoneParseError::new(ZoneParseErrorType::DirectiveValue, line_idx, 0))?
					.to_string(),
				Some("$TTL")    => self_.ttl    = s.next()
					.ok_or_else(|| ZoneParseError::new(ZoneParseErrorType::DirectiveValue, line_idx, 0))?
					.parse()
					.map_err(|_| ZoneParseError::new(ZoneParseErrorType::DirectiveValue, line_idx, 0))?,
				Some(v) if v.starts_with('$') => return Err(ZoneParseError::new(ZoneParseErrorType::Directive, line_idx, 0)),
				Some(_) => self_.records.push(line[..end].parse()?),
				None    => continue
			}
		}
		
		Ok(self_)
	}
}

impl fmt::Display for Zone {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "$ORIGIN {}\n$TTL {}\n", &self.origin, self.ttl)?;
		
		for record in &self.records {
			fmt::Display::fmt(record, f)?;
		}
		
		Ok(())
	}
}
