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
// #[deny(unused)]

mod error;

use core_v14::{metadata::runtime_metadata_version, Decoder as TypeInfoDecoder, Metadata as DesubMetadata};
use frame_metadata::RuntimeMetadataPrefixed;
use desub_legacy::{
	decoder::{Chain, Decoder as LegacyDecoder, Metadata as LegacyDesubMetadata},
	RustTypeMarker, TypeDetective,
};
use std::{collections::HashMap, marker::PhantomData, convert::TryInto};
use codec::{Encode, Decode};

#[cfg(feature = "polkadot-js")]
use extras::TypeResolver as PolkadotJsResolver;

pub use self::error::Error;

/// Struct That implements TypeDetective but refuses to resolve anything
/// that is not of metadata v14+.
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

type SpecVersion = u32;

pub struct Decoder<T: TypeDetective> {
	legacy_decoder: LegacyDecoder,
	decoder: HashMap<SpecVersion, TypeInfoDecoder>,
	_marker: PhantomData<T>,
}

impl<T: TypeDetective> Decoder<T> {
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		let legacy_decoder = LegacyDecoder::new(types, chain);
		let decoder = HashMap::new();
		Self { legacy_decoder, decoder, _marker: PhantomData }
	}

	pub fn register_version(&mut self, version: SpecVersion, metadata: &[u8]) -> Result<(), Error> {
		let metadata: RuntimeMetadataPrefixed = Decode::decode(&metadata)?;
		if runtime_metadata_version(&metadata.1) >= 14 {
				self.decoder.insert(version, DesubMetadata::from_runtime_metadata(metadata.1));
		} else {
			self.legacy_decoder.register_version(version, LegacyDesubMetadata::from_runtime_metadata(metadata.1)?)?;
		}
		Ok(())
	}
}

pub struct InfoDecoder(Decoder<NoLegacyTypes>);
impl Default for InfoDecoder {
	fn default() -> Self {
		let decoder = Decoder::new(NoLegacyTypes, Chain::Custom("none".to_string()));
		Self(decoder)
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
}
