// Copyright 2021 Parity Technologies (UK) Ltd.
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


use desub::{decoder::Decoder as DecoderOld, TypeDetective};
use desub_v14::{Decoder as DecoderNew, Metadata as MetadataNew};
use desub::decoder::Chain;
use std::{collections::HashMap, convert::TryInto};
use anyhow::{Error, anyhow};

use crate::app::SpecVersion;

pub struct Decoder {
	old: DecoderOld,
	new: HashMap<u32, DecoderNew>,
}

impl Decoder {
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		Self { old: DecoderOld::new(types, chain), new: HashMap::new() }
	}

	pub fn register_version(&mut self, version: SpecVersion, meta: &[u8]) -> Result<(), Error> {
		log::debug!("Registering version {}", version);
		let new = MetadataNew::from_bytes(meta);
		if let Err(e) = new {
			log::debug!("{}", e);
			self.old.register_version(version.try_into()?, meta.try_into()?)?;
		} else {
			self.new.insert(version.try_into()?, DecoderNew::with_metadata(new?));
		};
		Ok(())
	}

	// Decodes extrinsics and serializes to String
	pub fn decode_extrinsics(&self, version: SpecVersion, mut data: &[u8]) -> Result<String, Error> {
		if self.is_version_new(version) {
			log::debug!("DECODING NEW");
			let decoder = self.new.get(&version.try_into()?).ok_or_else(|| anyhow!("version {} not found for new decoder", version))?;
			match decoder.decode_extrinsics(&mut data) {
				Ok(v) => Ok(format!("{:#?}", v)),
				Err(e) => Err(e.1.into())
			}
		} else {
			log::debug!("DECODING OLD");
			let ext = self.old.decode_extrinsics(version.try_into()?, data)?;
			Ok(serde_json::to_string_pretty(&ext)?)
		}
	}

	fn is_version_new(&self, version: SpecVersion) -> bool {
		self.new.contains_key(&(version as u32))
	}

	pub fn has_version(&self, version: SpecVersion) -> bool {
		self.new.contains_key(&(version as u32)) || self.old.has_version(version as u32)
	}
}
