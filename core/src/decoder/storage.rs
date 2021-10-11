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

use super::metadata::{ModuleMetadata, StorageHasher, StorageMetadata};
use crate::RustTypeMarker;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::substrate_types::SubstrateType;

#[derive(Debug, Clone, PartialEq)]
pub struct StorageInfo {
	pub meta: StorageMetadata,
	pub module: Arc<ModuleMetadata>,
}

impl StorageInfo {
	pub fn new(meta: StorageMetadata, module: Arc<ModuleMetadata>) -> Self {
		Self { meta, module }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageLookupTable {
	table: HashMap<Vec<u8>, StorageInfo>,
}

impl StorageLookupTable {
	pub fn new(map: HashMap<Vec<u8>, StorageInfo>) -> Self {
		Self {
			table: map,
			// module: module.clone()
		}
	}

	/// Returns the StorageMetadata given the `prefix` of a StorageKey.
	pub fn lookup(&self, prefix: &[u8]) -> Option<&StorageInfo> {
		self.table.get(prefix)
	}

	pub fn meta_for_key(&self, key: &[u8]) -> Option<&StorageInfo> {
		let key = self.table.keys().find(|&k| &key[..k.len()] == k.as_slice());
		key.map(|k| self.lookup(k)).flatten()
	}

	pub fn extra_key_data<'a>(&self, key: &'a [u8]) -> Option<&'a [u8]> {
		let k = self.table.keys().find(|k| &key[..k.len()] == k.as_slice());

		k.map(|k| &key[k.len()..])
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StorageKeyData {
	Map {
		hasher: StorageHasher,
		/// hashed and scale-encoded key
		key: Vec<u8>,
		key_type: RustTypeMarker,
	},
	DoubleMap {
		hasher: StorageHasher,
		/// hashed and scale-encoded key
		key1: Vec<u8>,
		key1_type: RustTypeMarker,
		/// hashed and scale-encoded key
		key2: Vec<u8>,
		key2_type: RustTypeMarker,
		key2_hasher: StorageHasher,
	},
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageKey {
	pub module: String,
	pub prefix: String,
	pub extra: Option<StorageKeyData>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageValue(SubstrateType);

impl StorageValue {
	pub fn new(val: SubstrateType) -> Self {
		Self(val)
	}

	pub fn ty(&self) -> &SubstrateType {
		&self.0
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GenericStorage {
	key: StorageKey,
	value: Option<StorageValue>,
}

impl GenericStorage {
	pub fn new(key: StorageKey, value: Option<StorageValue>) -> Self {
		Self { key, value }
	}

	pub fn key(&self) -> &StorageKey {
		&self.key
	}

	pub fn value(&self) -> Option<&StorageValue> {
		self.value.as_ref()
	}
}
