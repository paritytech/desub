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

//! Decode SCALE encoded metadata from a substrate node into a format that
//! we can make use of for decoding (see [`crate::decoder`]).

mod readonly_array;
mod u8_map;
mod version_14;

use crate::{ScaleInfoTypeId, Type, TypeId};
use codec::Decode;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use readonly_array::ReadonlyArray;
use scale_info::{form::PortableForm, PortableRegistry};
use u8_map::U8Map;

// Some type aliases used below. `scale-info` is re-exported at the root,
// so to avoid confusion we only publicly export all scale-info types from that
// one place.
type TypeDefVariant = scale_info::TypeDefVariant<PortableForm>;
type SignedExtensionMetadata = frame_metadata::SignedExtensionMetadata<PortableForm>;
type StorageEntryMetadata = frame_metadata::v14::StorageEntryMetadata<scale_info::form::PortableForm>;

/// An enum of the possible errors that can be returned from attempting to construct
/// a [`Metadata`] struct.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
	#[error("metadata version {0} is not supported")]
	UnsupportedVersion(u32),
	#[error("{0}")]
	CodecError(#[from] codec::Error),
	#[error("unexpected type; expecting a Variant type, but got {got}")]
	ExpectedVariantType { got: String },
	#[error("could not find type with ID {0}")]
	TypeNotFound(u32),
}

/// This is a representation of the SCALE encoded metadata obtained from a substrate
/// node. While not very useful on its own, It can be passed to [`crate::decoder`] functions
/// to decode encoded extrinsics and storage keys.
#[derive(Debug)]
pub struct Metadata {
	/// Details about the extrinsic format.
	extrinsic: MetadataExtrinsic,
	/// Hash pallet calls by index, since when decoding, we'll have the pallet/call
	/// `u8`'s available to us to look them up by.
	pallet_calls_by_index: U8Map<MetadataPalletCalls>,
	/// Store storage entry information as a readonly array, allowing us to look up a
	/// specific storage entry using a key like `(usize,usize)`. Since the order of
	/// entries in this array is not guaranteed between metadata versions, it should
	/// not be exposed.
	pub pallet_storage: ReadonlyArray<MetadataPalletStorage>,
	/// Type information lives inside this.
	pub types: PortableRegistry,
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
		let meta = RuntimeMetadataPrefixed::decode(&mut &*bytes)?;
		Self::from_runtime_metadata(meta.1)
	}

	/// Convert the substrate runtime metadata into our Metadata.
	pub fn from_runtime_metadata(metadata: RuntimeMetadata) -> Result<Self, MetadataError> {
		match metadata {
			RuntimeMetadata::V14(meta_v14) => {
				log::trace!("V14 metadata found.");
				version_14::decode(meta_v14)
			}
			unsupported_meta => Err(MetadataError::UnsupportedVersion(unsupported_meta.version())),
		}
	}

	/// Return details about the type of extrinsic supported by this metadata.
	pub fn extrinsic(&self) -> &MetadataExtrinsic {
		&self.extrinsic
	}

	pub fn get_types(&self) -> &PortableRegistry {
		&self.types
	}

	/// Given a [`crate::TypeId`], return the corresponding type from the type registry, if possible.
	pub fn resolve<Id: Into<TypeId>>(&self, id: Id) -> Option<&Type> {
		self.types.resolve(id.into().id())
	}

	/// Return a reference to the [`scale_info`] type registry.
	pub(crate) fn types(&self) -> &PortableRegistry {
		&self.types
	}

	/// Retrieve the storage entry at the location provided. Locations are generated from
	/// [`crate::decoder::StorageDecoder`] calls, and should always exist. It is a user error
	/// to use a different [`Metadata`] instance for obtaining these locations from the instance
	/// used to retrieve storage entry details from them.
	pub(crate) fn storage_entry(&self, loc: StorageLocation) -> StorageEntry<'_> {
		let pallet =
			self.pallet_storage.get(loc.prefix_index).expect("Storage entry with the prefix index given should exist");

		let entry =
			pallet.storage_entries.get(loc.entry_index).expect("Storage entry with the entry index given should exist");

		StorageEntry { prefix: &pallet.prefix, metadata: entry }
	}

	pub fn get_storage_entries(&self) -> impl Iterator<Item = &MetadataPalletStorage> {
		self.pallet_storage.iter()
	}

	/// In order to generate a lookup table to decode storage entries, we need to be able to
	/// iterate over them.
	pub(crate) fn storage_entries(&self) -> impl Iterator<Item = &MetadataPalletStorage> {
		self.pallet_storage.iter()
	}

	/// Given the `u8` variant index of a pallet and call, this returns the pallet name and the call Variant
	/// if found, or `None` if no such call exists at those indexes, or we don't have suitable call data.
	pub(crate) fn call_variant_by_enum_index(
		&self,
		pallet: u8,
		call: u8,
	) -> Option<(&str, &scale_info::Variant<PortableForm>)> {
		self.pallet_calls_by_index.get(pallet).and_then(|p| {
			p.calls.as_ref().and_then(|calls| {
				let type_def_variant = self.get_variant(calls.calls_type_id)?;
				let index = *calls.call_variant_indexes.get(call)?;
				let variant = type_def_variant.variants().get(index)?;
				Some((&*p.name, variant))
			})
		})
	}

	/// A helper function to get hold of a Variant given a type ID, or None if it's not found.
	fn get_variant(&self, ty: ScaleInfoTypeId) -> Option<&TypeDefVariant> {
		self.types.resolve(ty.id()).and_then(|ty| match ty.type_def() {
			scale_info::TypeDef::Variant(variant) => Some(variant),
			_ => None,
		})
	}
}

#[derive(Debug)]
pub struct MetadataPalletStorage {
	/// The storage prefix (normally identical to the pallet name,
	/// although they are distinct values in the metadata).
	prefix: String,
	/// Details for each storage entry, in a readonly array so
	/// that we can rely on the indexes not changing.
	storage_entries: ReadonlyArray<StorageEntryMetadata>,
}

impl MetadataPalletStorage {
	pub fn prefix(&self) -> &str {
		&self.prefix
	}
	pub fn entries(&self) -> impl Iterator<Item = &StorageEntryMetadata> {
		self.storage_entries.iter()
	}
}

#[derive(Debug)]
struct MetadataPalletCalls {
	/// The pallet name.
	name: String,
	/// Metadata may not contain call information. If it does,
	/// it'll be here.
	calls: Option<MetadataCalls>,
}

#[derive(Debug)]
struct MetadataCalls {
	/// This allows us to find the type information corresponding to
	/// the call in the [`PortableRegistry`]/
	calls_type_id: ScaleInfoTypeId,
	/// This allows us to map a u8 enum index to the correct call variant
	/// from the calls type, above. The variant contains information on the
	/// fields and such that the call has.
	call_variant_indexes: U8Map<usize>,
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

/// An opaque struct that can be used to obtain details for a specific
/// storage entry via [`Metadata::storage_entry`]. Used internally by
/// our storage decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StorageLocation {
	pub prefix_index: usize,
	pub entry_index: usize,
}

pub(crate) struct StorageEntry<'a> {
	pub prefix: &'a str,
	pub metadata: &'a StorageEntryMetadata,
}
