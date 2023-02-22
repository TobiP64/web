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

use std::{mem::{replace, zeroed}, ops::{Index, IndexMut}};

pub struct RingBuffer<T> {
	data:   Box<[T]>,
	offset: usize,
	length: usize,
}

impl<T> RingBuffer<T> {
	#[inline]
	pub fn new(capacity: usize) -> Self where T: Clone {
		Self {
			data:   vec![unsafe { zeroed() }; capacity].into_boxed_slice(),
			offset: 0,
			length: 0
		}
	}
	
	#[inline]
	pub fn len(&self) -> usize {
		self.length
	}
	
	#[inline]
	pub fn is_empty(&self) -> bool { self.length == 0 }
	
	#[inline]
	pub fn capacity(&self) -> usize {
		self.data.len()
	}
	
	pub fn push(&mut self, value: T) -> Option<T> {
		if self.offset < self.data.len() {
			let value = replace(&mut self.data[self.offset], value);
			self.offset += 1;
			if self.length < self.data.len() {
				self.length += 1;
				None
			} else {
				Some(value)
			}
		} else {
			let value = replace(&mut self.data[0], value);
			self.offset = 1;
			Some(value)
		}
	}
	
	pub fn pop(&mut self) -> Option<T> {
		if self.length == 0 {
			None
		} else {
			let __len = self.length - 1;
			Some(replace(&mut self[__len], unsafe { zeroed() }))
		}
	}
	
	pub fn pop_first(&mut self) -> Option<T> {
		if self.length == 0 {
			None
		} else if self.offset > 0 {
			self.offset -= 1;
			self.length -= 1;
			Some(replace(&mut self[0], unsafe { zeroed() }))
		} else {
			self.offset = self.data.len();
			self.length -= 1;
			Some(replace(&mut self[0], unsafe { zeroed() }))
		}
	}
	
	#[inline]
	pub fn clear(&mut self) {
		self.offset = 0;
		self.length = 0;
	}
	
	#[inline]
	pub fn iter(&self) -> Iter<T> {
		self.into_iter()
	}
	
	/*#[inline]
	pub fn iter_mut(&self) -> IterMut<T> {
		self.into_iter()
	}*/
}

impl<T> Index<usize> for RingBuffer<T> {
	type Output = T;
	
	fn index(&self, index: usize) -> &Self::Output {
		if index >= self.length { panic!("index out of bounds") }
		
		&self.data[if index > self.offset {
			self.data.len() - index - self.offset
		} else {
			self.offset - index - 1
		}]
	}
}

impl<T> IndexMut<usize> for RingBuffer<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index >= self.length { panic!("index out of bounds") }
		
		&mut self.data[if index > self.offset {
			self.data.len() - index - self.offset
		} else {
			self.offset - index
		}]
	}
}

impl<'a, T> IntoIterator for &'a RingBuffer<T> {
	type Item = &'a T;
	type IntoIter = Iter<'a, T>;
	
	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		Iter {
			inner: self,
			index: 0
		}
	}
}

pub struct Iter<'a, T> {
	inner: &'a RingBuffer<T>,
	index: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.inner.length {
			None
		} else {
			self.index += 1;
			Some(&self.inner[self.index - 1])
		}
	}
}

/*impl<'a, T> IntoIterator for &'a mut RingBuffer<T> {
	type Item = &'a mut T;
	type IntoIter = IterMut<'a, T>;
	
	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		IterMut {
			inner: self,
			index: 0
		}
	}
}

pub struct IterMut<'a, T> {
	inner: &'a mut RingBuffer<T>,
	index: usize
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = &'a mut T;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.inner.length {
			None
		} else {
			self.index += 1;
			Some(&mut self.inner[self.index - 1])
		}
	}
}*/