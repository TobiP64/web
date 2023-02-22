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

#![warn(clippy::all)]
#![forbid(unsafe_code)]

use {std::{fmt, collections::HashMap}, serde::{*, ser::*, de::*}};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
	Unit,
	Bool(bool),
	Char(char),
	UInt(usize),
	SInt(isize),
	F32(f32),
	F64(f64),
	String(String),
	Seq(Vec<Self>),
	Map(HashMap<String, Self>),
	Enum(u32, Box<Self>),
	Bytes(Vec<u8>)
}

impl Value {
	pub fn serialize_from(v: impl Serialize) -> Result<Self, Error> {
		v.serialize(SerType)
	}
	
	pub fn deserialize_to<'de, T: Deserialize<'de>>(self) -> Result<T, Error> {
		T::deserialize(self)
	}
}

impl Default for Value {
	fn default() -> Self {
		Self::Unit
	}
}

pub struct SerType;

impl Serializer for SerType {
	type Ok                     = Value;
	type Error                  = Error;
	type SerializeSeq           = SerSeq;
	type SerializeTuple         = SerSeq;
	type SerializeTupleStruct   = SerSeq;
	type SerializeTupleVariant  = SerEnum<SerSeq>;
	type SerializeMap           = SerMap;
	type SerializeStruct        = SerMap;
	type SerializeStructVariant = SerEnum<SerMap>;
	
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bool(v as _))
	}
	
	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::SInt(v as _))
	}
	
	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::SInt(v as _))
	}
	
	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::SInt(v as _))
	}
	
	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::SInt(v as _))
	}
	
	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::UInt(v as _))
	}
	
	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::UInt(v as _))
	}
	
	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::UInt(v as _))
	}
	
	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::UInt(v as _))
	}
	
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::F32(v))
	}
	
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::F64(v))
	}
	
	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Char(v))
	}
	
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(v.to_string()))
	}
	
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bytes(v.to_vec()))
	}
	
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Unit)
	}
	
	fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
		value.serialize(self)
	}
	
	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Unit)
	}
	
	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}
	
	fn serialize_unit_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Enum(variant_index, Box::new(Value::Unit)))
	}
	
	fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error> {
		value.serialize(self)
	}
	
	fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _name: &'static str, variant_index: u32, _variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Enum(variant_index, Box::new(value.serialize(self)?)))
	}
	
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Ok(SerSeq(Vec::with_capacity(len.unwrap_or(0))))
	}
	
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		self.serialize_seq(Some(len))
	}
	
	fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
		self.serialize_seq(Some(len))
	}
	
	fn serialize_tuple_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
		Ok(SerEnum(variant_index, self.serialize_seq(Some(len))?))
	}
	
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(SerMap(HashMap::with_capacity(len.unwrap_or(0)), None))
	}
	
	fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
		self.serialize_map(Some(len))
	}
	
	fn serialize_struct_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
		Ok(SerEnum(variant_index, self.serialize_map(Some(len))?))
	}
	
	fn is_human_readable(&self) -> bool {
		false
	}
}

pub struct SerSeq(pub Vec<Value>);

impl SerializeSeq for SerSeq {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
		self.0.push(value.serialize(SerType)?);
		Ok(())
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Seq(self.0))
	}
}

impl SerializeTuple for SerSeq {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
		SerializeSeq::serialize_element(self, value)
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		SerializeSeq::end(self)
	}
}

impl SerializeTupleStruct for SerSeq {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
		SerializeSeq::serialize_element(self, value)
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		SerializeSeq::end(self)
	}
}

pub struct SerMap(pub HashMap<String, Value>, pub Option<String>);

impl SerializeMap for SerMap {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
		use serde::ser::Error;
		match key.serialize(SerType) {
			Ok(Value::String(key)) => {
				self.1 = Some(key);
				Ok(())
			},
			_ => Err(Error::custom("failed to serialize key"))
		}
	}
	
	fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
		use serde::ser::Error;
		match self.1.take() {
			Some(key) =>  {
				self.0.insert(key, value.serialize(SerType)?);
				Ok(())
			},
			None => Err(Error::custom("no key serialized"))
		}
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(self.0))
	}
}

impl SerializeStruct for SerMap {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
		SerializeMap::serialize_entry(self, key, value)
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		SerializeMap::end(self)
	}
}

pub struct SerEnum<T>(pub u32, pub T);

impl SerializeTupleVariant for SerEnum<SerSeq> {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
		SerializeSeq::serialize_element(&mut self.1, value)
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		let Self(i, v) = self;
		SerializeSeq::end(v)
			.map(|v| Value::Enum(i, Box::new(v)))
	}
}

impl SerializeStructVariant for SerEnum<SerMap> {
	type Ok    = Value;
	type Error = Error;
	
	fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
		SerializeMap::serialize_entry(&mut self.1, key, value)
	}
	
	fn end(self) -> Result<Self::Ok, Self::Error> {
		let Self(i, v) = self;
		SerializeMap::end(v)
			.map(|v| Value::Enum(i, Box::new(v)))
	}
}

impl<'de> Deserializer<'de> for Value {
	type Error = Error;
	
	fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
		match self {
			Self::Unit       => visitor.visit_unit(),
			Self::Bool(v)    => visitor.visit_bool(v),
			Self::Char(v)    => visitor.visit_char(v),
			Self::SInt(v)    => visitor.visit_i64(v as _),
			Self::UInt(v)    => visitor.visit_u64(v as _),
			Self::F32(v)     => visitor.visit_f32(v),
			Self::F64(v)     => visitor.visit_f64(v),
			Self::String(v)  => visitor.visit_string(v),
			Self::Bytes(v)   => visitor.visit_byte_buf(v),
			Self::Seq(v)     => visitor.visit_seq(DeSeq(v.into_iter())),
			Self::Map(v)     => visitor.visit_map(DeMap(v.into_iter(), None)),
			Self::Enum(i, v) => visitor.visit_enum(DeEnum(i, *v))
		}
	}
	
	forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
	
	fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error> where
		V: Visitor<'de> {
		match self {
			Self::Unit => visitor.visit_none(),
			v => visitor.visit_some(v)
		}
	}
}

pub struct DeSeq<I: Iterator<Item = Value>>(pub I);

impl<'de, I: Iterator<Item =Value>> SeqAccess<'de> for DeSeq<I> {
	type Error = Error;
	
	fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
		self.0.next()
			.map(|v| seed.deserialize(v))
			.transpose()
	}
}

pub struct DeMap<I: Iterator<Item = (String, Value)>>(pub I, pub Option<Value>);

impl<'de, I: Iterator<Item = (String, Value)>> MapAccess<'de> for DeMap<I> {
	type Error = Error;
	
	fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
		self.0.next()
			.map(|(k, v)| {
				self.1 = Some(v);
				seed.deserialize(Value::String(k))
			})
			.transpose()
	}
	
	fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
		seed.deserialize(self.1.take().expect("invalid state"))
	}
}

pub struct DeEnum(pub u32, pub Value);

impl<'de> EnumAccess<'de> for DeEnum {
	type Error   = Error;
	type Variant = Self;
	
	fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
		Ok((seed.deserialize(Value::UInt(self.0 as _))?, self))
	}
}

impl<'de> VariantAccess<'de> for DeEnum {
	type Error = Error;
	
	fn unit_variant(self) -> Result<(), Self::Error> {
		use serde::de::Error;
		match self.1 {
			Value::Unit => Ok(()),
			v => Err(Error::custom(format!("expected unit variant, got {:?}", v)))
		}
	}
	
	fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
		seed.deserialize(self.1)
	}
	
	fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error> {
		self.1.deserialize_any(visitor)
	}
	
	fn struct_variant<V: Visitor<'de>>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
		self.1.deserialize_any(visitor)
	}
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Error(String);

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		Self(msg.to_string())
	}
}

impl serde::de::Error for Error {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		Self(msg.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
	struct Data {
		int:    isize,
		float:  f32,
		string: String,
		seq:    Vec<usize>
	}
	
	fn dyn_data() -> Value {
		Value::Map([
			("int".to_string(), Value::SInt(-123)),
			("float".to_string(), Value::F32(3.21)),
			("string".to_string(), Value::String("test".to_string())),
			("seq".to_string(), Value::Seq(vec![Value::UInt(1), Value::UInt(2), Value::UInt(3)]))
		].iter().cloned().collect())
	}
	
	fn data() -> Data {
		Data {
			int:    -123,
			float:  3.21,
			string: "test".to_string(),
			seq:    vec![1, 2, 3]
		}
	}
	
	#[test]
	fn serialize() {
		assert_eq!(data().serialize(SerType), Ok(dyn_data()));
	}
	
	#[test]
	fn serialize_self() {
		assert_eq!(dyn_data().serialize(SerType), Ok(dyn_data()));
	}
	
	#[test]
	fn deserialize() {
		assert_eq!(Data::deserialize(dyn_data()), Ok(data()));
	}
	
	#[test]
	fn deserialize_self() {
		assert_eq!(Value::deserialize(dyn_data()), Ok(dyn_data()));
	}
}