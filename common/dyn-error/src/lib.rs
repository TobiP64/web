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

#![no_std]
#![warn(clippy::all)]
//#![forbid(unsafe_code)]

extern crate alloc;

use {
	core::{any::Any, fmt::{self, Display, Debug, Formatter}},
	alloc::{boxed::Box, string::ToString}
};

pub type Result<T> = core::result::Result<T, Error>;

pub struct Error(pub Box<dyn AnyDisplay>);

pub trait AnyDisplay: Any + Display + Send + Sync {}

impl<T: Any + Display + Send + Sync> AnyDisplay for T {}

impl Error {
	pub fn new(v: impl AnyDisplay) -> Self {
		Self(Box::new(v))
	}
	
	pub fn new_string(v: impl ToString) -> Self {
		Self::new(v.to_string())
	}
	
	pub fn new_debug(v: impl Debug) -> Self {
		Self::new_string(alloc::format!("{:#?}", v))
	}
	
	pub fn is<T: Any>(&self) -> bool {
		core::any::TypeId::of::<T>() == self.0.type_id()
	}
	
	pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
		if self.is::<T>() {
			unsafe { Some(&*(self as *const dyn Any as *const T)) }
		} else {
			None
		}
	}
	
	pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
		if self.is::<T>() {
			unsafe { Some(&mut *(self as *mut dyn Any as *mut T)) }
		} else {
			None
		}
	}
}

impl Debug for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

/*impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}*/

impl Error {
	pub fn display(self) -> ErrorDisplay {
		ErrorDisplay(self)
	}
}

pub struct ErrorDisplay(pub Error);

impl Display for ErrorDisplay {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl<T: AnyDisplay> From<T> for Error {
	fn from(v: T) -> Self {
		Error::new(v)
	}
}