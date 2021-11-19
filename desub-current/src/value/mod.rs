// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

/*!
This module exposes the [`Value`] type and related subtypes, which are used as the runtime
representations of SCALE encoded data (much like `serde_json::Value` is a runtime representation
of JSON data).
*/

mod deserialize;
mod deserializer;
mod serialize;

use bitvec::{order::Lsb0, vec::BitVec};
use serde::Deserialize;
use std::convert::From;
use std::fmt::Debug;

/// [`Value`] holds a representation of some value that has been decoded, as well as some arbitrary context.
///
/// Not all SCALE encoded types have an similar-named value; for instance, the values corresponding to
/// sequence, array and composite types can all be represented with [`Composite`]. Only enough information
/// is preserved here to construct a valid value for any type that we know about, and be able to verify
/// that a given value is compatible with some type (see the [`scale_info`] crate), if we have both.
#[derive(Debug, Clone, PartialEq)]
pub struct Value<T> {
	/// The shape and associated values for this Value
	pub value: ValueDef<T>,
	/// Some additional arbitrary context that can be associated with a value.
	pub context: T
}

impl Value<()> {
	/// Create a new value without any context.
	pub fn new(value: ValueDef<()>) -> Value<()> {
		Value { value, context: () }
	}
}

impl <T> Value<T> {
	/// Create a new value with some associated context.
	pub fn with_context(value: ValueDef<T>, context: T) -> Value<T> {
		Value { value, context }
	}
	/// Remove the context.
	pub fn without_context(self) -> Value<()> {
		Value { value: self.value.without_context(), context: () }
	}
}

/// The underlying shape of a given value.
#[derive(Clone, PartialEq)]
pub enum ValueDef<T> {
	/// A named or unnamed struct-like, array-like or tuple-like set of values.
	Composite(Composite<T>),
	/// An enum variant.
	Variant(Variant<T>),
	/// A sequence of bits (which is more compactly encoded using [`bitvec`])
	BitSequence(BitSequence),
	/// Any of the primitive values we can have.
	Primitive(Primitive),
}

impl <T> ValueDef<T> {
	/// Remove the context.
	pub fn without_context(self) -> ValueDef<()> {
		match self {
			ValueDef::Composite(val) => ValueDef::Composite(val.without_context()),
			ValueDef::Variant(val) => ValueDef::Variant(val.without_context()),
			ValueDef::BitSequence(val) => ValueDef::BitSequence(val),
			ValueDef::Primitive(val) => ValueDef::Primitive(val),
		}
	}
}

impl <T: Debug> Debug for ValueDef<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Composite(val) => Debug::fmt(val, f),
			Self::Variant(val) => Debug::fmt(val, f),
			Self::Primitive(val) => Debug::fmt(val, f),
			Self::BitSequence(val) => Debug::fmt(val, f),
		}
	}
}

/// A named or unnamed struct-like, array-like or tuple-like set of values.
/// This is used to represent a range of composite values on their own, or
/// as values for a specific [`Variant`].
#[derive(Clone, PartialEq)]
pub enum Composite<T> {
	/// Eg `{ foo: 2, bar: false }`
	Named(Vec<(String, Value<T>)>),
	/// Eg `(2, false)`
	Unnamed(Vec<Value<T>>),
}

impl <T> Composite<T> {
	/// Return the number of values stored in this composite type.
	pub fn len(&self) -> usize {
		match self {
			Composite::Named(values) => values.len(),
			Composite::Unnamed(values) => values.len(),
		}
	}

	/// Remove the context.
	pub fn without_context(self) -> Composite<()> {
		match self {
			Composite::Named(values) => {
				let vals = values
					.into_iter()
					.map(|(k,v)| (k, v.without_context()))
					.collect();
				Composite::Named(vals)
			},
			Composite::Unnamed(values) => {
				let vals = values
					.into_iter()
					.map(|v| v.without_context())
					.collect();
				Composite::Unnamed(vals)
			},
		}
	}
}

impl <T: Debug> Debug for Composite<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Composite::Named(fields) => {
				let mut struc = f.debug_struct("");
				for (name, val) in fields {
					struc.field(name, val);
				}
				struc.finish()
			}
			Composite::Unnamed(fields) => {
				let mut struc = f.debug_tuple("");
				for val in fields {
					struc.field(val);
				}
				struc.finish()
			}
		}
	}
}

impl <T> From<Composite<T>> for ValueDef<T> {
	fn from(val: Composite<T>) -> Self {
		ValueDef::Composite(val)
	}
}

/// This represents the value of a specific variant from an enum, and contains
/// the name of the variant, and the named/unnamed values associated with it.
#[derive(Clone, PartialEq)]
pub struct Variant<T> {
	/// The name of the variant.
	pub name: String,
	/// Values for each of the named or unnamed fields associated with this variant.
	pub values: Composite<T>,
}

impl <T> Variant<T> {
	/// Remove the context.
	pub fn without_context(self) -> Variant<()> {
		Variant {
			name: self.name,
			values: self.values.without_context()
		}
	}
}

impl <T: Debug> Debug for Variant<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.name)?;
		f.write_str(" ")?;
		Debug::fmt(&self.values, f)
	}
}

impl <T> From<Variant<T>> for ValueDef<T> {
	fn from(val: Variant<T>) -> Self {
		ValueDef::Variant(val)
	}
}

/// A "primitive" value (this includes strings).
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
	Bool(bool),
	Char(char),
	Str(String),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	U128(u128),
	U256([u8; 32]),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
	I256([u8; 32]),
}

impl <T> From<Primitive> for ValueDef<T> {
	fn from(val: Primitive) -> Self {
		ValueDef::Primitive(val)
	}
}

/// A sequence of bits.
pub type BitSequence = BitVec<Lsb0, u8>;

/// An opaque error that is returned if we cannot deserialize the [`Value`] type.
pub use deserializer::Error as DeserializeError;

/// Attempt to deserialize a [`Value`] into some type that has [`serde::Deserialize`] implemented on it.
pub fn from_value<'de, Ctx, T: Deserialize<'de>>(value: Value<Ctx>) -> Result<T, DeserializeError> {
	T::deserialize(value)
}
