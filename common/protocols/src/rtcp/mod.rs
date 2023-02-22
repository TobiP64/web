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

pub const VERSION_2:          u8 = 2;
pub const FLAGS_VERSION_MASK: u8 = 0b11;
pub const FLAGS_PADDING_BIT:  u8 = 1 << 2;
pub const FLAGS_RC_MASK:      u8 = 0b00011111;
pub const FLAGS_RC_SHIFT:     u8 = 3;
pub const PACKET_TYPE_SR:     u8 = 200;
pub const PACKET_TYPE_RR:     u8 = 201;
pub const PACKET_TYPE_SDES:   u8 = 202;
pub const PACKET_TYPE_BYE:    u8 = 203;
pub const PACKET_TYPE_APP:    u8 = 204;
pub const SDES_CNAME:         u8 = 1;
pub const SDES_NAME:          u8 = 2;
pub const SDES_EMAIL:         u8 = 3;
pub const SDES_PHONE:         u8 = 4;
pub const SDES_LOC:           u8 = 5;
pub const SDES_TOOL:          u8 = 6;
pub const SDES_NOTE:          u8 = 7;
pub const SDES_PRIV:          u8 = 8;

pub fn read_packet(mut reader: impl io::Read) -> io::Result<Packet> {
	let header = Header::read(&mut reader)?;
	let mut buf = Vec::with_capacity(header.length as _);
	unsafe { buf.set_len(header.length as _); }
	reader.read_exact(&mut buf)?;
	
	unimplemented!()
}

pub fn recv_packet<T: io::Read>(mut reader: &mut T) -> io::Result<PacketReceiver<T>> {
	let header = Header::read(&mut reader)?;
	let len = (header.length * 4) as usize;
	
	if header.flags & FLAGS_VERSION_MASK != VERSION_2 {
		return Err(io::Error::new(io::ErrorKind::Other, "Unsupported version"));
	}
	
	let count = (header.flags & FLAGS_RC_MASK << FLAGS_RC_SHIFT) as usize;
	Ok(match header.packet_type {
		PACKET_TYPE_SR   => PacketReceiver::SenderReport(PacketSenderReportSenderInfo { reader, count }),
		PACKET_TYPE_RR   => PacketReceiver::ReceiverReport(PacketSenderReportReportBlocks { reader, count }),
		PACKET_TYPE_SDES => PacketReceiver::SourceDescription(PacketSourceDescriptionChunks { reader, count }),
		PACKET_TYPE_BYE  => PacketReceiver::Bye(ByeReceiver { reader, count }),
		PACKET_TYPE_APP  => PacketReceiver::App(AppReceiver { reader, length: len }),
		_                => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown packet type"))
	})
}

pub fn write_packet(writer: impl io::Write, packet: Packet, padding: usize) -> io::Result<()> {
	unimplemented!()
}

/*pub fn send_packet<T: io::Write>(writer: &mut T, buf: &mut Vec<u8>, padding: usize) -> io::Result<PacketSender<T>> {
	let mut buf = Vec::with_capacity(BUF_LEN);
	
	unimplemented!()
}*/

pub enum PacketReceiver<'a, T: io::Read> {
	SenderReport(PacketSenderReportSenderInfo<'a, T>),
	ReceiverReport(PacketSenderReportReportBlocks<'a, T>),
	SourceDescription(PacketSourceDescriptionChunks<'a, T>),
	Bye(ByeReceiver<'a, T>),
	App(AppReceiver<'a, T>)
}

pub struct PacketSenderReportSenderInfo<'a, T: io::Read> {
	reader: &'a mut T,
	count:  usize
}

impl<'a, T: io::Read> PacketSenderReportSenderInfo<'a, T> {
	pub fn read_sender_info(self) -> io::Result<(SenderInfo, PacketSenderReportReportBlocks<'a, T>)> {
		let mut buf = [0u8; 20];
		self.reader.read_exact(&mut buf)?;
		Ok((
			SenderInfo {
				ntp_timestamp:       u64::from_be_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]]),
				rtp_timestamp:       u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
				sender_packet_count: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
				sender_octet_count:  u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]])
			},
			PacketSenderReportReportBlocks {
				reader: self.reader,
				count:  self.count
			}
		))
	}
}

pub struct PacketSenderReportReportBlocks<'a, T: io::Read> {
	reader: &'a mut T,
	count:  usize
}

impl<'a, T: io::Read> PacketSenderReportReportBlocks<'a, T> {
	pub fn read_report_block(mut self) -> io::Result<Result<(ReportBlock, Self), PacketSenderReportExtensions<'a, T>>> {
		if self.count == 0 {
			return Ok(Err(PacketSenderReportExtensions {
				reader: self.reader
			}));
		}
		
		self.count -= 1;
		let mut buf = [0u8; 24];
		self.reader.read_exact(&mut buf)?;
		Ok(Ok((
			ReportBlock {
				ssrc:                    u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
				fraction_lost:           buf[4],
				packets_lost:            [buf[5], buf[6], buf[7]],
				highest_sqeuence_number: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
				interarrival_jitter:     u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
				last_sr:                 u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
				delay_since_last_sr:     u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]])
			},
			self
		)))
	}
}

pub struct PacketSenderReportExtensions<'a, T: io::Read> {
	reader: &'a mut T
}

impl<'a, T: io::Read> PacketSenderReportExtensions<'a, T> {
	pub fn read_extension(self) -> io::Result<Option<()>> {
		unimplemented!()
	}
}

pub struct PacketSourceDescriptionChunks<'a, T: io::Read> {
	reader: &'a mut T,
	count:  usize
}

impl<'a, T: io::Read> PacketSourceDescriptionChunks<'a, T> {
	pub fn read_chunk(mut self) -> io::Result<Option<(u32, PacketSourceDescriptionChunkItems<'a, T>)>> {
		if self.count == 0 {
			return Ok(None);
		}
		
		self.count = 0;
		let mut buf = [0u8; 4];
		self.reader.read_exact(&mut buf)?;
		let ssrc = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
		
		Ok(Some((
			ssrc,
			PacketSourceDescriptionChunkItems {
				reader: self.reader,
				count:  self.count,
				read:   0
			}
		)))
	}
}

pub struct PacketSourceDescriptionChunkItems<'a, T: io::Read> {
	reader: &'a mut T,
	count:  usize,
	read:   usize
}

impl<'a, T: io::Read> PacketSourceDescriptionChunkItems<'a, T> {
	pub fn read_item(mut self, text: &mut String) -> io::Result<Result<(u8, Self), PacketSourceDescriptionChunks<'a, T>>> {
		let mut buf = [0u8; 1];
		self.reader.read_exact(&mut buf)?;
		
		match buf {
			[0] => {
				self.read &= 0b11;
				let mut padding = [0u8; 4];
				self.reader.read_exact(&mut padding[..4 - (self.read & 0b11)])?;
				Ok(Err(PacketSourceDescriptionChunks {
					reader: self.reader,
					count:  self.count
				}))
			}
			[ty] => {
				self.reader.read_exact(&mut buf)?;
				let len = buf[0] as _;
				let mut buf = Vec::new();
				buf.reserve(len);
				unsafe { buf.set_len(len); }
				self.reader.read_exact(&mut buf)?;
				*text = String::from_utf8(buf).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;
				self.read += 2 + len;
				Ok(Ok((ty, self)))
			}
		}
	}
}

pub struct ByeReceiver<'a, T: io::Read> {
	reader: &'a mut T,
	count:  usize,
}

impl<'a, T: io::Read> ByeReceiver<'a, T> {
	pub fn read_ssrc(&mut self) -> io::Result<Option<u32>> {
		unimplemented!()
	}
	
	pub fn read_reason(&mut self) -> io::Result<Option<&str>> {
		unimplemented!()
	}
}

pub struct AppReceiver<'a, T: io::Read> {
	reader: &'a mut T,
	length: usize
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Header {
	pub flags:       u8,
	pub packet_type: u8,
	pub length:      u16
}

impl Header {
	pub fn read(mut reader: impl io::Read) -> io::Result<Self> {
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf)?;
		Ok(Self {
			flags:       buf[0],
			packet_type: buf[1],
			length:      u16::from_be_bytes([buf[2], buf[3]])
		})
	}
	
	pub fn write(self, mut writer: impl io::Write) -> io::Result<()> {
		let mut buf = [0u8; 4];
		buf[0] = self.flags;
		buf[1] = self.packet_type;
		buf[2..=3].copy_from_slice(&self.packet_type.to_be_bytes());
		writer.write_all(&buf)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packet {
	pub header: Header,
	pub r#type: PacketType
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketType {
	SenderReport(SenderReportPacket),
	ReceiverReport(ReceiverReportPacket),
	SourceDescription(SourceDescriptionPacket),
	Bye(ByePacket),
	App(AppPacket)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SenderReportPacket {
	pub sender_ssrc:   u32,
	pub sender_info:   SenderInfo,
	pub report_blocks: Vec<ReportBlock>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SenderInfo {
	pub ntp_timestamp:       u64,
	pub rtp_timestamp:       u32,
	pub sender_packet_count: u32,
	pub sender_octet_count:  u32
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ReportBlock {
	pub ssrc:                    u32,
	pub fraction_lost:           u8,
	pub packets_lost:            [u8; 3],
	pub highest_sqeuence_number: u32,
	pub interarrival_jitter:     u32,
	pub last_sr:                 u32,
	pub delay_since_last_sr:     u32
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverReportPacket {
	pub sender_ssrc:   u32,
	pub report_blocks: Vec<ReportBlock>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDescriptionPacket {
	pub chunks: Vec<SourceDescriptionChunk>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDescriptionChunk {
	pub ssrc_ccsrc: u32,
	pub items:      Vec<SourceDescriptionItem>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDescriptionItem {
	pub ty: u8,
	pub val: String
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByePacket {
	pub ssrc_csrc: Vec<u32>,
	pub reason:    Option<String>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPacket {
	pub ssrc_csrs: u32,
	pub name:      [u8; 4],
	pub data:      Vec<u8>
}