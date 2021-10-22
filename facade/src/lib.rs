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


use desub::Decoder as TypeInfoDecoder;
use desub_legacy::{decoder::{Decoder as LegacyDecoder, Chain}, RustTypeMarker, TypeDetective};
#[cfg(feature = "polkadot-js")]
use extras::TypeResolver as PolkadotJsResolver;
use std::{marker::PhantomData, collections::HashMap};

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
	_marker: PhantomData<T>
}

// pub type TypeInfoDecoder = Decoder<NoLegacyTypes>;
// pub type PolkadotJsDecoder = Decoder<PolkadotJsResolver>;

impl<T: TypeDetective> Decoder<T> {
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		let legacy_decoder = LegacyDecoder::new(types, chain);
		let decoder = HashMap::new();
		Self { legacy_decoder, decoder, _marker: PhantomData }
	}
}

struct InfoDecoder(Decoder<NoLegacyTypes>);

impl InfoDecoder {
	pub fn new() -> Self {
		todo!()
	}
}

#[cfg(feature = "polkadot-js")]
struct PolkadotJsDecoder(Decoder<PolkadotJsResolver>);

#[cfg(feature = "polkadot-js")]
impl PolkadotJsDecoder {
	pub fn new() -> Self {
		todo!();
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
