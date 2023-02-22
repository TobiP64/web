// MIT License
//
// Copyright (c) 2021 Tobias Pfeiffer
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

use std::{io, borrow::Cow};

pub enum Command<'a> {
	/// This command is used to identify the SMTP client to the SMTP
	/// server.
	Helo(Cow<'a, str>),
	/// This command is used to identify the SMTP client to the SMTP
	/// server.
	Ehlo(Cow<'a, str>),
	/// This command is used to initiate a mail transaction in which the mail
	/// data is delivered to an SMTP server that may, in turn, deliver it to
	/// one or more mailboxes or pass it on to another system (possibly using
	/// SMTP).
	Mail(Cow<'a, str>),
	/// This command is used to identify an individual recipient of the mail
	/// data; multiple recipients are specified by multiple uses of this
	/// command.
	Rcpt(Cow<'a, str>),
	Data,
	/// This command specifies that the receiver MUST send a "221 OK" reply,
	/// and then close the transmission channel.
	Rset,
	/// This command asks the receiver to confirm that the argument identifies
	/// a user or mailbox.
	Vrfy(Cow<'a, str>),
	/// This command asks the receiver to confirm that the argument
	/// identifies a mailing list, and if so, to return the membership of
	/// that list.
	Expn(Cow<'a, str>),
	/// This command causes the server to send helpful information to the
	/// client.
	Help(Option<Cow<'a, str>>),
	/// This command causes the server to send helpful information to the
	/// client.
	Noop,
	/// This command specifies that the receiver MUST send a "221 OK" reply,
	/// and then close the transmission channel.
	Quit,
	Other(Cow<'a, str>, Option<Cow<'a, str>>)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ReplyCode {
	SystemStatus                       = 211,
	HelpMessage                        = 214,
	ServiceReady                       = 220,
	ServiceClosing                     = 221,
	ReqMailActOk                       = 250,
	UserNotLocalWillForward            = 251,
	VerifyFAiled                       = 252,
	StartMailInput                     = 354,
	ServiceNotAvailable                = 421,
	MailboxTemporaryUnavailable        = 450,
	LocalError                         = 451,
	InsufficientStorage                = 452,
	UnableToAccommodateParams          = 455,
	InvalidCommand                     = 500,
	InvalidParams                      = 501,
	CommandNotImplemented              = 502,
	BadCommandSequence                 = 503,
	ParamNotImplemented                = 504,
	MailboxPermanentlyUnavailable      = 550,
	UserNotLocal                       = 551,
	ExceededStorageAllocation          = 552,
	MailboxNameNotAllowed              = 553,
	TransactionFailed                  = 554,
	MailFromRcptToParamsNotImplemented = 555
}

impl TryFrom<[u8; 3]> for ReplyCode {
	type Error = io::Error;
	
	fn try_from(v: [u8; 3]) -> std::result::Result<Self, Self::Error> {
		Ok(match &v {
			b"211" => Self::SystemStatus,
			b"214" => Self::HelpMessage,
			b"220" => Self::ServiceReady,
			b"221" => Self::ServiceClosing,
			b"250" => Self::ReqMailActOk,
			b"251" => Self::UserNotLocalWillForward,
			b"252" => Self::VerifyFAiled,
			b"354" => Self::StartMailInput,
			b"421" => Self::ServiceNotAvailable,
			b"450" => Self::MailboxTemporaryUnavailable,
			b"451" => Self::LocalError,
			b"452" => Self::InsufficientStorage,
			b"455" => Self::UnableToAccommodateParams,
			b"500" => Self::InvalidCommand,
			b"501" => Self::InvalidParams,
			b"502" => Self::CommandNotImplemented,
			b"503" => Self::BadCommandSequence,
			b"504" => Self::ParamNotImplemented,
			b"550" => Self::MailboxPermanentlyUnavailable,
			b"551" => Self::UserNotLocal,
			b"552" => Self::ExceededStorageAllocation,
			b"553" => Self::MailboxNameNotAllowed,
			b"554" => Self::TransactionFailed,
			b"555" => Self::MailFromRcptToParamsNotImplemented,
			_      => return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid SMTP reply code"))
		})
	}
}