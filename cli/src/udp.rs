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

use {super::*, std::net::UdpSocket};

pub fn connect(opts: Options) -> Result<UdpSocket, ()> {
	let socket = UdpSocket::bind(opts.conn_socket)?;
	socket.connect(opts.conn_peer)?;
	
	if let Some(v) = opts.tcp_udp_ttl {
		socket.set_ttl(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_read_timeout {
		socket.set_read_timeout(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_write_timeout {
		socket.set_write_timeout(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_recv_buffer {
		socket.set_recv_buffer_size(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_send_buffer {
		socket.set_send_buffer_size(v)?;
	}
	
	Ok(stream)
}

pub fn run(opts: Options) {
	let mut stream = connect(opts);
}
