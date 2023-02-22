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

//! Salted Challenge Response Authentication Mechanism (SCRAM)
//!
//! [RFC 5802](https://datatracker.ietf.org/doc/html/rfc5802)

use {std::io::{self, Write}, hmac::{Mac, Hmac, NewMac}, digest::*, pbkdf2::pbkdf2};

pub const SCRAM_SHA_256:  &str  = "SCRAM-SHA-256";
pub const MIN_ITERATIONS: usize = 4096;
pub const BUFFER_LEN:     usize = 1024;

#[derive(Copy, Clone, Debug)]
pub struct User<'a, const N: usize> {
	pub iterations: usize,
	pub salt:       &'a [u8],
	pub server_key: [u8; N],
	pub stored_key: [u8; N]
}

#[derive(Debug)]
pub enum Error {
	SaslPrepFailed,
	InvalidReply,
	InvalidNonce,
	InvalidSignatureOrProof,
	InvalidCredentials,
	InvalidIterationCount,
	Io(io::Error)
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Self::Io(e)
	}
}

pub fn client_auth<
	D: Update + BlockInput + FixedOutput + Reset + Default + Clone + Sync,
	S: io::Read + io::Write,
	const N: usize
>(
	mut stream:   S,
	user:         &str,
	password:     &str,
	client_nonce: &str
) -> Result<(), Error>
	where digest::generic_array::GenericArray<u8, <D as FixedOutput>::OutputSize>: Into<[u8; N]>
{
	let user         = stringprep::saslprep(user).map_err(|_| Error::SaslPrepFailed)?;
	let password     = stringprep::saslprep(password).map_err(|_| Error::SaslPrepFailed)?;
	let mut buf_full = [0u8; BUFFER_LEN];
	
	// client first message
	
	let ptr = buf_full.as_ptr() as usize;
	let mut buf = &mut buf_full[..];
	buf.write_all(b"n,,n=")?;
	buf.write_all(user.as_bytes())?;
	buf.write_all(b",r=")?;
	buf.write_all(client_nonce.as_bytes())?;
	
	let off = buf.as_ptr() as usize - ptr;
	let (client_first, mut buf) = buf_full.split_at_mut(off);
	stream.write_all(client_first)?;
	
	buf[0] = b',';
	buf = &mut buf[1..];
	
	// server first message
	
	let len = stream.read(buf)?;
	let (server_first, mut buf) = buf.split_at_mut(len);
	let server_first = std::str::from_utf8(server_first)
		.map_err(|_| Error::InvalidReply)?;
	
	let combined_nonce = sasl_get(server_first, "r=")
		.ok_or(Error::InvalidReply)?;
	
	let salt = base64::decode(sasl_get(server_first, "s=")
		.ok_or(Error::InvalidReply)?)
		.map_err(|_| Error::InvalidReply)?;
	
	let iterations = sasl_get(server_first, "i=")
		.ok_or(Error::InvalidReply)?
		.parse::<usize>()
		.map_err(|_| Error::InvalidReply)?;
	
	if !password.is_empty() && iterations < MIN_ITERATIONS {
		return Err(Error::InvalidIterationCount);
	} else if !combined_nonce.starts_with(&client_nonce) {
		return Err(Error::InvalidNonce);
	}
	
	// client final message
	
	buf[0] = b',';
	buf = &mut buf[1..];
	
	let mut buf_tmp = &mut*buf;
	buf_tmp.write_all(b"c=biws,r=")?;
	buf_tmp.write_all(combined_nonce.as_bytes())?;
	
	// save buffer offsets ------------------------------
	let mut __buf_tmp_off__ = buf_tmp.as_ptr() as usize;
	let mut __buf_off__ = buf.as_ptr() as usize;
	__buf_tmp_off__ -= buf_full.as_ptr() as usize;
	__buf_off__ -= buf_full.as_ptr() as usize;
	// --------------------------------------------------
	
	let off              = __buf_tmp_off__;
	//let off              = buf_tmp.as_ptr() as usize - buf_full.as_ptr() as usize;
	let auth_msg         = &buf_full[3..off];
	let mut salted_pwd   = [0u8; N];
	pbkdf2::<Hmac<D>>(password.as_bytes(), &salt, iterations as _, &mut salted_pwd);
	let client_key       = hmac_varkey::<D, N>(salted_pwd, b"Client Key");
	let server_key       = hmac_varkey::<D, N>(salted_pwd, b"Server Key");
	let stored_key       = D::digest(client_key.as_ref()).into();
	let client_signature = hmac_varkey::<D, N>(stored_key, auth_msg);
	let server_signature = hmac_varkey::<D, N>(server_key, auth_msg);
	let mut client_proof = [0u8; N];
	
	for i in 0..32 {
		client_proof[i] = client_key[i] ^ client_signature[i];
	}
	
	// reconstruct buffers -------------------------------------
	let buf = &mut buf_full[__buf_off__..];
	let mut buf_tmp = &mut buf[..__buf_tmp_off__ - __buf_off__];
	// ---------------------------------------------------------
	
	buf_tmp.write_all(b",p=")?;
	let len = base64::encode_config_slice(&client_proof, base64::STANDARD, buf_tmp);
	buf_tmp = &mut buf_tmp[len..];
	
	let end = buf_tmp.as_ptr() as usize - buf.as_ptr() as usize;
	stream.write_all(&buf[..end])?;
	
	// server final message
	
	let buf = &mut buf_full[..];
	let len = stream.read(buf)?;
	let server_last = std::str::from_utf8(&buf[..len])
		.map_err(|_| Error::InvalidReply)?;
	
	let recv_server_signature = base64::decode(sasl_get(server_last, "v=")
		.ok_or(Error::InvalidReply)?)
		.map_err(|_| Error::InvalidReply)?;
	
	if recv_server_signature != server_signature {
		return Err(Error::InvalidSignatureOrProof);
	}
	
	// cleanup
	buf_full.iter_mut().for_each(|b| *b = 0);
	Ok(())
}

pub fn server_auth<
	D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
	S: io::Read + io::Write,
	const N: usize
>(
	mut stream: S,
	server_nonce: &str,
	get_user: impl FnOnce(&str) -> Option<User<{N}>>
) -> Result<(), Error>
	where digest::generic_array::GenericArray<u8, <D as FixedOutput>::OutputSize>: Into<[u8; N]>
{
	let mut buf_full = [0u8; BUFFER_LEN];
	let buf = &mut buf_full[..];
	
	// client first message
	
	let len = stream.read(buf)?;
	let (client_first, mut buf) = buf.split_at_mut(len);
	let client_first = std::str::from_utf8(client_first)
		.map_err(|_| Error::InvalidReply)?;
	
	if !client_first.starts_with("n,,") {
		return Err(Error::InvalidReply);
	}
	
	let user = sasl_get(client_first, "n=")
		.ok_or(Error::InvalidReply)?;
	
	let client_nonce = sasl_get(client_first, "r=")
		.ok_or(Error::InvalidReply)?;
	
	let User { iterations, salt, server_key, stored_key } = get_user(user).ok_or(Error::InvalidCredentials)?;
	
	buf[0] = b',';
	buf = &mut buf[1..];
	
	// server first message
	
	let mut buf_tmp = &mut*buf;
	buf_tmp.write_all(b"r=")?;
	buf_tmp.write_all(client_nonce.as_bytes())?;
	buf_tmp.write_all(server_nonce.as_bytes())?;
	buf_tmp.write_all(b",s=")?;
	let len = base64::encode_config_slice(salt, base64::STANDARD, buf_tmp);
	buf_tmp = &mut buf_tmp[len..];
	buf_tmp.write_all(b",i=")?;
	write!(&mut buf_tmp, "{}", iterations)?;
	
	let off = buf_tmp.as_ptr() as usize - buf.as_ptr() as usize;
	let (server_first, mut buf) = buf.split_at_mut(off);
	stream.write_all(server_first)?;
	
	buf[0] = b',';
	buf = &mut buf[1..];
	
	// client last message
	
	let len = stream.read(buf)?;
	let (client_last, buf) = buf.split_at_mut(len);
	let client_last = std::str::from_utf8(client_last)
		.map_err(|_| Error::InvalidReply)?;
	
	let combined_nonce = sasl_get(client_last, "r=")
		.ok_or(Error::InvalidReply)?;
	
	let client_proof = base64::decode(sasl_get(client_last, "p=")
		.ok_or(Error::InvalidReply)?)
		.map_err(|_| Error::InvalidReply)?;
	
	if !client_last.starts_with("c=biws,") {
		return Err(Error::InvalidReply);
	} else if &combined_nonce[..server_nonce.len()] == server_nonce
		&& &combined_nonce[server_nonce.len()..] == client_nonce {
		return Err(Error::InvalidNonce);
	} else if client_proof.len() != N {
		return Err(Error::InvalidSignatureOrProof);
	}
	
	// TODO trim client proof
	let auth_msg_len = buf.as_ptr() as usize - buf_full.as_ptr() as usize;
	let auth_msg = &buf_full[3..auth_msg_len];
	let client_signature = hmac_varkey::<D, N>(stored_key, auth_msg);
	let mut client_key = [0u8; N];
	
	for i in 0..N {
		client_key[i] = client_signature[i] ^ client_proof[i];
	}
	
	let hashed_client_key: [u8; N] = D::digest(client_key.as_ref()).into();
	if hashed_client_key != stored_key {
		return Err(Error::InvalidSignatureOrProof);
	}
	
	// server final message
	
	let server_signature = hmac_varkey::<D, N>(server_key, auth_msg);
	
	let mut buf = &mut buf_full[..];
	buf.write_all(b"v=")?;
	let len = base64::encode_config_slice(&server_signature, base64::STANDARD, buf);
	buf = &mut buf[len..];
	
	let off = buf.as_ptr() as usize - buf_full.as_ptr() as usize;
	stream.write_all(&buf_full[..off])?;
	
	// cleanup
	buf_full.iter_mut().for_each(|b| *b = 0);
	Ok(())
}


pub fn gen_nonce<const N: usize>() -> [u8; N] {
	use rand::Rng;
	let mut nonce = [0u8; N];
	for e in &mut nonce {
		*e = rand::thread_rng().gen_range(0x30u8, 0x5Bu8);
	}
	nonce
}

fn sasl_get<'a>(msg: &'a str, attr: &str) -> Option<&'a str> {
	let off = msg.find(attr)? + attr.len();
	let len = msg[off..].find(',').unwrap_or(msg.len() - off);
	Some(&msg[off..off + len])
}

fn hmac_varkey<
	D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
	const N: usize
>(key: [u8; N], data: &[u8]) -> [u8; N] where
	digest::generic_array::GenericArray<u8, <D as FixedOutput>::OutputSize>: Into<[u8; N]>
{
	let mut hmac = Hmac::<D>::new_varkey(&key).unwrap();
	hmac.update(data);
	hmac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_auth() {
	
	}
}