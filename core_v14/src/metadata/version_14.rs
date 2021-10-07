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

use super::{ Metadata, MetadataPallet, MetadataCall, MetadataExtrinsic, DecodeError, TypeDef, full_type_name };
use frame_metadata::{ RuntimeMetadataV14 };

/// Decode V14 metadata into our general Metadata struct
pub fn decode(meta: RuntimeMetadataV14) -> Result<Metadata, DecodeError> {
	let registry = meta.types;
	let mut pallets = vec![];

	//// Since a version change could lead to more global changes than
	//// just this, it's debatable that using it is actually useful, versus
	//// just inspecting the version and manually decoding accordingly:
	// let signed_extensions = meta.extrinsic.signed_extensions
	// 	.into_iter()
	// 	.map(|ext| {
	// 		registry
	// 			.resolve(ext.ty.id())
	// 			.ok_or(DecodeError::TypeNotFound(ext.ty.id()))
	// 			.map(|t| t.clone())
	// 	})
	// 	.collect::<Result<_,_>>()?;

	// Gather some details about the extrinsic itself:
	let extrinsic = MetadataExtrinsic {
		version: meta.extrinsic.version,
		// signed_extensions
	};

	// Gather information about the pallets in use:
	for pallet in meta.pallets {
		let mut calls = vec![];
		if let Some(call_md) = pallet.calls {
			// Get the type representing the variant of available calls:
			let call_ty_id = call_md.ty.id();
			let call_ty = registry
				.resolve(call_ty_id)
				.ok_or(DecodeError::TypeNotFound(call_ty_id))?;

			// Expect that type to be a variant:
			let call_ty_def = call_ty.type_def();
			let call_variant = match call_ty_def {
				TypeDef::Variant(variant) => {
					variant
				},
				_ => {
					let name = full_type_name(call_ty, &registry);
					return Err(DecodeError::ExpectedVariantType { got: name })
				}
			};

			// Treat each variant as a function call and push to our calls list
			for variant in call_variant.variants() {
				// Allow case insensitive matching; lowercase the name:
				let name = variant.name().to_ascii_lowercase();
				let args = variant
					.fields()
					.iter()
					.map(|field| {
						let id = field.ty().id();
						registry.resolve(id)
							.ok_or(DecodeError::TypeNotFound(id))
							.map(|t| t.clone())
					})
					.collect::<Result<_,_>>()?;

				calls.push(MetadataCall { name, args });
			}
		}

		pallets.push(MetadataPallet {
			name: pallet.name,
			calls
		})
	}

	Ok(Metadata {
		pallets,
		extrinsic,
		types: registry,
	})
}