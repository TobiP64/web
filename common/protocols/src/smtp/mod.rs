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

//! Simple Mail Transfer Protocol
//!
//! [RFC 5321](https://datatracker.ietf.org/doc/html/rfc5321)

pub mod wire;
pub mod traits;
pub mod impls;

pub use {wire::*, impls::*};

pub const DEFAULT_PORT:     u16 = 25;
pub const DEFAULT_PORT_TLS: u16 = 587;

/*pub trait ClientConnection: BufRead + Write {
	fn recv_greeting(&mut self, buf: &mut String) -> Result<ReplyCode>;
	
	fn send(&mut self, cmd: &Command, buf: &mut String) -> Result<ReplyCode>;
}

pub trait ServerConnection: BufRead + Write {
	fn recv(&mut self) -> Result<Command>;
	
	fn reply<'a>(&mut self, code: ReplyCode, lines: impl IntoIterator<Item = &'a str>) -> Result<()>;
}

pub trait AsyncClientConnection: AsyncBufRead + AsyncWrite  {
	fn poll_send(self: Pin<&mut Self>, cx: &mut Context<'_>, cmd: &Command, buf: &mut String) -> Poll<Result<ReplyCode>>;
}

pub trait AsyncServerConnection: AsyncBufRead + AsyncWrite {
	fn poll_recv(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Command>>;
	
	fn poll_send<'a>(self: Pin<&mut Self>, cx: &mut Context<'_>, code: ReplyCode, lines: impl IntoIterator<Item = &'a str>) -> Poll<Result<()>>;
}

pub struct Connection<T> {
	stream: T
}

impl<T> Connection<T> {
	pub fn new(inner: T) -> Self {
		Self { stream: inner }
	}
}

impl<T: BufRead + Write> ClientConnection for Connection<T> {
	fn recv_greeting(&mut self, buf: &mut String) -> Result<ReplyCode> {
		loop {
			let mut bytes = [0u8; 4];
			self.stream.read_exact(&mut bytes)?;
			let code = ReplyCode::try_from([bytes[0], bytes[1], bytes[2]])?;
			self.stream.read_line(&mut*buf)?;
			
			match bytes[3] {
				b'-' => continue,
				b' ' => return Ok(code),
				_   => return Err(Error::new(ErrorKind::InvalidData, "invalid reply message format"))
			}
		}
	}
	
	fn send(&mut self, cmd: &Command, buf: &mut String) -> Result<ReplyCode> {
		match cmd {
			Command::Helo(v)             => write!(&mut self.stream, "HELO {}\r\n", v),
			Command::Ehlo(v)             => write!(&mut self.stream, "EHLO {}\r\n", v),
			Command::Mail(v)             => write!(&mut self.stream, "MAIL FROM:{}\r\n", v),
			Command::Rcpt(v)             => write!(&mut self.stream, "RCPT TO:{}\r\n", v),
			Command::Data                => write!(&mut self.stream, "DATA\r\n"),
			Command::Rset                => write!(&mut self.stream, "RSET\r\n"),
			Command::Vrfy(v)             => write!(&mut self.stream, "VRFY {}\r\n", v),
			Command::Expn(v)             => write!(&mut self.stream, "EXPN {}\r\n", v),
			Command::Help(None)          => write!(&mut self.stream, "HELP\r\n"),
			Command::Help(Some(v))       => write!(&mut self.stream, "HELP {}\r\n", v),
			Command::Noop                => write!(&mut self.stream, "NOOP\r\n"),
			Command::Quit                => write!(&mut self.stream, "QUIT\r\n"),
			Command::Other(cmd, None)    => write!(&mut self.stream, "{}\r\n", cmd),
			Command::Other(cmd, Some(v)) => write!(&mut self.stream, "{} {}\r\n", cmd, v),
		}?;
		
		self.recv_greeting(buf)
	}
}

impl<T: BufRead + Write> ServerConnection for Connection<T> {
	fn recv(&mut self) -> Result<Command> {
		let mut cmd = [0u8; 4];
		let mut buf = String::new();
		self.stream.read_exact(&mut cmd)?;
		self.stream.read_line(&mut buf)?;
		
		Ok(match &cmd {
			b"HELO" => Command::Helo(Cow::Owned(buf)),
			b"EHLO" => Command::Ehlo(Cow::Owned(buf)),
			b"RCPT" => Command::Rcpt(Cow::Owned(buf)),
			b"DATA" => Command::Data,
			b"RSET" => Command::Rset,
			b"VRFY" => Command::Vrfy(Cow::Owned(buf)),
			b"EXPN" => Command::Expn(Cow::Owned(buf)),
			b"HELP" => Command::Help((!buf.is_empty()).then(|| Cow::Owned(buf))),
			b"NOOP" => Command::Noop,
			b"QUIT" => Command::Quit,
			cmd     => Command::Other(
				String::from_utf8_lossy(cmd).to_owned(),
				(!buf.is_empty()).then(|| Cow::Owned(buf))
			)
		})
	}
	
	fn reply<'a>(&mut self, code: ReplyCode, lines: impl IntoIterator<Item = &'a str>) -> Result<()> {
		let mut lines = lines.into_iter();
		let mut next: &str = if let Some(line) = lines.next() {
			line
		} else {
			return write!(&mut self.stream, "{}\r\n", code as u32);
		};
		
		loop {
			if let Some(line) = lines.next() {
				write!(&mut self.stream, "{}-{}\r\n", code as u32, next)?;
				next = line;
			} else {
				write!(&mut self.stream, "{} {}\r\n", code as u32, next)?;
				break;
			}
		}
		
		Ok(())
	}
}

impl<T: AsyncBufRead + AsyncWrite + Unpin> AsyncClientConnection for Connection<T> {
	fn poll_send<'a>(self: Pin<&'a mut Self>, cx: &mut Context<'_>, cmd: &Command, buf: &mut String) -> Poll<Result<ReplyCode>> {
		todo!()
	}
}

impl<T: AsyncBufRead + AsyncWrite + Unpin> AsyncServerConnection for Connection<T> {
	fn poll_recv(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Command>> {
		todo!()
	}
	
	fn poll_send<'a>(self: Pin<&mut Self>, cx: &mut Context<'_>, code: ReplyCode, lines: impl IntoIterator<Item = &'a str>) -> Poll<Result<()>> {
		todo!()
	}
}

impl<T: Read> Read for Connection<T> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		self.stream.read(buf)
	}
	
	fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
		self.stream.read_vectored(bufs)
	}
	
	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
		self.stream.read_to_end(buf)
	}
	
	fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
		self.stream.read_to_string(buf)
	}
	
	fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stream.read_exact(buf)
	}
}

impl<T: BufRead> BufRead for Connection<T> {
	fn fill_buf(&mut self) -> Result<&[u8]> {
		self.stream.fill_buf()
	}
	
	fn consume(&mut self, amt: usize) {
		self.stream.consume(amt)
	}
	
	fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
		self.stream.read_until(byte, buf)
	}
	
	fn read_line(&mut self, buf: &mut String) -> Result<usize> {
		self.stream.read_line(buf)
	}
}

impl<T: Write> Write for Connection<T> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.stream.write(buf)
	}
	
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
		self.stream.write_vectored(bufs)
	}
	
	fn flush(&mut self) -> Result<()> {
		self.stream.flush()
	}
	
	fn write_all(&mut self, buf: &[u8]) -> Result<()> {
		self.stream.write_all(buf)
	}
	
	fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> Result<()> {
		self.stream.write_fmt(fmt)
	}
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Connection<T> {
	fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
		Pin::new(&mut (*self).stream).poll_write(cx, buf)
	}
	
	fn poll_write_vectored(mut self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>]) -> Poll<Result<usize>> {
		Pin::new(&mut (*self).stream).poll_write_vectored(cx, bufs)
	}
	
	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		Pin::new(&mut (*self).stream).poll_flush(cx)
	}
	
	fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		Pin::new(&mut (*self).stream).poll_close(cx)
	}
}

impl<T: AsyncRead + Unpin> AsyncRead for Connection<T> {
	fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
		Pin::new(&mut (*self).stream).poll_read(cx, buf)
	}
	
	fn poll_read_vectored(mut self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [IoSliceMut<'_>]) -> Poll<Result<usize>> {
		Pin::new(&mut (*self).stream).poll_read_vectored(cx, bufs)
	}
}

impl<T: AsyncBufRead + Unpin> AsyncBufRead for Connection<T> {
	fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.stream) }.poll_fill_buf(cx)
	}
	
	fn consume(mut self: Pin<&mut Self>, amt: usize) {
		Pin::new(&mut (*self).stream).consume(amt)
	}
}

impl<T> ops::Deref for Connection<T> {
	type Target = T;
	
	fn deref(&self) -> &Self::Target {
		&self.stream
	}
}*/

#[cfg(all(test, feature = "assert_matches"))]
mod tests {
	use super::*;
	
	#[test]
	fn connect() {
		const GREETING: &str = "localhost Simple Mail Transfer Service Ready";
		
		let (client, server) = crate::utils::pipe::Pipe::new_buffered();
		let (mut client, mut server) = (super::Connection::new(client), super::Connection::new(server));
		let mut buf = String::new();
		
		assert_matches!(server.reply(ReplyCode::ServiceReady, [GREETING]), Ok(()));
		assert_matches!(server.flush(), Ok(()));
		assert_matches!(client.recv_greeting(&mut buf), Ok(ReplyCode::ServiceReady));
		assert_eq!(buf.trim_end(), GREETING);
		buf.clear();
		
		assert_matches!(client.send(&Command::Ehlo(Cow::Borrowed("localhost")), &mut buf), Ok(ReplyCode::ReqMailActOk));
	}
}