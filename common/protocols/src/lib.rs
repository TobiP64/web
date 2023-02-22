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

#![feature(
	linked_list_cursors,
	type_alias_impl_trait,
	associated_type_bounds
)]
#![warn(clippy::all)]
#![allow(
	unused_variables,
	dead_code,
	unconditional_recursion,
	clippy::result_unit_err,
	clippy::uninit_vec,
	clippy::type_complexity,
	clippy::redundant_closure,
	clippy::only_used_in_recursion
)]

pub mod dns;
pub mod grpc;
pub mod http;
pub mod ice;
pub mod ldap;
pub mod imf;
pub mod otlp;
pub mod quic;
pub mod rss;
pub mod rtp;
pub mod rtcp;
pub mod rtsp;
pub mod sasl;
pub mod sdp;
pub mod smtp;
pub mod stun;
pub mod tcp;
pub mod tls;
pub mod turn;
pub mod ws;
pub mod utils;
pub mod builder;
pub mod buffered;
//pub mod protobuf;

pub use webpki;

pub fn client() -> builder::ConnectorBuilder {
	builder::ConnectorBuilder
}

pub fn server() -> builder::AcceptorBuilder {
	builder::AcceptorBuilder
}

#[cfg(feature = "smol")]
pub fn client_async() -> builder::AsyncConnectorBuilder {
	builder::AsyncConnectorBuilder
}

#[cfg(feature = "smol")]
pub fn server_async() -> builder::AsyncAcceptorBuilder {
	builder::AsyncAcceptorBuilder
}
