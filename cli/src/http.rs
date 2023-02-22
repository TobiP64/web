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

use {super::*, std::net::TcpStream, net::http::{self, Connection, impls::Stream}};

static USER_AGENT: &str = env!("CARGO_PKG_NAME");

pub fn run(opts: Options) -> Option<Vec<u8>> {
	let (method) = match opts.params {
		Some(Parameters::Http { method }) => method,
		_ => unreachable!()
	};
	
	let mut headers = vec![
		http::Header::Method(method),
		http::Header::Path(opts.url),
		http::Header::UserAgent(USER_AGENT.to_string())
	];
	
	if let Some(data) = &opts.io_data {
		headers.push(http::Header::ContentLength(data.len()));
	}
	
	let tcp = TcpStream::connect(ip_addr).unwrap_or_else(
		|e| exit!("failed to connect to {}: {}", opts.conn_peer, e));
	eprintln!("[TRAFFIC] TCP: established connection to {}", opts.conn_peer);
	let mut conn = http::v1::Connection::new(tcp);
	let mut stream = conn.stream().unwrap_or_else(
		|e| exit!("failed to open HTTP stream: {}", e));
	eprintln!("[TRAFFIC] HTTP: opened stream");
	
	if opts.log_verbose {
		for header in &headers {
			eprintln!("[TRAFFIC] > {}", header);
		}
	}
	
	stream.write_headers(&headers)?;
	
	if let Some(data) = &opts.io_data {
		if opts.log_verbose {
			eprintln!("[TRAFFIC] > binary data ({} bytes)", data.len());
		}
		
		stream.write_all(data)?;
	}
	
	headers.clear();
	stream.read_all_headers(&mut headers)?;
	
	if opts.log_verbose {
		for header in &headers {
			eprintln!("[TRAFFIC] < {}", header);
		}
	}
	
	let data = match headers.iter().find_map(http::Header::as_content_length) {
		Some(len) => {
			let mut data = Vec::with_capacity(len);
			unsafe { data.set_len(len); }
			stream.read_exact(&data)?;
			data
		}
		None => {
			let mut data = Vec::with_capacity(1024);
			stream.read_to_end(&mut data)?;
			data
		}
	};
	
	eprintln!("[TRAFFIC] < binary data ({} bytes)", data.len());
	std::mem::drop(stream);
	eprintln!("[TRAFFIC] closed stream");
	std::mem::drop(conn);
	eprintln!("[TRAFFIC] closed connection to {}", opts.conn_peer);
}