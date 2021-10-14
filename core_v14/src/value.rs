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

use bitvec::{order::Lsb0, vec::BitVec};
use std::convert::From;
use std::fmt::Debug;

/// Whereas [`crate::substrate_type::SubstrateType`] is concerned with type information,
/// [`SubstrateValue`] is concerned with holding a representation of actual values
/// corresponding to each of those types.
///
/// Not all types have an similar-named value; for example, sequences and array
/// values can both be represented with [`SequenceValue`], and structs and tuple values can
/// both be represented with [`CompositeValue`]. Only enough information is preserved to
/// construct a valid value for any type that we know about, and it should be possible to
/// verify whether a value can be treated as a given [`crate::substrate_type::SubstrateType`]
/// or not.
#[derive(Clone, PartialEq)]
pub enum Value {
	/// Values for a named or unnamed struct or tuple.
	Composite(CompositeValue),
	/// An enum variant.
	Variant(VariantValue),
	/// A value corresponding to a sequence or array type, or even a BitVec.
	Sequence(SequenceValue),
	/// Special handling for BitVec (since it has it's own scale_info type).
	/// We make assumptions about the bitvec structure (based on how we decoded
	/// these prior to V14).
	BitSequence(BitSequenceValue),
	/// Any of the primitive values we can have.
	Primitive(PrimitiveValue),
}

impl Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Composite(val) => Debug::fmt(val, f),
			Self::Variant(val) => Debug::fmt(val, f),
			Self::Sequence(val) => Debug::fmt(val, f),
			Self::Primitive(val) => Debug::fmt(val, f),
			Self::BitSequence(val) => Debug::fmt(val, f),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum CompositeValue {
	/// Eg `{ foo: 2, bar: false }`
	Named(Vec<(String, Value)>),
	/// Eg `(2, false)`
	Unnamed(Vec<Value>),
}

impl Debug for CompositeValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CompositeValue::Named(fields) => {
				let mut struc = f.debug_struct("");
				for (name, val) in fields {
					struc.field(name, val);
				}
				struc.finish()
			}
			CompositeValue::Unnamed(fields) => {
				let mut struc = f.debug_tuple("");
				for val in fields {
					struc.field(val);
				}
				struc.finish()
			}
		}
	}
}

impl From<CompositeValue> for Value {
	fn from(val: CompositeValue) -> Self {
		Value::Composite(val)
	}
}

#[derive(Clone, PartialEq)]
pub struct VariantValue {
	pub name: String,
	pub fields: CompositeValue,
}

impl Debug for VariantValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.name)?;
		f.write_str(" ")?;
		Debug::fmt(&self.fields, f)
	}
}

impl From<VariantValue> for Value {
	fn from(val: VariantValue) -> Self {
		Value::Variant(val)
	}
}

#[derive(Clone, PartialEq)]
pub enum PrimitiveValue {
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

impl Debug for PrimitiveValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PrimitiveValue::Bool(val) => Debug::fmt(val, f),
			PrimitiveValue::Char(val) => Debug::fmt(val, f),
			PrimitiveValue::Str(val) => Debug::fmt(val, f),
			PrimitiveValue::U8(val) => Debug::fmt(val, f),
			PrimitiveValue::U16(val) => Debug::fmt(val, f),
			PrimitiveValue::U32(val) => Debug::fmt(val, f),
			PrimitiveValue::U64(val) => Debug::fmt(val, f),
			PrimitiveValue::U128(val) => Debug::fmt(val, f),
			PrimitiveValue::I8(val) => Debug::fmt(val, f),
			PrimitiveValue::I16(val) => Debug::fmt(val, f),
			PrimitiveValue::I32(val) => Debug::fmt(val, f),
			PrimitiveValue::I64(val) => Debug::fmt(val, f),
			PrimitiveValue::I128(val) => Debug::fmt(val, f),
			PrimitiveValue::U256(val) | PrimitiveValue::I256(val) => {
				f.write_str("BigNum(")?;
				Debug::fmt(val, f)?;
				f.write_str(")")
			}
		}
	}
}

impl From<PrimitiveValue> for Value {
	fn from(val: PrimitiveValue) -> Self {
		Value::Primitive(val)
	}
}

pub type SequenceValue = Vec<Value>;
pub type BitSequenceValue = BitVec<Lsb0, u8>;
