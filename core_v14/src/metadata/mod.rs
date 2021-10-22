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

// We don't expose anything scale-info or parity-scale-codec related outside of
// this crate currently, so no need to expose these either:
pub(crate) type Type = scale_info::Type<PortableForm>;
pub(crate) type TypeDef = scale_info::TypeDef<PortableForm>;
pub(crate) type TypeId = <scale_info::form::PortableForm as scale_info::form::Form>::Type;
pub(crate) type SignedExtensionMetadata = frame_metadata::SignedExtensionMetadata<PortableForm>;

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
#[derive(Debug)]
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

	/// Given the `u8` variant index of a pallet and call, this returns information about
	/// the call if it's fgound, or `None` if it no such call exists at those indexes.
	pub(crate) fn call_by_variant_index(&self, pallet: u8, call: u8) -> Option<(&str, &MetadataCall)> {
		self.pallets.get(&pallet).and_then(|p| {
			let call = p.calls.get(&call)?;
			Some((&*p.name, call))
		})
	}

	/// Return a reference to the type registry. This is used for helping to decode things.
	pub(crate) fn types(&self) -> &PortableRegistry {
		&self.types
	}
}

/// Get the decoded metadata version. At some point `RuntimeMetadataPrefixed` will end up
/// with a `.version()` method to return the version, and then this can be removed.
pub fn runtime_metadata_version(meta: &RuntimeMetadata) -> usize {
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
	calls: HashMap<u8, MetadataCall>,
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
	/// [`TypeId`]'s can be resolved into [`Type`]'s using
	/// [`Metadata::resolve_type`]
	pub fn args(&self) -> &[TypeId] {
		&self.args
	}
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
