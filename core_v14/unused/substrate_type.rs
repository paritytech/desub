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
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// A field with an associated name
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct StructField {
	pub name: String,
	pub ty: SubstrateType,
}

impl Display for StructField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "struct_field({}: {})", self.name, self.ty)
	}
}

impl StructField {
	pub fn new<S: Into<String>>(name: S, ty: SubstrateType) -> Self {
		let name = name.into();
		Self { name, ty }
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SetField {
	pub name: String,
	pub num: u8,
}

impl Display for SetField {
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

/// TODO: Allow mixed struct-unit types
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct EnumField {
	/// name of the Variant
	/// if the variant is a Unit enum, it will not have a name
	pub name: String,
	pub value: Option<SubstrateType>,
}

impl EnumField {
	pub fn new(name: String, value: Option<SubstrateType>) -> Self {
		EnumField { name, value }
	}
}

impl Display for EnumField {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "enum[{}:{}]", self.name, self.value.as_ref().unwrap_or(&SubstrateType::Null))
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
/// Definitions for common patterns seen in Substrate/Polkadot
/// type definitions
/// Definitions for Vec/Option/Compact
/// Boxed because self-referential otherwise
pub enum CommonTypes {
	/// Rust std Vec<T> type
	Vec(Box<SubstrateType>),
	/// Rust std Option<T> type
	Option(Box<SubstrateType>),
	/// Rust  Result<T, E> type
	Result(Box<SubstrateType>, Box<SubstrateType>),
	/// parity-scale-codec Compact<T> type
	Compact(Box<SubstrateType>),
}

impl Display for CommonTypes {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			CommonTypes::Vec(t) => {
				write!(f, "Vec<{}>", t)
			}
			CommonTypes::Option(t) => {
				write!(f, "Option<{}>", t)
			}
			CommonTypes::Result(r, e) => {
				write!(f, "Result<{},{}>", r, e)
			}
			CommonTypes::Compact(t) => {
				write!(f, "Compact<{}>", t)
			}
		}
	}
}

impl CommonTypes {
	/// returns the inner types of Common Rust Constructs
	/// types with more than one generic (E.G Result<T, E>)
	/// are indexes in a Vector
	/// Types with only one generic (E.G Option<T>) have only
	/// one vector element
	pub fn get_inner_type(&self) -> Vec<&SubstrateType> {
		match self {
			CommonTypes::Vec(ref v_inner) => vec![v_inner],
			CommonTypes::Option(ref o_inner) => vec![o_inner],
			CommonTypes::Result(ref r_inner1, ref r_inner2) => vec![r_inner1, r_inner2],
			CommonTypes::Compact(ref c_inner) => vec![c_inner],
		}
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SubstrateType {
	/// name of a type that exists elsewhere in type declarations
	TypePointer(String),

	/// A unit type. A struct or the variant of an enum.
	Unit(String),

	/// Some Struct
	/// Field Name -> Field Type
	Struct(Vec<StructField>),

	/// A C-Like Enum
	Set(Vec<SetField>),

	/// A tuple type (max size 32)
	Tuple(Vec<SubstrateType>),

	/// A Rust enum
	Enum(Vec<EnumField>),

	/// A sized array
	Array {
		/// size of the array
		size: usize,
		/// type of array
		ty: Box<SubstrateType>,
	},

	/// Definitions for common patterns seen in substrate/polkadot
	/// type definitions
	Std(CommonTypes),

	/// A Generic Type, EX: HeartBeat<BlockNumber>
	/// Tuple of (OuterType, InnerType)
	Generic(Box<SubstrateType>, Box<SubstrateType>),
	/// A Number for which the bit size is unknown
	Number,
	/// primitive unsigned 8 bit integer
	U8,
	/// primitive unsigned 16 bit integer
	U16,
	/// primitive unsigned 32 bit integer
	U32,
	/// primitive unsigned 64 bit integer
	U64,
	/// primitive unsigned 128 bit integer
	U128,
	/// primitive unsigned word-sized integer
	USize,

	/// primitive signed 8 bit integer
	I8,
	/// primitive signed 16 bit integer
	I16,
	/// primitive signed 32 bit integer
	I32,
	/// primitive signed 64 bit integer
	I64,
	/// primitive signed 128 bit integer
	I128,
	/// primitive signed word-sized integer
	ISize,

	/// primitive IEEE-spec 32-bit floating-point number
	F32,
	/// primitive IEEE-spec 64-bit floating-point number
	F64,

	/// Boolean true/false type
	Bool,

	/// String type
	String,

	/// Used for fields that don't exist (ex Unit variant in an enum with both
	/// units/structs)
	Null,
}

fn display_types(fields: &[SubstrateType]) -> String {
	let mut s = String::new();

	s.push('(');
	for substring in fields.iter() {
		s.push_str(&substring.to_string());
        s.push_str(", ");
	}
	s.push(')');
	s
}

impl Display for SubstrateType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SubstrateType::TypePointer(t) => write!(f, "{}", t),
			SubstrateType::Unit(u) => write!(f, "{}", u),
			SubstrateType::Struct(t) => {
				for substring in t.iter() {
					write!(f, "{}, ", substring)?;
				}
                Ok(())
			}
			SubstrateType::Set(t) => {
				for substring in t.iter() {
					write!(f, "{}, ", substring)?;
				}
                Ok(())
			}
			SubstrateType::Tuple(t) => write!(f, "{}", &display_types(t)),
			SubstrateType::Enum(t) => {
				write!(f, "{}", "{ ");
				for field in t.iter() {
					write!(f, "{} ,", field.to_string())?;
				}
				write!(f, "{}", " }")
			}
			SubstrateType::Array { size, ty } => write!(f, "{}", &format!("[{};{}], ", ty, size)),
			SubstrateType::Std(t) => write!(f, "{}", t.to_string()),
			SubstrateType::Generic(outer, inner) => write!(f, "{}", &format!("{}<{}>", outer, inner)),
			SubstrateType::Number => write!(f, "number"),
			SubstrateType::U8 => write!(f, "u8"),
			SubstrateType::U16 => write!(f, "u16"),
			SubstrateType::U32 => write!(f, "u32"),
			SubstrateType::U64 => write!(f, "u64"),
			SubstrateType::U128 => write!(f, "u128"),
			SubstrateType::USize => write!(f, "usize"),

			SubstrateType::I8 => write!(f, "i8"),
			SubstrateType::I16 => write!(f, "i16"),
			SubstrateType::I32 => write!(f, "i32"),
			SubstrateType::I64 => write!(f, "i64"),
			SubstrateType::I128 => write!(f, "i128"),
			SubstrateType::ISize => write!(f, "isize"),

			SubstrateType::F32 => write!(f, "f32"),
			SubstrateType::F64 => write!(f, "f64"),

			SubstrateType::Bool => write!(f, "bool"),

			SubstrateType::String => write!(f, "string"),

			SubstrateType::Null => write!(f, "null"),
		}
	}
}
