
use {
	super::{wire::*, traits::*},
	std::io
};

#[derive(Clone, Debug, Default)]
pub struct ClientMessageBuilder(Message);

impl ClientMessageBuilder {
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn control(mut self, v: Control) -> Self {
		self.0.controls.get_or_insert_with(Default::default).push(v);
		self
	}
	
	pub fn controls(mut self, v: Vec<Control>) -> Self {
		self.0.controls = Some(v);
		self
	}
	
	pub fn bind_request(mut self) -> BindRequestBuilder {
		self.0.protocol_op = ProtocolOp::BindRequest(Default::default());
		BindRequestBuilder(self)
	}
	
	pub fn unbind_request(mut self) -> UnbindRequestBuilder {
		self.0.protocol_op = ProtocolOp::UnbindRequest;
		UnbindRequestBuilder(self)
	}
	
	pub fn search_request(mut self) -> SearchRequestBuilder {
		self.0.protocol_op = ProtocolOp::SearchRequest(Default::default());
		SearchRequestBuilder(self)
	}
	
	pub fn modify_request(mut self) -> ModifyRequestBuilder {
		self.0.protocol_op = ProtocolOp::ModifyRequest(Default::default());
		ModifyRequestBuilder(self)
	}
	
	pub fn add_request(mut self) -> AddRequestBuilder {
		self.0.protocol_op = ProtocolOp::AddRequest(Default::default());
		AddRequestBuilder(self)
	}
	
	pub fn del_request(mut self) -> DelRequestBuilder {
		self.0.protocol_op = ProtocolOp::DelRequest(Default::default());
		DelRequestBuilder(self)
	}
	
	pub fn modify_dn_request(mut self) -> ModifyDNRequestBuilder {
		self.0.protocol_op = ProtocolOp::ModifyDNRequest(Default::default());
		ModifyDNRequestBuilder(self)
	}
	
	pub fn compare_request(mut self) -> CompareRequestBuilder {
		self.0.protocol_op = ProtocolOp::CompareRequest(Default::default());
		CompareRequestBuilder(self)
	}
	
	pub fn abandon_request(mut self) -> AbandonRequestBuilder {
		self.0.protocol_op = ProtocolOp::AbandonRequest(Default::default());
		AbandonRequestBuilder(self)
	}
	
	pub fn extended_request(mut self) -> ExtendedRequestBuilder {
		self.0.protocol_op = ProtocolOp::ExtendedRequest(Default::default());
		ExtendedRequestBuilder(self)
	}
}

#[derive(Clone, Debug, Default)]
pub struct ServerMessageBuilder(Message);

impl ServerMessageBuilder {
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn message_id(mut self, v: MessageId) -> Self {
		self.0.message_id = v;
		self
	}
	
	pub fn control(mut self, v: Control) -> Self {
		self.0.controls.get_or_insert_with(Default::default).push(v);
		self
	}
	
	pub fn controls(mut self, v: Vec<Control>) -> Self {
		self.0.controls = Some(v);
		self
	}
	
	pub fn bind_response(mut self) -> BindResponseBuilder {
		self.0.protocol_op = ProtocolOp::BindResponse(Default::default());
		BindResponseBuilder(self)
	}
	
	pub fn search_result_entry(mut self) -> SearchResultEntryBuilder {
		self.0.protocol_op = ProtocolOp::SearchResultEntry(Default::default());
		SearchResultEntryBuilder(self)
	}
	
	pub fn search_result_done(mut self) -> SearchResultDoneBuilder {
		self.0.protocol_op = ProtocolOp::SearchResultDone(Default::default());
		SearchResultDoneBuilder(self)
	}
	
	pub fn search_result_reference(mut self) -> SearchResultReferenceBuilder {
		self.0.protocol_op = ProtocolOp::SearchResultReference(Default::default());
		SearchResultReferenceBuilder(self)
	}
	
	pub fn modify_response(mut self) -> ModifyResponseBuilder {
		self.0.protocol_op = ProtocolOp::ModifyResponse(Default::default());
		ModifyResponseBuilder(self)
	}
	
	pub fn add_response(mut self) -> AddResponseBuilder {
		self.0.protocol_op = ProtocolOp::AddResponse(Default::default());
		AddResponseBuilder(self)
	}
	
	pub fn del_response(mut self) -> DelResponseBuilder {
		self.0.protocol_op = ProtocolOp::DelResponse(Default::default());
		DelResponseBuilder(self)
	}
	
	pub fn modify_dn_response(mut self) -> ModifyDNResponseBuilder {
		self.0.protocol_op = ProtocolOp::ModifyDNResponse(Default::default());
		ModifyDNResponseBuilder(self)
	}
	
	pub fn compare_response(mut self) -> CompareResponseBuilder {
		self.0.protocol_op = ProtocolOp::CompareResponse(Default::default());
		CompareResponseBuilder(self)
	}
	
	pub fn extended_response(mut self) -> ExtendedResponseBuilder {
		self.0.protocol_op = ProtocolOp::ExtendedResponse(Default::default());
		ExtendedResponseBuilder(self)
	}
	
	pub fn intermediate_response(mut self) -> IntermediateResponseBuilder {
		self.0.protocol_op = ProtocolOp::IntermediateResponse(Default::default());
		IntermediateResponseBuilder(self)
	}
}

pub struct BindRequestBuilder(ClientMessageBuilder);

impl BindRequestBuilder {
	pub fn version(mut self, v: u8) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::BindRequest(r) => r.version = v,
			_ => unreachable!()
		}
		self
	}
	
	
	pub fn name(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::BindRequest(r) => r.name = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn auth_simple(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::BindRequest(r) => r.authentication = AuthenticationChoice::Simple(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn auth_sasl(self) -> Self {
		unimplemented!()
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &impl AsyncSharedClientConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &impl AsyncSharedClientConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct BindResponseBuilder(ServerMessageBuilder);

impl BindResponseBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct UnbindRequestBuilder(ClientMessageBuilder);

impl UnbindRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct SearchRequestBuilder(ClientMessageBuilder);

impl SearchRequestBuilder {
	pub fn base_object(mut self, v: DN) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.base_object = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn scope(mut self, v: SearchRequestScope) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.scope = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn deref_aliases(mut self, v: SearchRequestDerefAliases) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.deref_aliases = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn size_limit(mut self, v: usize) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.size_limit = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn time_limit(mut self, v: usize) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.time_limit = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn types_only(mut self, v: bool) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.types_only = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn filter(mut self, v: SearchRequestFilter) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.filter = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn attributes(mut self, v: Vec<String>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchRequest(r) => r.attributes = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<LdapResult> {
		todo!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub fn send_recv_async(&self, stream: &impl AsyncSharedClientConnection) -> impl futures_lite::Stream<Item = io::Result<SearchResult>> {
		todo!();
		futures_lite::stream::empty()
	}
}

pub struct SearchResultEntryBuilder(ServerMessageBuilder);

impl SearchResultEntryBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct SearchResultDoneBuilder(ServerMessageBuilder);

impl SearchResultDoneBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchResultDone(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchResultDone(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchResultDone(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchResultDone(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::SearchResultDone(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct SearchResultReferenceBuilder(ServerMessageBuilder);

impl SearchResultReferenceBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct ModifyRequestBuilder(ClientMessageBuilder);

impl ModifyRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct ModifyResponseBuilder(ServerMessageBuilder);

impl ModifyResponseBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyResponse(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyResponse(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyResponse(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyResponse(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyResponse(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct AddRequestBuilder(ClientMessageBuilder);

impl AddRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct AddResponseBuilder(ServerMessageBuilder);

impl AddResponseBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::AddResponse(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::AddResponse(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::AddResponse(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::AddResponse(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::AddResponse(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct DelRequestBuilder(ClientMessageBuilder);

impl DelRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct DelResponseBuilder(ServerMessageBuilder);

impl DelResponseBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::DelResponse(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::DelResponse(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::DelResponse(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::DelResponse(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::DelResponse(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct ModifyDNRequestBuilder(ClientMessageBuilder);

impl ModifyDNRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct ModifyDNResponseBuilder(ServerMessageBuilder);

impl ModifyDNResponseBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyDNResponse(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyDNResponse(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyDNResponse(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyDNResponse(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::ModifyDNResponse(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct CompareRequestBuilder(ClientMessageBuilder);

impl CompareRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct CompareResponseBuilder(ServerMessageBuilder);

impl CompareResponseBuilder {
	pub fn result_code(mut self, v: ResultCode) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::CompareResponse(r) => r.result_code = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn matched_dn(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::CompareResponse(r) => r.matched_dn = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn diagnostic_message(mut self, v: String) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::CompareResponse(r) => r.diagnostic_message = v,
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral_uri(mut self, v: Uri) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::CompareResponse(r) => r.referral.get_or_insert_with(Default::default).push(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn referral(mut self, v: Vec<Uri>) -> Self {
		match &mut self.0.0.protocol_op {
			ProtocolOp::CompareResponse(r) => r.referral = Some(v),
			_ => unreachable!()
		}
		self
	}
	
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct AbandonRequestBuilder(ClientMessageBuilder);

impl AbandonRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct ExtendedRequestBuilder(ClientMessageBuilder);

impl ExtendedRequestBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub fn send_recv(&self, stream: &mut impl Connection) -> io::Result<BindResponse> {
		unimplemented!()
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
	
	pub async fn send_recv_async(&self, stream: &mut impl AsyncConnection) -> io::Result<BindResponse> {
		unimplemented!()
	}
}

pub struct ExtendedResponseBuilder(ServerMessageBuilder);

impl ExtendedResponseBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}

pub struct IntermediateResponseBuilder(ServerMessageBuilder);

impl IntermediateResponseBuilder {
	pub fn send(&self, stream: &mut impl Connection) -> io::Result<()> {
		stream.send_msg(&self.0.0)
	}
	
	pub async fn send_async(&self, stream: &mut impl AsyncConnection) -> io::Result<()> {
		AsyncConnectionExt::send_msg(stream, &self.0.0).await
	}
}
