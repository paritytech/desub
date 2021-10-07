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

#[forbid(unsafe_code)]
#[deny(unused)]
pub mod decoder;
mod error;
pub mod regex;
mod substrate_types;
mod util;

#[cfg(test)]
pub mod test_suite;

pub use self::error::Error;
pub use self::substrate_types::SubstrateType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub trait TypeDetective: fmt::Debug + dyn_clone::DynClone + Send + Sync {
	/// Get a 'RustTypeMarker'
	fn get(&self, chain: &str, spec: u32, module: &str, ty: &str) -> Option<&RustTypeMarker>;

	/// Some types have a fallback type that may be decoded into if the original
	/// type fails.
	fn try_fallback(&self, module: &str, ty: &str) -> Option<&RustTypeMarker>;

	/// get a type specific to decoding extrinsics
	fn get_extrinsic_ty(&self, chain: &str, spec: u32, ty: &str) -> Option<&RustTypeMarker>;
}

/// A field with an associated name
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct StructField {
	pub name: String,
	pub ty: RustTypeMarker,
}

impl Display for StructField {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "struct_field({}: {})", self.name, self.ty)
	}
}

impl StructField {
	pub fn new<S: Into<String>>(name: S, ty: RustTypeMarker) -> Self {
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct EnumField {
	/// name of the Variant
	/// if the variant is a Unit enum, it will not have a name
	pub name: String,
	pub value: Option<RustTypeMarker>,
}

impl EnumField {
	pub fn new(name: String, value: Option<RustTypeMarker>) -> Self {
		EnumField { name, value }
	}
}

impl Display for EnumField {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "enum[{}:{}]", self.name, self.value.as_ref().unwrap_or(&RustTypeMarker::Null))
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
/// Definitions for common patterns seen in Substrate/Polkadot
/// type definitions
/// Definitions for Vec/Option/Compact
/// Boxed because self-referential otherwise
pub enum CommonTypes {
	/// Rust std Vec<T> type
	Vec(Box<RustTypeMarker>),
	/// Rust std Option<T> type
	Option(Box<RustTypeMarker>),
	/// Rust  Result<T, E> type
	Result(Box<RustTypeMarker>, Box<RustTypeMarker>),
	/// parity-scale-codec Compact<T> type
	Compact(Box<RustTypeMarker>),
}

impl Display for CommonTypes {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut common_types = String::from("");
		match self {
			CommonTypes::Vec(t) => {
				common_types.push_str(&format!("Vec<{}>", t));
			}
			CommonTypes::Option(t) => {
				common_types.push_str(&format!("Option<{}>", t));
			}
			CommonTypes::Result(r, e) => {
				common_types.push_str(&format!("Result<{},{}>", r, e));
			}
			CommonTypes::Compact(t) => {
				common_types.push_str(&format!("Compact<{}>", t));
			}
		}
		write!(f, "{}", common_types)
	}
}

impl CommonTypes {
	/// returns the inner types of Common Rust Constructs
	/// types with more than one generic (E.G Result<T, E>)
	/// are indexes in a Vector
	/// Types with only one generic (E.G Option<T>) have only
	/// one vector element
	pub fn get_inner_type(&self) -> Vec<&RustTypeMarker> {
		match self {
			CommonTypes::Vec(ref v_inner) => vec![v_inner],
			CommonTypes::Option(ref o_inner) => vec![o_inner],
			CommonTypes::Result(ref r_inner1, ref r_inner2) => vec![r_inner1, r_inner2],
			CommonTypes::Compact(ref c_inner) => vec![c_inner],
		}
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RustTypeMarker {
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
	Tuple(Vec<RustTypeMarker>),

	/// A Rust enum
	Enum(Vec<EnumField>),

	/// A sized array
	Array {
		/// size of the array
		size: usize,
		/// type of array
		ty: Box<RustTypeMarker>,
	},

	/// Definitions for common patterns seen in substrate/polkadot
	/// type definitions
	Std(CommonTypes),

	/// A Generic Type, EX: HeartBeat<BlockNumber>
	/// Tuple of (OuterType, InnerType)
	Generic(Box<RustTypeMarker>, Box<RustTypeMarker>),
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

	/// Boolean true/false type
	Bool,

	/// Used for fields that don't exist (ex Unit variant in an enum with both
	/// units/structs)
	Null,
}

fn display_types(fields: &[RustTypeMarker]) -> String {
	let mut s = String::new();

	s.push('(');
	for substring in fields.iter() {
		s.push_str(&format!("{}, ", substring))
	}
	s.push(')');
	s
}

impl Display for RustTypeMarker {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut type_marker = String::from("");
		match self {
			RustTypeMarker::TypePointer(t) => type_marker.push_str(t),
			RustTypeMarker::Unit(u) => type_marker.push_str(u),
			RustTypeMarker::Struct(t) => {
				for substring in t.iter() {
					type_marker.push_str(&format!("{}, ", substring))
				}
			}
			RustTypeMarker::Set(t) => {
				for substring in t.iter() {
					type_marker.push_str(&format!("{}, ", substring))
				}
			}
			RustTypeMarker::Tuple(t) => type_marker.push_str(&display_types(t)),
			RustTypeMarker::Enum(t) => {
				type_marker.push_str("{ ");
				for field in t.iter() {
					type_marker.push_str(&format!("{} ,", &field.to_string()));
				}
				type_marker.push_str(" }")
			}
			RustTypeMarker::Array { size, ty } => type_marker.push_str(&format!("[{};{}], ", ty, size)),
			RustTypeMarker::Std(t) => type_marker.push_str(&t.to_string()),
			RustTypeMarker::Generic(outer, inner) => type_marker.push_str(&format!("{}<{}>", outer, inner)),
			RustTypeMarker::Number => type_marker.push_str("number"),
			RustTypeMarker::U8 => type_marker.push_str("u8"),
			RustTypeMarker::U16 => type_marker.push_str("u16"),
			RustTypeMarker::U32 => type_marker.push_str("u32"),
			RustTypeMarker::U64 => type_marker.push_str("u64"),
			RustTypeMarker::U128 => type_marker.push_str("u128"),

			RustTypeMarker::I8 => type_marker.push_str("i8"),
			RustTypeMarker::I16 => type_marker.push_str("i16"),
			RustTypeMarker::I32 => type_marker.push_str("i32"),
			RustTypeMarker::I64 => type_marker.push_str("i64"),
			RustTypeMarker::I128 => type_marker.push_str("i128"),
			RustTypeMarker::Bool => type_marker.push_str("bool"),
			RustTypeMarker::Null => type_marker.push_str("null"),
		}
		write!(f, "{}", type_marker)
	}
}
