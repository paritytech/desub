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

mod data;
mod remote;

use self::remote::*;
use crate::{Error, SetField};
use bitvec::order::Lsb0 as BitOrderLsb0;
use serde::Serialize;
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_runtime::MultiAddress;
use std::{convert::TryFrom, fmt};

pub use self::data::Data;

pub type Address = MultiAddress<AccountId32, u32>;

/// Stripped down version of https://docs.substrate.io/rustdocs/latest/pallet_democracy
/// Remove when/if the real pallet_democracy is published.
pub mod pallet_democracy {
	use codec::{Decode, Input};
	use sp_runtime::RuntimeDebug;
	/// Static copy of https://docs.substrate.io/rustdocs/latest/pallet_democracy/struct.Vote.html
	#[derive(Copy, Clone, Eq, PartialEq, Default, RuntimeDebug)]
	pub struct Vote {
		pub aye: bool,
		pub conviction: Conviction,
	}

	impl Decode for Vote {
		fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
			let b = input.read_byte()?;
			Ok(Vote {
				aye: (b & 0b1000_0000) == 0b1000_0000,
				conviction: Conviction::try_from(b & 0b0111_1111)
					.map_err(|_| codec::Error::from("Invalid conviction"))?,
			})
		}
	}

	/// Static copy of https://docs.substrate.io/rustdocs/latest/pallet_democracy/enum.Conviction.html
	#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum Conviction {
		/// 0.1x votes, unlocked.
		None,
		/// 1x votes, locked for an enactment period following a successful vote.
		Locked1x,
		/// 2x votes, locked for 2x enactment periods following a successful vote.
		Locked2x,
		/// 3x votes, locked for 4x...
		Locked3x,
		/// 4x votes, locked for 8x...
		Locked4x,
		/// 5x votes, locked for 16x...
		Locked5x,
		/// 6x votes, locked for 32x...
		Locked6x,
	}

	impl Default for Conviction {
		fn default() -> Self {
			Conviction::None
		}
	}

	impl Conviction {
		/// The amount of time (in number of periods) that our conviction implies a successful voter's
		/// balance should be locked for.
		pub fn lock_periods(self) -> u32 {
			match self {
				Conviction::None => 0,
				Conviction::Locked1x => 1,
				Conviction::Locked2x => 2,
				Conviction::Locked3x => 4,
				Conviction::Locked4x => 8,
				Conviction::Locked5x => 16,
				Conviction::Locked6x => 32,
			}
		}
	}

	impl TryFrom<u8> for Conviction {
		type Error = ();
		fn try_from(i: u8) -> Result<Conviction, ()> {
			Ok(match i {
				0 => Conviction::None,
				1 => Conviction::Locked1x,
				2 => Conviction::Locked2x,
				3 => Conviction::Locked3x,
				4 => Conviction::Locked4x,
				5 => Conviction::Locked5x,
				6 => Conviction::Locked6x,
				_ => return Err(()),
			})
		}
	}
}

/// A 'stateful' version of [RustTypeMarker](enum.RustTypeMarker.html).
/// 'Std' variant is not here like in RustTypeMarker.
/// Instead common types are just apart of the enum
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum SubstrateType {
	/// 512-bit hash type
	H512(sp_core::H512),
	/// 256-bit hash type
	H256(sp_core::H256),

	/// BitVec type
	BitVec(bitvec::vec::BitVec<BitOrderLsb0, u8>),

	/// Recursive Call Type
	Call(Vec<(String, SubstrateType)>),
	/// Era
	Era(sp_runtime::generic::Era),

	/// Vote
	#[serde(with = "RemoteVote")]
	GenericVote(pallet_democracy::Vote),

	/// Substrate Indices Address Type
	#[serde(with = "desub_common::RemoteAddress")]
	Address(Address),
	/// Data Identity Type
	Data(Data),

	/// Identity fields but as just an enum.
	IdentityField(u64),

	/// SignedExtension Type
	SignedExtra(String),

	/// Rust unit type (Struct or enum variant)
	Unit(String),

	/// vectors, arrays, and tuples
	#[serde(serialize_with = "crate::util::as_hex")]
	Composite(Vec<SubstrateType>),

	/// C-Like Enum Type
	Set(SetField),
	/// Enum
	Enum(EnumField),
	/// Struct Type
	Struct(Vec<StructField>),
	/// Option Type
	Option(Box<Option<SubstrateType>>),
	/// Result Type
	Result(Box<Result<SubstrateType, SubstrateType>>),

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

impl fmt::Display for SubstrateType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SubstrateType::H512(v) => write!(f, "{}", v),
			SubstrateType::H256(v) => write!(f, "{}", v),
			SubstrateType::BitVec(v) => write!(f, "{}", v),
			SubstrateType::Call(c) => {
				write!(f, "CALL")?;
				for arg in c.iter() {
					write!(f, "{}: {}", arg.0, arg.1)?;
				}
				Ok(())
			}
			SubstrateType::Era(v) => match v {
				sp_runtime::generic::Era::Mortal(s, e) => write!(f, " Era {}..{}", s, e),
				sp_runtime::generic::Era::Immortal => write!(f, " Immortal Era"),
			},
			SubstrateType::GenericVote(v) => write!(f, "Aye={}, Conviction={}", v.aye, v.conviction.lock_periods()),
			SubstrateType::Address(v) => match v {
				sp_runtime::MultiAddress::Id(ref i) => {
					write!(f, "Account::Id({})", i.to_ss58check())
				}
				sp_runtime::MultiAddress::Index(i) => write!(f, "Index: {:?}", i),
				sp_runtime::MultiAddress::Raw(bytes) => write!(f, "Raw: {:?}", bytes),
				sp_runtime::MultiAddress::Address32(ary) => write!(f, "Address32: {:?}", ary),
				sp_runtime::MultiAddress::Address20(ary) => write!(f, "Address20: {:?}", ary),
			},
			SubstrateType::Data(d) => write!(f, "{:?}", d),
			SubstrateType::SignedExtra(v) => write!(f, "{}", v),
			SubstrateType::Unit(u) => write!(f, "{}", u),
			SubstrateType::IdentityField(field) => write!(f, "{:?}", field),
			SubstrateType::Composite(v) => {
				let mut s = String::from("");
				for v in v.iter() {
					s.push_str(&format!("{}", v))
				}
				write!(f, "{}", s)
			}
			SubstrateType::Set(v) => write!(f, "{}", v),
			SubstrateType::Enum(v) => write!(f, "{}", v),
			SubstrateType::Struct(v) => {
				let mut s = String::from("");
				for val in v.iter() {
					s.push_str(&format!("{}", val))
				}
				write!(f, "{}", s)
			}
			SubstrateType::Option(v) => write!(f, "{:?}", v),
			SubstrateType::Result(v) => write!(f, "{:?}", v),
			SubstrateType::U8(v) => {
				write!(f, "{:X}", v) // u8's print in hex format
			}
			SubstrateType::U16(v) => write!(f, "{}", v),
			SubstrateType::U32(v) => write!(f, "{}", v),
			SubstrateType::U64(v) => write!(f, "{}", v),
			SubstrateType::U128(v) => write!(f, "{}", v),
			SubstrateType::USize(v) => write!(f, "{}", v),
			SubstrateType::I8(v) => write!(f, "{}", v),
			SubstrateType::I16(v) => write!(f, "{}", v),
			SubstrateType::I32(v) => write!(f, "{}", v),
			SubstrateType::I64(v) => write!(f, "{}", v),
			SubstrateType::I128(v) => write!(f, "{}", v),
			SubstrateType::ISize(v) => write!(f, "{}", v),
			SubstrateType::F32(v) => write!(f, "{}", v),
			SubstrateType::F64(v) => write!(f, "{}", v),
			SubstrateType::Bool(v) => write!(f, "{}", v),
			SubstrateType::Null => write!(f, "Null"),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct EnumField {
	/// name of the field.
	pub name: String,
	/// Optional field value. An enum field without a value are unit fields.
	pub value: Option<Box<SubstrateType>>,
}

impl EnumField {
	pub fn new(name: String, value: Option<Box<SubstrateType>>) -> Self {
		Self { name, value }
	}
}

impl fmt::Display for EnumField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "enum[{}:{}]", self.name, self.value.as_ref().unwrap_or(&Box::new(SubstrateType::Null)))
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
	pub ty: SubstrateType,
}

impl StructField {
	pub fn new<S: Into<String>>(name: Option<S>, ty: SubstrateType) -> Self {
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

impl TryFrom<&SubstrateType> for Vec<u8> {
	type Error = Error;
	fn try_from(ty: &SubstrateType) -> Result<Vec<u8>, Error> {
		match ty {
			SubstrateType::Composite(elements) => {
				if elements.iter().any(|ty| !matches!(ty, SubstrateType::U8(_))) {
					Err(Error::Conversion(format!("{:?}", ty), "u8".to_string()))
				} else {
					Ok(elements
						.iter()
						.map(|v| match v {
							SubstrateType::U8(byte) => *byte,
							_ => unreachable!(),
						})
						.collect::<Vec<u8>>())
				}
			}
			_ => Err(Error::Conversion(format!("{}", ty), "Vec<u8>".to_string())),
		}
	}
}

impl From<u8> for SubstrateType {
	fn from(num: u8) -> SubstrateType {
		SubstrateType::U8(num)
	}
}

impl From<u16> for SubstrateType {
	fn from(num: u16) -> SubstrateType {
		SubstrateType::U16(num)
	}
}

impl From<u32> for SubstrateType {
	fn from(num: u32) -> SubstrateType {
		SubstrateType::U32(num)
	}
}

impl From<u64> for SubstrateType {
	fn from(num: u64) -> SubstrateType {
		SubstrateType::U64(num)
	}
}

impl From<u128> for SubstrateType {
	fn from(num: u128) -> SubstrateType {
		SubstrateType::U128(num)
	}
}

impl From<usize> for SubstrateType {
	fn from(num: usize) -> SubstrateType {
		SubstrateType::USize(num)
	}
}

impl From<i8> for SubstrateType {
	fn from(num: i8) -> SubstrateType {
		SubstrateType::I8(num)
	}
}

impl From<i16> for SubstrateType {
	fn from(num: i16) -> SubstrateType {
		SubstrateType::I16(num)
	}
}

impl From<i32> for SubstrateType {
	fn from(num: i32) -> SubstrateType {
		SubstrateType::I32(num)
	}
}

impl From<i64> for SubstrateType {
	fn from(num: i64) -> SubstrateType {
		SubstrateType::I64(num)
	}
}

impl From<i128> for SubstrateType {
	fn from(num: i128) -> SubstrateType {
		SubstrateType::I128(num)
	}
}

impl From<isize> for SubstrateType {
	fn from(num: isize) -> SubstrateType {
		SubstrateType::ISize(num)
	}
}

impl From<f32> for SubstrateType {
	fn from(num: f32) -> SubstrateType {
		SubstrateType::F32(num)
	}
}

impl From<f64> for SubstrateType {
	fn from(num: f64) -> SubstrateType {
		SubstrateType::F64(num)
	}
}

impl From<bool> for SubstrateType {
	fn from(val: bool) -> SubstrateType {
		SubstrateType::Bool(val)
	}
}
