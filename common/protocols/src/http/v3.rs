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

#![allow(dead_code)]

//! https://datatracker.ietf.org/doc/html/draft-ietf-quic-http

use {std::io::Result, crate::quic};

pub struct Connection {
	inner: quic::Connection
}

impl Connection {
	pub fn open(_connection: quic::Connection) -> Result<Self> {
		unimplemented!()
	}
	
	pub fn close(self) {
		unimplemented!()
	}
}

enum FrameType {
	Data          = 0x0,
	Headers       = 0x1,
	Priority      = 0x2,
	CancelPush    = 0x3,
	Settings      = 0x4,
	PushPromise   = 0x5,
	GoAway        = 0x6,
	MaxPushId     = 0x7,
	DuplicatePush = 0x8
}

#[repr(u16)]
enum ErrorCode {
	NoError               = 0x0,
	WrongSettingDirection = 0x1,
	PushRefused           = 0x2,
	InternalError         = 0x3,
	PushAlreadyInCache    = 0x4,
	RequestCancelled      = 0x5,
	IncompleteRequest     = 0x6,
	ConnectError          = 0x7,
	ExcessiveLoad         = 0x8,
	VersionFallback       = 0x9,
	WrongStream           = 0xA,
	LimitExceeded         = 0xB,
	DuplicatePush         = 0xC,
	UnknownStreamType     = 0xD,
	WrongStreamCount      = 0xE,
	ClosedCriticalStream  = 0xF,
	WrongStreamDirection  = 0x10,
	EarlyResponse         = 0x11,
	MissingSettings       = 0x12,
	UnexpectedFrame       = 0x13,
	RequestRejected       = 0x14,
	MalformedFrame        = 0x100
}

pub enum SettingsId {
	MaxHeaderListSize = 0x6,
	NumPlaceholders   = 0x9
}