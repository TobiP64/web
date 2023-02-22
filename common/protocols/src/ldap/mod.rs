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

//! Lightweight Directory Access Protocol (LDAP)
//!
//! [RFC 4511](https://datatracker.ietf.org/doc/html/rfc4511)

pub mod ber;
pub mod wire;
pub mod builder;
pub mod traits;
pub mod impls;

pub use {impls::*, wire::*};

pub const DEFAULT_PORT:     u16 = 389;
pub const DEFAULT_PORT_TLS: u16 = 636;
pub const LDAP3:            u8  = 3;

#[cfg(test)]
mod tests {
	use {super::*, crate::utils::pipe::Pipe};
	
	fn connect() -> (Connection<Pipe>, Connection<Pipe>) {
		let (client, server) = Pipe::new();
		(Connection::new(client), Connection::new(server))
	}
	
	#[test]
	fn bind_simple() {
		let (mut client, mut server) = connect();
		
		let id = client.bind(
			LDAP3,
			"cn=ldapadmin,dc=localhost,dc=localdomain",
			AuthenticationChoice::Simple("test".to_string())
		).unwrap();
		
		let msg = server.recv_msg().unwrap();
		
		assert_eq!(Message {
			message_id:         id,
			protocol_op:        ProtocolOp::BindRequest(BindRequest {
				version:        LDAP3,
				name:           "cn=ldapadmin,dc=localhost,dc=localdomain".to_string(),
				authentication: AuthenticationChoice::Simple("test".to_string())
			}),
			controls: None
		}, msg);
		
		let mut response = Message {
			message_id:  0,
			protocol_op: ProtocolOp::BindResponse(BindResponse {
				result: LdapResult {
					result_code:        ResultCode::Success,
					matched_dn:         "".to_string(),
					diagnostic_message: "".to_string(),
					referral:           None
				},
				server_sasl_creds:      None
			}),
			controls: None
		};
		
		server.send_msg(&mut response).unwrap();
		
		let msg = client.recv_msg().unwrap();
		
		assert_eq!(response, msg);
	}
}