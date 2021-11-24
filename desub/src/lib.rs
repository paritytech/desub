// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

//! Facade crate for decoding data that uses any version of metadata (V8+)

#![forbid(unsafe_code)]
#[deny(unused)]
mod error;
pub mod types;

use codec::Decode;
use desub_current::{
	decoder::{self, Extrinsic},
	Metadata as DesubMetadata,
};
use desub_legacy::{
	decoder::{Decoder as LegacyDecoder, Metadata as LegacyDesubMetadata},
	RustTypeMarker, TypeDetective,
};
use frame_metadata::RuntimeMetadataPrefixed;
use std::collections::HashMap;

#[cfg(feature = "polkadot-js")]
use desub_json_resolver::TypeResolver as PolkadotJsResolver;

pub use self::error::Error;
pub use desub_common::SpecVersion;
#[cfg(feature = "polkadot-js")]
pub use desub_json_resolver::runtimes;
pub use desub_legacy::decoder::Chain;
use types::LegacyOrCurrentExtrinsic;

/// Struct That implements TypeDetective but refuses to resolve anything
/// that is not of metadata v14+.
/// Useful for use with a new chain that does not require historical metadata.
#[derive(Copy, Clone, Debug)]
struct NoLegacyTypes;

impl TypeDetective for NoLegacyTypes {
	fn get(&self, _: &str, _: u32, _: &str, _: &str) -> Option<&RustTypeMarker> {
		None
	}

	fn try_fallback(&self, _: &str, _: &str) -> Option<&RustTypeMarker> {
		None
	}

	fn get_extrinsic_ty(&self, _: &str, _: u32, _: &str) -> Option<&RustTypeMarker> {
		None
	}
}

pub struct Decoder {
	legacy_decoder: LegacyDecoder,
	current_metadata: HashMap<SpecVersion, DesubMetadata>,
}

impl Decoder {
	#[cfg(feature = "polkadot-js")]
	pub fn new(chain: Chain) -> Self {
		let legacy_decoder = LegacyDecoder::new(PolkadotJsResolver::default(), chain);
		let current_metadata = HashMap::new();

		Self { legacy_decoder, current_metadata }
	}

	#[cfg(not(feature = "polkadot-js"))]
	pub fn new() -> Self {
		let legacy_decoder = LegacyDecoder::new(NoLegacyTypes, Chain::Custom("none".to_string()));
		let current_metadata = HashMap::new();

		Self { legacy_decoder, current_metadata }
	}

	/// Create a new general Decoder
	pub fn with_custom_types(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		let legacy_decoder = LegacyDecoder::new(types, chain);
		let current_decoder = HashMap::new();
		Self { legacy_decoder, current_metadata: current_decoder }
	}

	/// Register a runtime version with the decoder.
	pub fn register_version(&mut self, version: SpecVersion, mut metadata: &[u8]) -> Result<(), Error> {
		let metadata: RuntimeMetadataPrefixed = Decode::decode(&mut metadata)?;
		if metadata.1.version() >= 14 {
			let meta = DesubMetadata::from_runtime_metadata(metadata.1)?;
			self.current_metadata.insert(version, meta);
		} else {
			self.legacy_decoder.register_version(version, LegacyDesubMetadata::from_runtime_metadata(metadata.1)?)?;
		}
		Ok(())
	}

	pub fn decode_extrinsics(&self, version: SpecVersion, mut data: &[u8]) -> Result<Vec<LegacyOrCurrentExtrinsic>, Error> {
		if self.current_metadata.contains_key(&version) {
			let metadata = self.current_metadata.get(&version).expect("Checked if key is contained; qed");
			match decoder::decode_extrinsics(metadata, &mut data) {
				Ok(v) => Ok(v.into_iter().map(|e| e.into_owned()).map(LegacyOrCurrentExtrinsic::Current).collect()),
				Err((ext, e)) => {
					Err(Error::V14 { source: e, ext: ext.into_iter().map(Extrinsic::into_owned).collect() })
				}
			}
		} else {
			if !self.legacy_decoder.has_version(&version) {
				return Err(Error::SpecVersionNotFound(version));
			}
			let ext = self.legacy_decoder.decode_extrinsics(version, data)?;
			Ok(ext.into_iter().map(LegacyOrCurrentExtrinsic::Legacy).collect())
		}
	}

	pub fn has_version(&self, version: &SpecVersion) -> bool {
		self.current_metadata.contains_key(version) || self.legacy_decoder.has_version(version)
	}
}
