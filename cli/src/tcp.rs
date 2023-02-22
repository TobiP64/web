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

use {super::*, std::net::TcpStream};

pub fn connect(opts: Options) -> Result<TcpStream, ()> {
	let mut stream = match opts.tcp_udp_conn_timeout {
		None    => TcpStream::connect(opts.conn_peer),
		Some(t) => TcpStream::connect_timeout(opts.conn_peer, t)
	}?;
	
	if let Some(v) = opts.tcp_udp_ttl {
		stream.set_ttl(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_nodelay {
		stream.set_nodelay(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_keep_alive {
		stream.set_keepalive(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_read_timeout {
		stream.set_read_timeout(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_write_timeout {
		stream.set_write_timeout(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_recv_buffer {
		stream.set_recv_buffer_size(v)?;
	}
	
	if let Some(v) = opts.tcp_udp_send_buffer {
		stream.set_send_buffer_size(v)?;
	}
	
	Ok(stream)
}

pub fn run(opts: Options) {
	let mut stream = connect(opts)?;
	
	let recv_thread = opts.io_output.take().map(|dst| {
		let stream = stream.try_clone().expect("failed to clone TCP stream");
		std::thread::spawn(move || {
			let mut buf = Vec::with_capacity(0x1000);
			unsafe { buf.set_len(0x1000); }
			
			loop {
				let len = stream.read(&mut buf);
				
				if len == 0 {
					return Ok(());
				}
				
				dst.write_all(buf[..len])?;
			}
		})
	});
	
	let send_thread = opts.io_output.take().map(|src| {
		let stream = stream.try_clone().expect("failed to clone TCP stream");
		std::thread::spawn(move || {
			let mut buf = Vec::with_capacity(0x1000);
			unsafe { buf.set_len(0x1000); }
			
			loop {
				let len = src.read(&mut buf);
				
				if len == 0 {
					return Ok(());
				}
				
				stream.write_all(buf[..len])?;
			}
		})
	});
	
	if let Some(t) = recv_thread {
		t.join().expect("failed to join recv thread")?;
	}
	
	if let Some(t) = send_thread {
		t.join().expect("failed to join send thread")?;
	}
}
