// Copyright 2019 Parity Technologies (UK) Ltd.
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

//! Stucture for registering substrate types
//! Generic SubstrateType enum
//! Serialization and Deserialization Implementations (to serialize as if it were a native type)
//! Display Implementation

use super::remote;
use primitives::crypto::AccountId32;
use primitives::crypto::{Ss58AddressFormat, Ss58Codec};
use serde::{ Serialize, Deserialize };
use std::{convert::TryFrom, fmt};

pub use super::data::Data;

pub type Address = runtime_primitives::MultiAddress<AccountId32, u32>;
pub type Vote = pallet_democracy::Vote;
pub type Conviction = pallet_democracy::Conviction;

/// A substrate value which can be instantiated at runtime (normally
/// through being decoded from SCALE encoded bytes).
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum SubstrateValue {
	/// 512-bit hash type
	H512(primitives::H512),
	/// 256-bit hash type
	H256(primitives::H256),

	/// Recursive Call Type
	Call(Vec<(String, SubstrateValue)>),
	/// Era
	Era(runtime_primitives::generic::Era),

	/// Vote
	#[serde(with = "remote::RemoteVote")]
	GenericVote(pallet_democracy::Vote),

	/// Substrate Indices Address Type
	// TODO: this is not generic for any chain that doesn't use a
	// u32 and [u8; 32] for its index/id
	#[serde(with = "remote::RemoteAddress")]
	Address(Address),
	/// Data Identity Type
	Data(Data),
	/// SignedExtension Type
	SignedExtra(String),

	/// Rust unit type (Struct or enum variant)
	Unit(String),

	/// vectors, arrays, and tuples
	#[serde(serialize_with = "crate::util::as_hex")]
	Composite(Vec<SubstrateValue>),

	/// C-Like Enum Type
	Set(SetField),
	/// Enum
	Enum(EnumField),
	/// Struct Type
	Struct(Vec<StructField>),
	/// Option Type
	Option(Box<Option<SubstrateValue>>),
	/// Result Type
	Result(Box<Result<SubstrateValue, SubstrateValue>>),

	// Std
	/// The unsigned 8-bit type
	U8(u8),
	/// unsigned 16-bit type
	U16(u16),
	/// unsigned 32-bit type
	U32(u32),
	/// unsigned 64-bit type
	U64(u64),
	/// unsigned 128-bit type
	U128(u128),
	/// unsigned cpu word-size type
	USize(usize),
	/// signed 8-bit type
	I8(i8),
	/// signed 16-bit type
	I16(i16),
	/// signed 32-bit type
	I32(i32),
	/// signed 64-bit type
	I64(i64),
	/// signed 128-bit type
	I128(i128),
	/// signed word-sized type
	ISize(isize),
	/// floating-point 32-bit type (not supported by SCALE)
	F32(f32),
	/// floating-point 64-bit type (not supported by SCALE)
	F64(f64),

	/// boolean type
	Bool(bool),
	// not sure what to do with this yet
	// may get rid of it
	Null,
}

impl fmt::Display for SubstrateValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SubstrateValue::H512(v) => write!(f, "{}", v),
			SubstrateValue::H256(v) => write!(f, "{}", v),
			SubstrateValue::Call(c) => {
				write!(f, "CALL")?;
				for arg in c.iter() {
					write!(f, "{}: {}", arg.0, arg.1)?;
				}
				Ok(())
			}
			SubstrateValue::Era(v) => match v {
				runtime_primitives::generic::Era::Mortal(s, e) => write!(f, " Era {}..{}", s, e),
				runtime_primitives::generic::Era::Immortal => write!(f, " Immortal Era"),
			},
			SubstrateValue::GenericVote(v) => write!(f, "Aye={}, Conviction={}", v.aye, v.conviction.lock_periods()),
			SubstrateValue::Address(v) => match v {
				runtime_primitives::MultiAddress::Id(ref i) => {
					write!(f, "Account::Id({})", i.to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount))
				}
				runtime_primitives::MultiAddress::Index(i) => write!(f, "Index: {:?}", i),
				runtime_primitives::MultiAddress::Raw(bytes) => write!(f, "Raw: {:?}", bytes),
				runtime_primitives::MultiAddress::Address32(ary) => write!(f, "Address32: {:?}", ary),
				runtime_primitives::MultiAddress::Address20(ary) => write!(f, "Address20: {:?}", ary),
			},
			SubstrateValue::Data(d) => write!(f, "{:?}", d),
			SubstrateValue::SignedExtra(v) => write!(f, "{}", v),
			SubstrateValue::Unit(u) => write!(f, "{}", u),
			SubstrateValue::Composite(v) => {
				let mut s = String::from("");
				for v in v.iter() {
					s.push_str(&format!("{}", v))
				}
				write!(f, "{}", s)
			}
			SubstrateValue::Set(v) => write!(f, "{}", v),
			SubstrateValue::Enum(v) => write!(f, "{}", v),
			SubstrateValue::Struct(v) => {
				let mut s = String::from("");
				for val in v.iter() {
					s.push_str(&format!("{}", val))
				}
				write!(f, "{}", s)
			}
			SubstrateValue::Option(v) => write!(f, "{:?}", v),
			SubstrateValue::Result(v) => write!(f, "{:?}", v),
			SubstrateValue::U8(v) => {
				write!(f, "{:X}", v) // u8's print in hex format
			}
			SubstrateValue::U16(v) => write!(f, "{}", v),
			SubstrateValue::U32(v) => write!(f, "{}", v),
			SubstrateValue::U64(v) => write!(f, "{}", v),
			SubstrateValue::U128(v) => write!(f, "{}", v),
			SubstrateValue::USize(v) => write!(f, "{}", v),
			SubstrateValue::I8(v) => write!(f, "{}", v),
			SubstrateValue::I16(v) => write!(f, "{}", v),
			SubstrateValue::I32(v) => write!(f, "{}", v),
			SubstrateValue::I64(v) => write!(f, "{}", v),
			SubstrateValue::I128(v) => write!(f, "{}", v),
			SubstrateValue::ISize(v) => write!(f, "{}", v),
			SubstrateValue::F32(v) => write!(f, "{}", v),
			SubstrateValue::F64(v) => write!(f, "{}", v),
			SubstrateValue::Bool(v) => write!(f, "{}", v),
			SubstrateValue::Null => write!(f, "Null"),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SetField {
	pub name: String,
	pub num: u8,
}

impl std::fmt::Display for SetField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "set_field({} {})", self.name, self.num)
	}
}

impl SetField {
	pub fn new<S: Into<String>>(name: S, num: u8) -> Self {
		let (name, num) = (name.into(), num);
		Self { name, num }
	}
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct EnumField {
	/// name of the field.
	pub name: String,
	/// Optional field value. An enum field without a value are unit fields.
	pub value: Option<Box<SubstrateValue>>,
}

impl EnumField {
	pub fn new(name: String, value: Option<Box<SubstrateValue>>) -> Self {
		Self { name, value }
	}
}

impl fmt::Display for EnumField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "enum[{}:{}]", self.name, self.value.as_ref().unwrap_or(&Box::new(SubstrateValue::Null)))
	}
}

/// Type with an associated name
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct StructField {
	/// name of a field, if any
	/// this is an option, because IE a Tuple-enum Variant
	/// will not have named fields
	pub name: Option<String>,
	/// Type of field
	pub ty: SubstrateValue,
}

impl StructField {
	pub fn new<S: Into<String>>(name: Option<S>, ty: SubstrateValue) -> Self {
		let name: Option<String> = name.map(|s| s.into());
		Self { name, ty }
	}
}

impl fmt::Display for StructField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "struct_field( {:?}: {} )", self.name, self.ty)
	}
}

// ============================================
// /\/\/\         CONVERSIONS            /\/\/\
// ============================================

#[derive(Debug, thiserror::Error)]
#[error("Cannot convert {actual} into {expected}")]
pub struct ConversionError {
	/// The name of the type we saw
	actual: String,
	/// The name of the type that we expected to see
	expected: String
}

impl TryFrom<&SubstrateValue> for Vec<u8> {
	type Error = ConversionError;
	fn try_from(ty: &SubstrateValue) -> Result<Vec<u8>, ConversionError> {
		match ty {
			SubstrateValue::Composite(elements) => {
				elements
					.iter()
					.map(|v| match v {
						SubstrateValue::U8(byte) => Ok(*byte),
						other => Err(ConversionError {
							actual: format!("{:?}", other),
							expected: "u8".to_string()
						})
					})
					.collect()
			}
			_ => Err(ConversionError {
				actual: format!("{}", ty),
				expected: "Vec<u8>".to_string()
			}),
		}
	}
}

impl From<u8> for SubstrateValue {
	fn from(num: u8) -> SubstrateValue {
		SubstrateValue::U8(num)
	}
}

impl From<u16> for SubstrateValue {
	fn from(num: u16) -> SubstrateValue {
		SubstrateValue::U16(num)
	}
}

impl From<u32> for SubstrateValue {
	fn from(num: u32) -> SubstrateValue {
		SubstrateValue::U32(num)
	}
}

impl From<u64> for SubstrateValue {
	fn from(num: u64) -> SubstrateValue {
		SubstrateValue::U64(num)
	}
}

impl From<u128> for SubstrateValue {
	fn from(num: u128) -> SubstrateValue {
		SubstrateValue::U128(num)
	}
}

impl From<usize> for SubstrateValue {
	fn from(num: usize) -> SubstrateValue {
		SubstrateValue::USize(num)
	}
}

impl From<i8> for SubstrateValue {
	fn from(num: i8) -> SubstrateValue {
		SubstrateValue::I8(num)
	}
}

impl From<i16> for SubstrateValue {
	fn from(num: i16) -> SubstrateValue {
		SubstrateValue::I16(num)
	}
}

impl From<i32> for SubstrateValue {
	fn from(num: i32) -> SubstrateValue {
		SubstrateValue::I32(num)
	}
}

impl From<i64> for SubstrateValue {
	fn from(num: i64) -> SubstrateValue {
		SubstrateValue::I64(num)
	}
}

impl From<i128> for SubstrateValue {
	fn from(num: i128) -> SubstrateValue {
		SubstrateValue::I128(num)
	}
}

impl From<isize> for SubstrateValue {
	fn from(num: isize) -> SubstrateValue {
		SubstrateValue::ISize(num)
	}
}

impl From<f32> for SubstrateValue {
	fn from(num: f32) -> SubstrateValue {
		SubstrateValue::F32(num)
	}
}

impl From<f64> for SubstrateValue {
	fn from(num: f64) -> SubstrateValue {
		SubstrateValue::F64(num)
	}
}

impl From<bool> for SubstrateValue {
	fn from(val: bool) -> SubstrateValue {
		SubstrateValue::Bool(val)
	}
}
