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

use {
	std::io,
	serde::*
};

pub struct Serializer<T: io::Write>(T);

impl<T: io::Write> serde::Serializer for Serializer<T> {
	type Ok = ();
	type Error = ();
	type SerializeSeq = ();
	type SerializeTuple = ();
	type SerializeTupleStruct = ();
	type SerializeTupleVariant = ();
	type SerializeMap = ();
	type SerializeStruct = ();
	type SerializeStructVariant = ();
	
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_some<U: ?Sized>(self, value: &U) -> Result<Self::Ok, Self::Error> where T: Serialize {
		todo!()
	}
	
	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
		todo!()
	}
	
	fn serialize_newtype_struct<U: ?Sized>(self, name: &'static str, value: &U) -> Result<Self::Ok, Self::Error> where T: Serialize {
		todo!()
	}
	
	fn serialize_newtype_variant<U: ?Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &U) -> Result<Self::Ok, Self::Error> where T: Serialize {
		todo!()
	}
	
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		todo!()
	}
	
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		todo!()
	}
	
	fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
		todo!()
	}
	
	fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
		todo!()
	}
	
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		todo!()
	}
	
	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
		todo!()
	}
	
	fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
		todo!()
	}
}