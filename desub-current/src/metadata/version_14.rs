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

use super::u8_map::U8Map;
use super::{Metadata, MetadataCalls, MetadataError, MetadataExtrinsic, MetadataPalletCalls, MetadataPalletStorage};
use frame_metadata::RuntimeMetadataV14;

/// Decode V14 metadata into our general Metadata struct
pub fn decode(meta: RuntimeMetadataV14) -> Result<Metadata, MetadataError> {
	let registry = meta.types;
	let mut pallet_calls_by_index = U8Map::new();
	let mut pallet_storage = Vec::new();

	// Gather some details about the extrinsic itself:
	let extrinsic =
		MetadataExtrinsic { version: meta.extrinsic.version, signed_extensions: meta.extrinsic.signed_extensions };

	// Gather information about the calls/storage in use:
	for pallet in meta.pallets {
		// capture the call information in this pallet:
		let calls = pallet
			.calls
			.map(|call_md| {
				// Get the type representing the variant of available calls:
				let calls_type_id = call_md.ty;
				let calls_type = registry
					.resolve(calls_type_id.id())
					.ok_or_else(|| MetadataError::TypeNotFound(calls_type_id.id()))?;

				// Expect that type to be a variant:
				let calls_type_def = calls_type.type_def();
				let calls_variant = match calls_type_def {
					scale_info::TypeDef::Variant(variant) => variant,
					_ => {
						return Err(MetadataError::ExpectedVariantType { got: format!("{:?}", calls_type_def) });
					}
				};

				// Store the mapping from u8 index to variant slice index for quicker decode lookup:
				let call_variant_indexes =
					calls_variant.variants().iter().enumerate().map(|(idx, v)| (v.index(), idx)).collect();

				Ok(MetadataCalls { calls_type_id, call_variant_indexes })
			})
			.transpose()?;
		pallet_calls_by_index.insert(pallet.index, MetadataPalletCalls { name: pallet.name, calls });

		// Capture the storage information in this pallet:
		if let Some(storage_metadata) = pallet.storage {
			pallet_storage.push(MetadataPalletStorage {
				prefix: storage_metadata.prefix,
				storage_entries: storage_metadata.entries.into(),
			});
		}
	}

	Ok(Metadata { pallet_calls_by_index, pallet_storage: pallet_storage.into(), extrinsic, types: registry })
}
