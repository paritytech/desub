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

mod version_14;

use crate::substrate_type::{ConvertError, SubstrateType};
use codec::Decode;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use scale_info::PortableRegistry;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
	#[error("Cannot decode bytes into metadata: {0}")]
	DecodeError(#[from] DecodeError),
}

/// An error related to attempting to decode metadata from a slice of byres.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeError {
	#[error("metadata version {0} is not supported")]
	UnsupportedVersion(usize),
	#[error("{0}")]
	CodecError(#[from] codec::Error),
	#[error("unexpected type; expecting a Variant type, but got {got}")]
	ExpectedVariantType { got: String },
	#[error("could not convert type into the desired format: {0}")]
	ConvertError(#[from] ConvertError),
	#[error("could not find type with ID {0}")]
	TypeNotFound(u32),
}

/// A Representation of some metadata for a node which aids in the
/// decoding of SCALE encoded data and such.
pub struct Metadata {
	extrinsic: MetadataExtrinsic,
	pallets: Vec<MetadataPallet>,
	types: PortableRegistry,
}

/// Types are internally stored away in a type registry. Rather than exposing
/// The scale-info logic, we store and hand back these pointers to the type
/// information. This can be resolved into [`crate::substrate_type::SubstrateType`]
/// when you'd like the full type information to work with.
#[derive(Debug, Clone, Copy)]
pub struct TypeId(u32);

#[derive(Debug)]
struct MetadataPallet {
	name: String,
	calls: Vec<MetadataCall>,
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
			}
		}
	}

	/// Given the `u8` variant index of a pallet and call, this returns information about
	/// the call if it's fgound, or `None` if it no such call exists at those indexes.
	pub fn call_by_variant_index(&self, pallet: u8, call: u8) -> Option<(&str, &MetadataCall)> {
		self.pallets.get(pallet as usize).and_then(|p| {
			let call = p.calls.get(call as usize)?;
			Some((&*p.name, call))
		})
	}

	/// Return information about the metadata extrinsic format.
	pub fn extrinsic(&self) -> &MetadataExtrinsic {
		&self.extrinsic
	}

	/// Given a [`TypeId`], attempt to resolve it into a [`SubstrateType`].
	///
	/// We hand back [`TypeId`]'s rather than [`SubstrateType`]'s in most places because [`SubstrateType`]'s
	/// are not as space/allocation friendly as the type registry. That said, they are easier to work with and
	/// can be manually constructed, which makes it easier to use them.
	pub fn resolve_type(&self, id: &TypeId) -> Result<SubstrateType, MetadataError> {
		let ty = self.types.resolve(id.0).ok_or(MetadataError::DecodeError(DecodeError::TypeNotFound(id.0)))?;
		let substrate_ty =
			SubstrateType::from_scale_info_type(ty, &self.types).map_err(|e| MetadataError::DecodeError(e.into()))?;
		Ok(substrate_ty)
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

/// This represents a single call (extrinsic) that exists in the system.
#[derive(Debug)]
pub struct MetadataCall {
	name: String,
	args: Vec<TypeId>,
}

impl MetadataCall {
	/// The name of the function call.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// The types expected to be provided as arguments to this call.
	/// [`TypeId`]'s can be resolved into [`SubstrateType`]'s using
	/// [`Metadata::resolve_type`]
	pub fn args(&self) -> &[TypeId] {
		&self.args
	}
}

/// Information about the shape of an extrinsic. This is not complete, and so
/// one must decode based on the extrinsic version number as much as anything,
/// but we can use this to help decode part of the signature.
#[derive(Debug)]
pub struct MetadataExtrinsic {
	version: u8,
}

impl MetadataExtrinsic {
	/// The version of the extrinsic format in use by the node. Extrinsics have
	/// a version embedded into them anyway, so we don't need this to decode them,
	/// but it may be useful for encoding in the future.
	pub fn version(&self) -> u8 {
		self.version
	}
}
