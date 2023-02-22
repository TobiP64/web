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

use {std::{io::*, sync::{Arc, Mutex}, collections::VecDeque}};

const BUF_LEN: usize = 0x1000;

pub struct Pipe {
	read:  Arc<Mutex<VecDeque<u8>>>,
	write: Arc<Mutex<VecDeque<u8>>>
}

impl Pipe {
	pub fn new() -> (Self, Self) {
		let buffers = (
			Arc::new(Mutex::new(VecDeque::with_capacity(BUF_LEN))),
			Arc::new(Mutex::new(VecDeque::with_capacity(BUF_LEN)))
		);
		
		(
			Self { read: buffers.0.clone(), write: buffers.1.clone() },
			Self { read: buffers.1, write: buffers.0 }
		)
	}
	
	pub fn new_buffered() -> (BufReader<Self>, BufReader<Self>) {
		let pipe = Self::new();
		(BufReader::new(pipe.0), BufReader::new(pipe.1))
	}
}

impl Read for Pipe {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let mut read = self.read.lock().unwrap();
		let slices = read.as_mut_slices();
		let slice = if slices.0.is_empty() { slices.1 } else { slices.0 };
		let len = slice.len().min(buf.len());
		buf[..len].copy_from_slice(&slice[..len]);
		read.drain(0..len);
		Ok(len)
	}
}

impl Write for Pipe {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.write.lock().unwrap().extend(buf.iter().copied());
		Ok(buf.len())
	}
	
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}