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

//! QUIC: A UDP-Based Multiplexed and Secure Transport
//!
//! [RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)

#![allow(dead_code)]

pub mod wire;
pub mod traits;
pub mod impls;

use std::net::{UdpSocket, ToSocketAddrs, IpAddr};
use std::io;
use std::sync::Arc;

pub struct Listener {
	inner: Arc<ListenerInner>
}

struct ListenerInner {
	socket: UdpSocket
}

impl Listener {
	pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
		Ok(Self {
			inner: Arc::new(ListenerInner {
				socket: UdpSocket::bind(addr)?
			})
		})
	}
	
	pub fn accept(&self) -> io::Result<Connection> {
		unimplemented!()
	}
}

pub struct Connection {
	peer:       IpAddr,
	buf:        Box<[u8]>,
	ids:        [usize; 64],
	peer_ids:   [usize; 64],
	flow_limit: usize
}

impl Connection {
	pub fn accept(&self) -> io::Result<Stream> {
		unimplemented!()
	}
	
	pub fn init_bidi(&self) -> io::Result<Stream> {
		unimplemented!()
	}
	
	pub fn init_uni(&self) -> io::Result<Stream> {
		unimplemented!()
	}
}

pub struct Stream {
	id:         usize,
	recv_state: ReceivingStreamState,
	recv_limit: usize,
	send_state: SendingStreamState,
	send_limit: usize
}

impl Stream {
	pub fn stop_sending(&self) -> io::Result<()> {
		unimplemented!()
	}
}

enum LongHeaderPacketType {
	Initial   = 0x0,
	ZeroRTT   = 0x1,
	Handshake = 0x2,
	Retry     = 0x3
}

enum FrameType {
	Padding                  = 0x00,
	Ping                     = 0x01,
	Ack                      = 0x02,
	AckECN                   = 0x03,
	ResetStream              = 0x04,
	StopSending              = 0x05,
	Crypto                   = 0x06,
	NewToken                 = 0x07,
	Stream0                  = 0x08,
	Stream1                  = 0x09,
	Stream2                  = 0x0A,
	Stream3                  = 0x0B,
	Stream4                  = 0x0C,
	Stream5                  = 0x0D,
	Stream6                  = 0x0E,
	Stream7                  = 0x0F,
	MaxData                  = 0x10,
	MaxStreamData            = 0x11,
	MaxStreamsBidi           = 0x12,
	MaxStreamsUni            = 0x13,
	DataBlocked              = 0x14,
	StreamDataBlocked        = 0x15,
	StreamsBlockedBidi       = 0x16,
	StreamsBlockedUni        = 0x17,
	NewConnectionId          = 0x18,
	RetireConnectionId       = 0x19,
	PathChallenge            = 0x1A,
	PathResponse             = 0x1B,
	ConnectionCloseQuicError = 0x1C,
	ConnectionCloseAppError  = 0x1D
}

enum SendingStreamState {
	Ready,
	Send,
	DataSent,
	DataRecvd,
	ResetSent,
	ResetRecvd
}

enum ReceivingStreamState {
	Recv,
	SizeKnown,
	DataRecvd,
	DataRead,
	ResetRecvd,
	ResetRead
}

enum TransportParameter {
	OriginalConnectionId           = 0x0,
	IdleTimeout                    = 0x1,
	StatelessResetToken            = 0x2,
	MaxPacketSize                  = 0x3,
	InitialMaxData                 = 0x4,
	InitialMaxStreamDataBidiLocal  = 0x5,
	InitialMaxStreamDataBidiRemote = 0x6,
	InitialMaxStreamDataUni        = 0x7,
	InitialMaxStreamsBidi          = 0x8,
	InitialMaxStreamsUni           = 0x9,
	AckDelayExponent               = 0xA,
	MaxAckDelay                    = 0xB,
	DisableMigration               = 0xC,
	PreferredAddress               = 0xD
}

#[allow(clippy::enum_variant_names)]
enum Error {
	NoError                 = 0x0,
	InternalError           = 0x1,
	ServerBusy              = 0x2,
	FlowControlError        = 0x3,
	StreamLimitError        = 0x4,
	StreamStateError        = 0x5,
	FinalSizeError          = 0x6,
	FrameEncodingError      = 0x7,
	TransportParameterError = 0x8,
	ProtocolViolation       = 0xA,
	InvalidMigration        = 0xC
}