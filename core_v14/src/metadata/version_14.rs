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

use super::{Metadata, MetadataCalls, MetadataError, MetadataExtrinsic, MetadataPallet};
use frame_metadata::RuntimeMetadataV14;
use std::collections::HashMap;

/// Decode V14 metadata into our general Metadata struct
pub fn decode(meta: RuntimeMetadataV14) -> Result<Metadata, MetadataError> {
	let registry = meta.types;
	let mut pallets = HashMap::new();

	// Gather some details about the extrinsic itself:
	let extrinsic =
		MetadataExtrinsic { version: meta.extrinsic.version, signed_extensions: meta.extrinsic.signed_extensions };

	// Gather information about the pallets in use:
	for pallet in meta.pallets {
		let calls = pallet
			.calls
			.map(|call_md| {
				let mut call_variant_indexes = HashMap::new();

				// Get the type representing the variant of available calls:
				let calls_type_id = call_md.ty;
				let calls_type =
					registry.resolve(calls_type_id.id()).ok_or_else(|| MetadataError::TypeNotFound(calls_type_id.id()))?;

				// Expect that type to be a variant:
				let calls_type_def = calls_type.type_def();
				let calls_variant = match calls_type_def {
					scale_info::TypeDef::Variant(variant) => variant,
					_ => {
						return Err(MetadataError::ExpectedVariantType { got: format!("{:?}", calls_type_def) });
					}
				};

				// Store the mapping from u8 index to variant slice index or quicker decode lookup:
				for (idx, variant) in calls_variant.variants().iter().enumerate() {
					call_variant_indexes.insert(variant.index(), idx);
				}

				Ok(MetadataCalls { calls_type_id, call_variant_indexes })
			})
			.transpose()?;

		pallets.insert(pallet.index, MetadataPallet { name: pallet.name, calls });
	}

	Ok(Metadata { pallets, extrinsic, types: registry })
}
