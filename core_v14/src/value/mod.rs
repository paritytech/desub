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
use serde::{Deserialize};
use std::convert::From;
use std::fmt::Debug;

/// [`Value`] holds a representation of some value that has been decoded.
///
/// Not all SCALE encoded types have an similar-named value; for instance, the values corresponding to
/// sequence, array and composite types can all be represented with [`Composite`]. Only enough information
/// is preserved here to construct a valid value for any type that we know about, and be able to verify
/// that a given value is compatible with some type (see the [`scale_info`] crate), if we have both.
#[derive(Clone, PartialEq)]
pub enum Value {
	/// A named or unnamed struct-like, array-like or tuple-like set of values.
	Composite(Composite),
	/// An enum variant.
	Variant(Variant),
	/// A sequence of bits (which is more compactly encoded using [`bitvec`])
	BitSequence(BitSequence),
	/// Any of the primitive values we can have.
	Primitive(Primitive),
}

impl Debug for Value {
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
pub enum Composite {
	/// Eg `{ foo: 2, bar: false }`
	Named(Vec<(String, Value)>),
	/// Eg `(2, false)`
	Unnamed(Vec<Value>),
}

impl Composite {
	fn len(&self) -> usize {
		match self {
			Composite::Named(values) => values.len(),
			Composite::Unnamed(values) => values.len(),
		}
	}
}

impl Debug for Composite {
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

impl From<Composite> for Value {
	fn from(val: Composite) -> Self {
		Value::Composite(val)
	}
}

/// This represents the value of a specific variant from an enum, and contains
/// the name of the variant, and the named/unnamed values associated with it.
#[derive(Clone, PartialEq)]
pub struct Variant {
	/// The name of the variant.
	pub name: String,
	/// Values for each of the named or unnamed fields associated with this variant.
	pub values: Composite,
}

impl Debug for Variant {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.name)?;
		f.write_str(" ")?;
		Debug::fmt(&self.values, f)
	}
}

impl From<Variant> for Value {
	fn from(val: Variant) -> Self {
		Value::Variant(val)
	}
}

/// A "primitive" value (this includes strings).
#[derive(Clone, PartialEq)]
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

impl Debug for Primitive {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Primitive::Bool(val) => Debug::fmt(val, f),
			Primitive::Char(val) => Debug::fmt(val, f),
			Primitive::Str(val) => Debug::fmt(val, f),
			Primitive::U8(val) => Debug::fmt(val, f),
			Primitive::U16(val) => Debug::fmt(val, f),
			Primitive::U32(val) => Debug::fmt(val, f),
			Primitive::U64(val) => Debug::fmt(val, f),
			Primitive::U128(val) => Debug::fmt(val, f),
			Primitive::I8(val) => Debug::fmt(val, f),
			Primitive::I16(val) => Debug::fmt(val, f),
			Primitive::I32(val) => Debug::fmt(val, f),
			Primitive::I64(val) => Debug::fmt(val, f),
			Primitive::I128(val) => Debug::fmt(val, f),
			Primitive::U256(val) | Primitive::I256(val) => {
				f.write_str("BigNum(")?;
				Debug::fmt(val, f)?;
				f.write_str(")")
			}
		}
	}
}

impl From<Primitive> for Value {
	fn from(val: Primitive) -> Self {
		Value::Primitive(val)
	}
}

/// A sequence of bits.
pub type BitSequence = BitVec<Lsb0, u8>;

/// An opaque error that is returned if we cannot deserialize the [`Value`] type.
pub use deserializer::Error as DeserializeError;

/// Attempt to deserialize a [`Value`] into some type that has [`serde::Deserialize`] implemented on it.
pub fn from_value<'de, T: Deserialize<'de>>(value: Value) -> Result<T, DeserializeError> {
	T::deserialize(value)
}
