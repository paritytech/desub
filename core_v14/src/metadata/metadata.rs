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

// taken directly and modified from substrate-subxt:
// https://github.com/paritytech/substrate-subxt

use codec::{ Decode };
use frame_metadata::{
	RuntimeMetadataPrefixed,
	RuntimeMetadata
};
use super::version_14;
use scale_info::PortableRegistry;
use std::fmt::Write;
use crate::util::{ for_each_between, ForEachBetween };

/// A variant describing the shape of a type.
pub type TypeDef = scale_info::TypeDef<scale_info::form::PortableForm>;

/// Information about a type, including its shape.
pub type Type = scale_info::Type<scale_info::form::PortableForm>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
    #[error("Cannot decode bytes into metadata: {0}")]
    DecodeError(#[from] DecodeError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeError {
    #[error("metadata version {0} is not supported")]
    UnsupportedVersion(usize),
    #[error("{0}")]
    DecodeError(#[from] codec::Error),
	#[error("unexpected type; expecting a Variant type, but got {got}")]
	ExpectedVariantType { got: String },
	#[error("could not find type with ID {0}")]
	TypeNotFound(u32)
}

pub struct Metadata {
	pub (super) pallets: Vec<MetadataPallet>
}

#[derive(Debug)]
pub struct MetadataPallet {
	pub (super) name: String,
	pub (super) calls: Vec<MetadataCall>
}

impl Metadata {
	/// Attempt to convert some SCALE encoded bytes into Metadata, returning
	/// an error if something goes wrong in doing so.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError> {
        log::trace!("Decoding metadata");
        let meta = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &*bytes)
            .map_err(|e| MetadataError::DecodeError(e.into()))?;

		match meta {
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V14(meta_v14)) => {
				log::trace!("V14 metadata found.");
				version_14::decode(meta_v14).map_err(|e| e.into())
			}
			RuntimeMetadataPrefixed(_, unsupported_meta) => {
				let version = runtime_metadata_version(&unsupported_meta);
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(version)))
			},
		}
	}

	/// Given the `u8` variant index of a pallet and call, this returns information about
	/// the call if it's fgound, or `None` if it no such call exists at those indexes.
	pub fn call_by_variant_index(&self, pallet: u8, call: u8) -> Option<&MetadataCall> {
		self.pallets
			.get(pallet as usize)
			.map(|p| p.calls.get(call as usize))
			.flatten()
	}
}

/// Get the decoded metadata version. At some point `RuntimeMetadataPrefixed` will end up
/// with a `.version()` method to return the version, and then this can be removed.
fn runtime_metadata_version(meta: &RuntimeMetadata) -> usize {
	match meta {
		RuntimeMetadata::V0(_) => 0,
		RuntimeMetadata::V1(_) => 1,
		RuntimeMetadata::V2(_) => 2,
		RuntimeMetadata::V3(_) => 3,
		RuntimeMetadata::V4(_) => 4,
		RuntimeMetadata::V5(_) => 5,
		RuntimeMetadata::V6(_) => 6,
		RuntimeMetadata::V7(_) => 7,
		RuntimeMetadata::V8(_) => 8,
		RuntimeMetadata::V9(_) => 9,
		RuntimeMetadata::V10(_) => 10,
		RuntimeMetadata::V11(_) => 11,
		RuntimeMetadata::V12(_) => 12,
		RuntimeMetadata::V13(_) => 13,
		RuntimeMetadata::V14(_) => 14,
	}
}

#[derive(Debug)]
pub struct MetadataCall {
	pub (super) name: String,
	pub (super) args: Vec<Type>
}

impl MetadataCall {
	/// The name of the function call.
	pub fn name(&self) -> &str {
		&self.name
	}
	/// The types expected to be provided as arguments to this call.
	pub fn args(&self) -> &[Type] {
		&self.args
	}
}

/// Output a human readable representation of the type provided.
pub (super) fn full_type_name(ty: &Type, registry: &PortableRegistry) -> String {
	let mut s = String::new();
	write_full_type_name(ty, registry, &mut s).expect("string shouldn't fmt error");
	s
}

/// Output a human readable representation of the type to the writer provided.
fn write_full_type_name<W: Write>(ty: &Type, registry: &PortableRegistry, w: &mut W) -> Result<(), std::fmt::Error> {
	let def = ty.type_def();
	let to_type = |ty: &<scale_info::form::PortableForm as scale_info::form::Form>::Type | {
		registry
			.resolve(ty.id())
			.expect("type ID to exist in registry")
	};

	match def {
		TypeDef::Array(inner) => {
			w.write_str("[")?;
			write_full_type_name(to_type(inner.type_param()), registry, w)?;
			w.write_str("; ")?;
			w.write_str(&inner.len().to_string())?;
			w.write_str("]")?;
		},
		TypeDef::BitSequence(_) => {
			w.write_str("BitSequence")?;
		},
		TypeDef::Compact(inner) => {
			w.write_str("Compact<")?;
			write_full_type_name(to_type(inner.type_param()), registry, w)?;
			w.write_str(">")?;
		},
		TypeDef::Primitive(prim) => {
			use scale_info::TypeDefPrimitive;
			match prim {
				TypeDefPrimitive::Bool => w.write_str("bool")?,
				TypeDefPrimitive::Char => w.write_str("char")?,
				TypeDefPrimitive::Str => w.write_str("str")?,
				TypeDefPrimitive::U8 => w.write_str("u8")?,
				TypeDefPrimitive::U16 => w.write_str("u16")?,
				TypeDefPrimitive::U32 => w.write_str("u32")?,
				TypeDefPrimitive::U64 => w.write_str("u64")?,
				TypeDefPrimitive::U128 => w.write_str("u128")?,
				TypeDefPrimitive::U256 => w.write_str("u256")?,
				TypeDefPrimitive::I8 => w.write_str("i8")?,
				TypeDefPrimitive::I16 => w.write_str("i16")?,
				TypeDefPrimitive::I32 => w.write_str("i32")?,
				TypeDefPrimitive::I64 => w.write_str("i64")?,
				TypeDefPrimitive::I128 => w.write_str("i128")?,
				TypeDefPrimitive::I256 => w.write_str("i256")?,
			}
		},
		TypeDef::Sequence(seq) => {
			w.write_str("Seq<")?;
			write_full_type_name(to_type(seq.type_param()), registry, w)?;
			w.write_str(">")?;
		},
		TypeDef::Tuple(tup) => {
			w.write_str("(")?;
			for field in for_each_between(tup.fields()) {
				match field {
					ForEachBetween::Item(field) => {
						write_full_type_name(to_type(field), registry, w)?;
					},
					ForEachBetween::Between => {
						w.write_str(", ")?;
					}
				}
			}
			w.write_str(")")?;
		},
		TypeDef::Variant(_) | TypeDef::Composite(_) => {
			// Just print the path for conciseness.
			for item in for_each_between(ty.path().segments()) {
				match item {
					ForEachBetween::Item(item) => {
						w.write_str(item)?;
					},
					ForEachBetween::Between => {
						w.write_str("::")?;
					}
				}
			}
		}
	};
	Ok(())
}
