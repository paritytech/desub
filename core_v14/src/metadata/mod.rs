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
Decode SCALE encoded metadata from a substrate node into a format that
we can pass to a [`crate::Decoder`].
*/

mod version_14;

use codec::Decode;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use scale_info::{form::PortableForm, PortableRegistry};
use std::collections::HashMap;

// Some type aliases used below. `scale-info` is re-exported at the root,
// so to avoid confusion we only publically export all scale-info types from that
// one place.
type TypeId = <scale_info::form::PortableForm as scale_info::form::Form>::Type;
type TypeDefVariant = scale_info::TypeDefVariant<PortableForm>;
type SignedExtensionMetadata = frame_metadata::SignedExtensionMetadata<PortableForm>;

/// An enum of the possible errors that can be returned from attempting to construct
/// a [`Metadata`] struct.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
	#[error("metadata version {0} is not supported")]
	UnsupportedVersion(usize),
	#[error("{0}")]
	CodecError(#[from] codec::Error),
	#[error("unexpected type; expecting a Variant type, but got {got}")]
	ExpectedVariantType { got: String },
	#[error("could not find type with ID {0}")]
	TypeNotFound(u32),
}

/// This is a representation of the SCALE encoded metadata obtained from a substrate
/// node. While not very useful on its own, It can be passed to [`crate::Decoder`]
/// to allow that to decode extrinsics compatible with the substrate node that
/// this was obtained from.
pub struct Metadata {
	extrinsic: MetadataExtrinsic,
	pallets: HashMap<u8, MetadataPallet>,
	types: PortableRegistry,
}

impl Metadata {
	/// Attempt to convert some SCALE encoded bytes into Metadata, returning an
	/// error if something goes wrong in doing so. Here's an example command using
	/// `curl` and `jq` to download this from a locally running node (on the default port)
	/// and save it as `node_metadata.scale`.
	///
	/// ```sh
	/// curl -sX POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"state_getMetadata", "id": 1}' localhost:9933 \
	///     | jq .result \
	///     | cut -d '"' -f 2 \
	///     | xxd -r -p > node_metadata.scale
	/// ```
	///
	/// This file can then be read and passed directly to this method.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError> {
		log::trace!("Decoding metadata");
		let meta = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &*bytes)?;

		match meta {
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V14(meta_v14)) => {
				log::trace!("V14 metadata found.");
				version_14::decode(meta_v14)
			}
			RuntimeMetadataPrefixed(_, unsupported_meta) => {
				let version = runtime_metadata_version(&unsupported_meta);
				Err(MetadataError::UnsupportedVersion(version))
			}
		}
	}

	/// Return details about the type of extrinsic supported by this metadata.
	pub fn extrinsic(&self) -> &MetadataExtrinsic {
		&self.extrinsic
	}

	/// Return a reference to the [`scale_info`] type registry.
	pub fn types(&self) -> &PortableRegistry {
		&self.types
	}

	/// Given the `u8` variant index of a pallet and call, this returns the pallet name and the call Variant
	/// if found, or `None` if it no such call exists at those indexes, or we don't have suitable call data.
	pub(crate) fn call_variant_by_enum_index(
		&self,
		pallet: u8,
		call: u8,
	) -> Option<(&str, &scale_info::Variant<PortableForm>)> {
		self.pallets.get(&pallet).and_then(|p| {
			p.calls.as_ref().and_then(|calls| {
				let type_def_variant = self.get_variant(calls.calls_type_id)?;
				let index = *calls.call_variant_indexes.get(&call)?;
				let variant = type_def_variant.variants().get(index)?;
				Some((&*p.name, variant))
			})
		})
	}

	/// A helper function to get hold of a Variant given a type ID, or None if it's not found.
	fn get_variant(&self, ty: TypeId) -> Option<&TypeDefVariant> {
		self.types.resolve(ty.id()).and_then(|ty| match ty.type_def() {
			scale_info::TypeDef::Variant(variant) => Some(variant),
			_ => None,
		})
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
struct MetadataPallet {
	name: String,
	/// Metadata may not contain call information. If it does,
	/// it'll be here.
	calls: Option<MetadataCalls>,
}

#[derive(Debug)]
struct MetadataCalls {
	/// This allows us to find the type information corresponding to
	/// the call in the [`PortableRegistry`]/
	calls_type_id: TypeId,
	/// This allows us to map a u8 enum index to the correct call variant
	/// from the calls type, above. The variant contains information on the
	/// fields and such that the call has.
	call_variant_indexes: HashMap<u8, usize>,
}

/// Information about the extrinsic format supported on the substrate node
/// that the metadata was obtained from.
#[derive(Debug, Clone)]
pub struct MetadataExtrinsic {
	version: u8,
	signed_extensions: Vec<SignedExtensionMetadata>,
}

impl MetadataExtrinsic {
	/// The version of the extrinsic format in use by the node.
	#[allow(unused)]
	pub fn version(&self) -> u8 {
		self.version
	}

	/// Part of the extrinsic signature area can be varied to include whatever information
	/// a node decides is important. This returns details about that part.
	pub(crate) fn signed_extensions(&self) -> &[SignedExtensionMetadata] {
		&self.signed_extensions
	}
}
