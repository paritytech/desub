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

//! Facade crate for decoding data that uses any version of metadata (V8+)

#![forbid(unsafe_code)]
#[deny(unused)]

mod error;

use codec::Decode;
use core_v14::{decoder::Extrinsic, Decoder as TypeInfoDecoder, Metadata as DesubMetadata};
use desub_legacy::{
	decoder::{Decoder as LegacyDecoder, Metadata as LegacyDesubMetadata},
	RustTypeMarker, TypeDetective,
};
use serde_json::Value;
use frame_metadata::RuntimeMetadataPrefixed;
use std::{collections::HashMap, marker::PhantomData};

#[cfg(feature = "polkadot-js")]
use extras::{TypeResolver as PolkadotJsResolver};

#[cfg(feature = "polkadot-js")]
pub use extras::runtimes;
pub use desub_legacy::decoder::Chain;
pub use desub_common::SpecVersion;
pub use self::error::Error;

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

pub struct Decoder<T: TypeDetective> {
	legacy_decoder: LegacyDecoder,
	current_decoder: HashMap<SpecVersion, TypeInfoDecoder>,
	_marker: PhantomData<T>,
}

impl<T: TypeDetective> Decoder<T> {

	/// Create a new general Decoder
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		let legacy_decoder = LegacyDecoder::new(types, chain);
		let current_decoder = HashMap::new();
		Self { legacy_decoder, current_decoder, _marker: PhantomData }
	}

	/// Register a runtime version with the decoder.
	pub fn register_version(&mut self, version: SpecVersion, mut metadata: &[u8]) -> Result<(), Error> {
		let metadata: RuntimeMetadataPrefixed = Decode::decode(&mut metadata)?;
		if &metadata.1.version() >= 14 {
			let meta = DesubMetadata::from_runtime_metadata(metadata.1)?;
			let decoder = TypeInfoDecoder::with_metadata(meta);
			self.current_decoder.insert(version, decoder);
		} else {
			self.legacy_decoder.register_version(version, LegacyDesubMetadata::from_runtime_metadata(metadata.1)?)?;
		}
		Ok(())
	}

	pub fn decode_extrinsics(&self, version: SpecVersion, mut data: &[u8]) -> Result<Value, Error> {
		if self.current_decoder.contains_key(&version) {
			let decoder = self.current_decoder.get(&version).expect("Checked if key is contained; qed");
			match decoder.decode_extrinsics(&mut data) {
				Ok(v) => Ok(serde_json::to_value(&v)?),
				Err((ext, e)) => Err(Error::V14{ source: e, ext: ext.into_iter().map(Extrinsic::into_owned).collect() }),
			}
		} else {
			if !self.legacy_decoder.has_version(&version) {
				return Err(Error::SpecVersionNotFound(version));
			}
			let ext = self.legacy_decoder.decode_extrinsics(version, data)?;
			Ok(serde_json::to_value(&ext)?)
		}
	}

	pub fn has_version(&self, version: &SpecVersion) -> bool {
		self.current_decoder.contains_key(version) || self.legacy_decoder.has_version(version)
	}
}

/// Decoder which does not resolve any types of metadata v13 and below.
/// To be used with newer chains where legacy types are not required.
pub struct InfoDecoder(Decoder<NoLegacyTypes>);
impl InfoDecoder {
	// No sensible default b/c of Chain::Custom("none")
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		let decoder = Decoder::new(NoLegacyTypes, Chain::Custom("none".to_string()));
		Self(decoder)
	}

	pub fn decode_extrinsics(&self, version: SpecVersion, data: &[u8]) -> Result<Value, Error> {
		self.0.decode_extrinsics(version, data)
	}

	pub fn has_version(&self, version: &SpecVersion) -> bool {
		self.0.has_version(version)
	}

	pub fn register_version(&mut self, version: SpecVersion, metadata: &[u8]) -> Result<(), Error> {
		self.0.register_version(version, metadata)
	}
}

/// Decoder that includes all Polkadot JS type definitions in addition to
/// V14 Metadata decoding.
/// This is useful if historical type network data that existed pre-v14 is required.
#[cfg(feature = "polkadot-js")]
pub struct PolkadotJsDecoder(Decoder<PolkadotJsResolver>);

#[cfg(feature = "polkadot-js")]
impl PolkadotJsDecoder {
	pub fn new(chain: Chain) -> Self {
		let types = PolkadotJsResolver::default();
		let decoder = Decoder::new(types, chain);
		Self(decoder)
	}

	pub fn decode_extrinsics(&self, version: SpecVersion, data: &[u8]) -> Result<Value, Error> {
		self.0.decode_extrinsics(version, data)
	}

	pub fn has_version(&self, version: &SpecVersion) -> bool {
		self.0.has_version(version)
	}

	pub fn register_version(&mut self, version: SpecVersion, metadata: &[u8]) -> Result<(), Error> {
		self.0.register_version(version, metadata)
	}
}
