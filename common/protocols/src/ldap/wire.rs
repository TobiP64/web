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

use {ProtocolOp::*, super::ber::*, std::{io, sync::atomic::AtomicU32}};

pub type MessageId = u32;
pub type DN        = String;
pub type Oid       = String;
pub type Uri       = String;

const PROTOCOL_OP_BIND_REQUEST:            u8 = 0x60;
const PROTOCOL_OP_BIND_RESPONSE:           u8 = 0x61;
const PROTOCOL_OP_UNBIND_REQUEST:          u8 = 0x42;
const PROTOCOL_OP_SEARCH_REQUEST:          u8 = 0x63;
const PROTOCOL_OP_SEARCH_RESULT_ENTRY:     u8 = 0x64;
const PROTOCOL_OP_SEARCH_RESULT_DONE:      u8 = 0x65;
const PROTOCOL_OP_MODIFY_REQUEST:          u8 = 0x66;
const PROTOCOL_OP_MODIFY_RESPONSE:         u8 = 0x67;
const PROTOCOL_OP_ADD_REQUEST:             u8 = 0x68;
const PROTOCOL_OP_ADD_RESPONSE:            u8 = 0x69;
const PROTOCOL_OP_DELETE_REQUEST:          u8 = 0x4a;
const PROTOCOL_OP_DELETE_RESPONSE:         u8 = 0x6b;
const PROTOCOL_OP_MODIFY_DN_REQUEST:       u8 = 0x6c;
const PROTOCOL_OP_MODIFY_DN_RESPONSE:      u8 = 0x6d;
const PROTOCOL_OP_COMPARE_REQUEST:         u8 = 0x6e;
const PROTOCOL_OP_COMPARE_RESPONSE:        u8 = 0x6f;
const PROTOCOL_OP_ABANDON_REQUEST:         u8 = 0x50;
const PROTOCOL_OP_SEARCH_RESULT_REFERENCE: u8 = 0x73;
const PROTOCOL_OP_EXTENDED_REQUEST:        u8 = 0x77;
const PROTOCOL_OP_EXTENDED_RESPONSE:       u8 = 0x78;
const PROTOCOL_OP_INTERMEDIATE_RESPONSE:   u8 = 0x79;

const AUTH_SIMPLE:       u8 = 0x80;
const AUTH_SASL:         u8 = 0xA3;
const SERVER_SASL_CREDS: u8 = 0x87;
const RESULT_REFERRAL:   u8 = 0xA3;

static MESSAGE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

#[allow(unused_must_use)]
pub fn write_msg(msg: &Message, buf: &mut Vec<u8>) {
	let mut writer = BerWriter::new(buf, BER_SEQUENCE);
	writer.write_ber_int(BER_INTEGER, msg.message_id as _);
	
	match &msg.protocol_op {
		ProtocolOp::BindRequest(v) => {
			let mut writer = BerWriter::new(&mut writer, PROTOCOL_OP_BIND_REQUEST);
			writer.write_ber_int(BER_INTEGER, v.version as _);
			writer.write_ber_str(BER_OCTET_STRING, &v.name);
			
			match &v.authentication {
				AuthenticationChoice::Simple(simple) => {
					writer.write_ber_str(AUTH_SIMPLE, simple);
				}
				AuthenticationChoice::Sasl(sasl) => {
					let mut writer = BerWriter::new(&mut writer, AUTH_SASL);
					writer.write_ber_str(BER_OCTET_STRING, &sasl.mechanism);
					
					if let Some(credentials) = &sasl.credentials {
						writer.write_ber_str(BER_OCTET_STRING, credentials);
					}
				}
			}
		}
		ProtocolOp::BindResponse(v) => {
			let mut writer = BerWriter::new(&mut writer, PROTOCOL_OP_BIND_RESPONSE);
			v.result.write_components(&mut writer);
			
			if let Some(credentials) = &v.server_sasl_creds {
				writer.write_ber_str(SERVER_SASL_CREDS, credentials);
			}
		}
		ProtocolOp::UnbindRequest => {
			writer.write_ber_null(PROTOCOL_OP_UNBIND_REQUEST);
		}
		_ => unimplemented!()
	}
}

pub fn read_msg(mut reader: impl io::Read) -> io::Result<Message> {
	use std::io::Read;
	
	Ok(Message {
		message_id:  reader.read_ber_int(BER_INTEGER)? as _,
		protocol_op: match reader.ber_reader()? {
			(PROTOCOL_OP_BIND_REQUEST, mut reader) => BindRequest(BindRequest {
				version:        reader.read_ber_int(BER_INTEGER)? as _,
				name:           reader.read_ber_str(BER_OCTET_STRING)?,
				authentication: match reader.ber_reader()? {
					(AUTH_SIMPLE, mut reader) => AuthenticationChoice::Simple({
						let mut buf = Vec::new();
						reader.read_to_end(&mut buf)?;
						String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
					}),
					(AUTH_SASL, mut reader)   => AuthenticationChoice::Sasl(SaslCredentials {
						mechanism: reader.read_ber_str(BER_OCTET_STRING)?,
						credentials: match reader.read_ber_str(BER_OCTET_STRING) {
							Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
							Err(e) => return Err(e),
							Ok(s) => Some(s)
						}
					}),
					_ => return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid authentication choice"))
				}
			}),
			(PROTOCOL_OP_BIND_RESPONSE, mut reader) => BindResponse(BindResponse {
				result:            LdapResult::read_components(&mut reader)?,
				server_sasl_creds: match reader.read_ber_str(SERVER_SASL_CREDS) {
					Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
					Err(e) => return Err(e),
					Ok(s) => Some(s)
				}
			}),
			(PROTOCOL_OP_UNBIND_REQUEST, _) => UnbindRequest,
			_ => return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid protocol op"))
		},
		controls:    None
	})
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
	pub message_id:  MessageId,
	pub protocol_op: ProtocolOp,
	pub controls:    Option<Vec<Control>>
}

impl Default for Message {
	fn default() -> Self {
		Self {
			message_id:  0,
			protocol_op: ProtocolOp::UnbindRequest,
			controls:    None
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolOp {
	BindRequest(BindRequest),
	BindResponse(BindResponse),
	UnbindRequest,
	SearchRequest(SearchRequest),
	SearchResultEntry(SearchResultEntry),
	SearchResultDone(LdapResult),
	SearchResultReference(SearchResultReference),
	ModifyRequest(ModifyRequest),
	ModifyResponse(LdapResult),
	AddRequest(AddRequest),
	AddResponse(LdapResult),
	DelRequest(DN),
	DelResponse(LdapResult),
	ModifyDNRequest(ModifyDNRequest),
	ModifyDNResponse(LdapResult),
	CompareRequest(CompareRequest),
	CompareResponse(LdapResult),
	AbandonRequest(AbandonRequest),
	ExtendedRequest(ExtendedRequest),
	ExtendedResponse(ExtendedResponse),
	IntermediateResponse(IntermediateResponse)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchResult {
	Entry(SearchResultEntry),
	Done(LdapResult),
	Reference(SearchResultReference)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Control {
	pub control_type: Oid,
	pub criticality:   bool,
	pub control_value: Option<String>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LdapResult {
	pub result_code:        ResultCode,
	pub matched_dn:         String,
	pub diagnostic_message: String,
	pub referral:           Option<Vec<Uri>>
}

impl LdapResult {
	#[allow(unused_must_use)]
	fn write_components(&self, buf: &mut Vec<u8>) {
		buf.write_ber_int(BER_ENUMERATED, self.result_code as _);
		buf.write_ber_str(BER_OCTET_STRING, &self.matched_dn);
		buf.write_ber_str(BER_OCTET_STRING, &self.diagnostic_message);
		
		if let Some(referral) = &self.referral {
			let mut writer = BerWriter::new(&mut*buf, RESULT_REFERRAL);
			
			for uri in referral {
				writer.write_ber_str(BER_OCTET_STRING, uri);
			}
		}
	}
	
	fn read_components(mut reader: impl io::Read) -> io::Result<Self> {
		let mut result = Self {
			result_code:        ResultCode::from_int(reader.read_ber_int(BER_ENUMERATED)? as _)
				.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid result code"))?,
			matched_dn:         reader.read_ber_str(BER_OCTET_STRING)?,
			diagnostic_message: reader.read_ber_str(BER_OCTET_STRING)?,
			referral:           None
		};
		
		//let reader = &mut reader;
		//let mut iter = BerSeqIter(move || BerRead::ber_reader(reader));
		
		while let Some((tag, mut reader)) = match (&mut reader).ber_reader() {
			Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
			Err(e) => Err(e),
			Ok(v)  => Ok(Some(v))
		}? {
			match tag {
				RESULT_REFERRAL if result.referral.is_none()  => result.referral = Some(
					BerSeqIter(|| reader.read_ber_str(BER_OCTET_STRING))
						.collect::<io::Result<Vec<_>>>()?),
				tag => return Err(unexp_tag_err::<Self>(tag))
			}
		}
		
		Ok(result)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResultCode {
	Success                      = 0,
	OperationsError              = 1,
	ProtocolError                = 2,
	TimeLimitExceeded            = 3,
	SizeLimitExceeded            = 4,
	CompareFalse                 = 5,
	CompareTrue                  = 6,
	AuthMethodNotSupported       = 7,
	StrongerAuthRequired         = 8,
	Referral                     = 10,
	AdminLimitExceeded           = 11,
	UnavailableCriticalExtension = 12,
	ConfidentialityRequired      = 13,
	SaslBindInProgress           = 14,
	NoSuchAttribute              = 16,
	UndefinedAttributeType       = 17,
	InappropriateMatching        = 18,
	ConstraintViolation          = 19,
	AttributeOrValueExists       = 20,
	InvalidAttributeSyntax       = 21,
	NoSuchObject                 = 32,
	AliasProblem                 = 33,
	InvalidDNSyntax              = 34,
	AliasDereferencingProblem    = 36,
	InappropriateAuthentication  = 48,
	InvalidCredentials           = 49,
	InsufficientAccessRights     = 50,
	Busy                         = 51,
	Unavailable                  = 52,
	UnwillingToPerform           = 53,
	LoopDetect                   = 54,
	NamingViolation              = 64,
	ObjectClassViolation         = 65,
	NotAllowedOnNonLeaf          = 66,
	NotAllowedOnRDN              = 67,
	EntryAlreadyExists           = 68,
	ObjectClassModsProhibited    = 69,
	AffectsMultipleDSAs          = 71,
	Other                        = 80
}

impl Default for ResultCode {
	fn default() -> Self {
		Self::Success
	}
}

impl ResultCode {
	pub fn from_int(v: usize) -> Option<Self> {
		match v {
			c @ 0..=8 | c @ 10..=21 | c @ 32..=34 | c @ 36 | c @ 48..=54 | c @ 64..=69 | c @ 71 | c @ 80 => Some(unsafe { std::mem::transmute(c as u8) }),
			_ => None
		}
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BindRequest {
	pub version:        u8,
	pub name:           String,
	pub authentication: AuthenticationChoice
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthenticationChoice {
	Simple(String),
	Sasl(SaslCredentials)
}

impl Default for AuthenticationChoice {
	fn default() -> Self {
		Self::Simple(String::new())
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaslCredentials {
	pub mechanism:   String,
	pub credentials: Option<String>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BindResponse {
	pub result:            LdapResult,
	pub server_sasl_creds: Option<String>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchRequest {
	pub base_object:   DN,
	pub scope:         SearchRequestScope,
	pub deref_aliases: SearchRequestDerefAliases,
	pub size_limit:    usize,
	pub time_limit:    usize,
	pub types_only:    bool,
	pub filter:        SearchRequestFilter,
	pub attributes:    Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchRequestScope {
	BaseObject,
	SingleLevel,
	WholeSubtree
}

impl Default for SearchRequestScope {
	fn default() -> Self {
		Self::BaseObject
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchRequestDerefAliases {
	NeverDerefAliases,
	DerefInSearching,
	DerefFindingBaseObj,
	DerefAlways
}

impl Default for SearchRequestDerefAliases {
	fn default() -> Self {
		Self::NeverDerefAliases
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchRequestFilter {
	And(Vec<Self>),
	Or(Vec<Self>),
	Not(Box<Self>),
	EqualityMatch(AttributeValueAssertion),
	GreaterOrEqual(AttributeValueAssertion),
	LessOrEqual(AttributeValueAssertion),
	Present(String),
	ApproxMatch(AttributeValueAssertion)
	// TODO ...
}

impl Default for SearchRequestFilter {
	fn default() -> Self {
		Self::Present("objectClass".to_string())
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchResultEntry {
	pub object_name: DN,
	pub attributes:  Vec<PartialAttribute>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchResultReference {
	pub uris: Vec<Uri>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModifyRequest {
	pub object:  DN,
	pub changes: Vec<Change>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Change {
	pub operation:    Operation,
	pub modification: PartialAttribute
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operation {
	Add       = 0,
	Delete    = 1,
	Replace   = 2,
	/// RFC4525
	Increment = 3
}

impl Default for Operation {
	fn default() -> Self {
		Self::Add
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AddRequest {
	pub entry: DN,
	pub attributes: Vec<Attribute>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModifyDNRequest {
	pub entry: DN,
	pub new_rdn:        String,
	pub delete_old_drn: bool,
	pub new_superior:   Option<DN>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompareRequest {
	pub entry: DN,
	pub ava:   AttributeValueAssertion
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AbandonRequest {
	pub message_id: MessageId
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtendedRequest {
	pub request_name:  Oid,
	pub request_value: Option<Box<[u8]>>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtendedResponse {
	pub result:         LdapResult,
	pub response_name:  Option<Oid>,
	pub response_value: Option<Box<[u8]>>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IntermediateResponse {
	pub response_name:  Option<Oid>,
	pub response_value: Option<Box<[u8]>>
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PartialAttribute {
	pub r#type: String,
	pub vals:   Vec<Box<[u8]>>
}

pub type Attribute = PartialAttribute;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AttributeValueAssertion {
	pub attribute_desc:  String,
	pub assertion_value: Box<[u8]>
}
