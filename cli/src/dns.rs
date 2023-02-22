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

use std::net::ToSocketAddrs;
use super::*;

pub fn resolve(host: &str, opts: &Options) -> Result<Vec<std::net::IpAddr>, ()> {
	if let Ok(addr) = host.parse::<std::net::IpAddr>() {
		return Ok(vec![addr].into_iter());
	}
	
	if opts.conn_dns.is_empty() {
		log::trace!("[RESOLVE] resolving `{}` via the system resolver", host);
		return (host, 0).to_socket_addrs()?.map(|v| v.ip()).collect::<Vec<_>>();
	}
	
	for server in &opts.conn_dns {
	
	}
	
	unimplemented!()
}

pub fn run(opts: Options) {
	let (domain, qtype, qclass) = match opts.params {
		Some(Parameters::Dns { domain, qtype, qclass }) => (domain, qtype, qclass),
		_ => unreachable!()
	};
	
	
}