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

use super::{Metadata, MetadataCall, MetadataError, MetadataExtrinsic, MetadataPallet};
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
		let mut calls = HashMap::new();
		if let Some(call_md) = pallet.calls {
			// Get the type representing the variant of available calls:
			let call_ty_id = call_md.ty.id();
			let call_ty = registry.resolve(call_ty_id).ok_or(MetadataError::TypeNotFound(call_ty_id))?;

			// Expect that type to be a variant:
			let call_ty_def = call_ty.type_def();
			let call_variant = match call_ty_def {
				scale_info::TypeDef::Variant(variant) => variant,
				_ => {
					return Err(MetadataError::ExpectedVariantType { got: format!("{:?}", call_ty_def) });
				}
			};

			// Treat each variant as a function call and push to our calls list
			for variant in call_variant.variants() {
				// Allow case insensitive matching; lowercase the name:
				let name = variant.name().to_ascii_lowercase();
				let args = variant.fields().iter().map(|field| *field.ty()).collect();

				calls.insert(variant.index(), MetadataCall { name, args });
			}
		}

		pallets.insert(pallet.index, MetadataPallet { name: pallet.name, calls });
	}

	Ok(Metadata { pallets, extrinsic, types: registry })
}
