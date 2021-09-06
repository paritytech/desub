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

// taken directly and modified from substrate-subxt:
// https://github.com/paritytech/substrate-subxt

//! A generic metadata structure that delegates decoding of metadata to its
//! native metadata version/structure in substrate runtime.
//! Everything is converted to a generalized representation of the metadata via the
//! `Metadata` struct
//!
//! # Note
//! Must be updated whenever the metadata version is updated
//! by adding a 'version_xx' file

#[cfg(test)]
pub mod test_suite;
mod version_08;
mod version_09;
mod version_10;
mod version_11;
mod version_12;
mod versions;

pub use frame_metadata::decode_different::DecodeDifferent;

use super::storage::{StorageInfo, StorageLookupTable};
use crate::RustTypeMarker;
use codec::{Decode, Encode, EncodeAsRef, HasCompact};
use codec1::Decode as Decode1;
// use codec411::Decode as OldDecode;
use primitives::{storage::StorageKey, twox_128};
use serde::{Deserialize, Serialize};

use std::{
	collections::{HashMap, HashSet},
	convert::TryInto,
	fmt,
	marker::PhantomData,
	str::FromStr,
	sync::Arc,
};
use thiserror::Error;

/// Newtype struct around a Vec<u8> (vector of bytes)
#[derive(Clone)]
pub struct Encoded(pub Vec<u8>);

impl Encode for Encoded {
	fn encode(&self) -> Vec<u8> {
		self.0.to_owned()
	}
}

#[allow(dead_code)]
pub fn compact<T: HasCompact>(t: T) -> Encoded {
	let encodable: <<T as HasCompact>::Type as EncodeAsRef<'_, T>>::RefType = From::from(&t);
	Encoded(encodable.encode())
}

#[derive(Debug, Clone, Error)]
pub enum MetadataError {
	#[error("Module {0} not found")]
	ModuleNotFound(String),
	#[error("Call {0} not found")]
	CallNotFound(&'static str),
	#[error("Module index {0} not found")]
	ModuleIndexNotFound(ModuleIndex),
	#[error("Storage {0} not found")]
	StorageNotFound(&'static str),
	#[error("StorageType Error")]
	StorageTypeError,
	#[error("MapValueType Error")]
	MapValueTypeError,
}

#[derive(Debug, Clone, derive_more::Display)]
pub enum ModuleIndex {
	Call(u8),
	Storage(u8),
	Event(u8),
}

#[derive(Clone, Debug, PartialEq)]
/// Metadata struct encompassing calls, storage, and events
pub struct Metadata {
	/// Hashmap of Modules (name -> module-specific metadata)
	modules: HashMap<String, Arc<ModuleMetadata>>,
	/// modules by their index in the event enum
	modules_by_event_index: HashMap<u8, String>,
	/// modules by their index in the Call Enum
	modules_by_call_index: HashMap<u8, String>,
	/// Optional extrinsic metadata. Only chains which use meta
	/// version 11+ support this.
	extrinsics: Option<ExtrinsicMetadata>,
}

impl From<Vec<u8>> for Metadata {
	fn from(bytes: Vec<u8>) -> Metadata {
		Metadata::new(bytes.as_slice())
	}
}

impl From<&[u8]> for Metadata {
	fn from(bytes: &[u8]) -> Metadata {
		Metadata::new(bytes)
	}
}

impl From<&Metadata> for Metadata {
	fn from(meta: &Metadata) -> Metadata {
		meta.clone()
	}
}

impl Metadata {
	/// Create a new Metadata type from raw encoded bytes
	///
	/// # Panics
	/// Panics is the metadata version is not supported,
	/// or the versiondebug is invalid
	///
	/// Panics if decoding into metadata prefixed fails
	pub fn new(bytes: &[u8]) -> Self {
		// Runtime metadata is a tuple struct with the following fields:
		// RuntimeMetadataPrefixed(u32, RuntimeMetadata)
		// this means when it's SCALE encoded, the first four bytes
		// are the 'u32' prefix, and since `RuntimeMetadata` is an enum,
		// the first byte is the index of the enum item.
		// Since RuntimeMetadata is versioned starting from 0, this also corresponds to
		// the Metadata version
		let version = bytes[4];

		match version {
			/* 0x07 => {
				let meta: runtime_metadata07::RuntimeMetadataPrefixed =
					OldDecode::decode(&mut &*bytes).expect("Decode failed");
				meta.try_into().expect("Conversion failed")
			} */
			0x08 => {
				log::debug!("Metadata V8");
				let meta: runtime_metadata08::RuntimeMetadataPrefixed =
					Decode1::decode(&mut &*bytes).expect("Decode failed");
				meta.try_into().expect("Conversion failed")
			}
			0x09 => {
				log::debug!("Metadata V9");
				let meta: runtime_metadata09::RuntimeMetadataPrefixed =
					Decode1::decode(&mut &*bytes).expect("Decode Failed");
				meta.try_into().expect("Conversion Failed")
			}
			0xA => {
				log::debug!("Metadata V10");
				let meta: runtime_metadata10::RuntimeMetadataPrefixed =
					Decode1::decode(&mut &*bytes).expect("Decode failed");
				meta.try_into().expect("Conversion failed")
			}
			0xB => {
				log::debug!("Metadata V11");
				let meta: runtime_metadata11::RuntimeMetadataPrefixed =
					Decode1::decode(&mut &*bytes).expect("Decode failed");
				meta.try_into().expect("Conversion failed")
			}
			0xC => {
				log::debug!("Metadata V12");
				let meta: frame_metadata::RuntimeMetadataPrefixed =
					Decode::decode(&mut &*bytes).expect("Decode failed");
				meta.try_into().expect("Conversion failed")
			}
			0xD => {
				log::debug!("Metadata V13");
				let meta: frame_metadata::RuntimeMetadataPrefixed =
					Decode::decode(&mut &*bytes).expect("decode failed");
				meta.try_into().expect("Conversion failed")
			}
			0xE => {
				log::debug!("Metadata V14");
				let meta: frame_metadata::RuntimeMetadataPrefixed =
					Decode::decode(&mut &*bytes).expect("decode failed");
				meta.try_into().expect("Conversion failed")
			}
			/* TODO remove panics */
			e => panic!("substrate metadata version {} is unknown, invalid or unsupported", e),
		}
	}

	/// returns an iterate over all Modules
	pub fn modules(&self) -> impl Iterator<Item = &ModuleMetadata> {
		self.modules.values().map(|v| v.as_ref())
	}

	/// returns a weak reference to a module from it's name
	pub fn module<S>(&self, name: S) -> Result<Arc<ModuleMetadata>, MetadataError>
	where
		S: ToString,
	{
		let name = name.to_string();
		self.modules.get(&name).ok_or(MetadataError::ModuleNotFound(name)).map(|m| (*m).clone())
	}

	pub fn signed_extensions(&self) -> Option<&[RustTypeMarker]> {
		self.extrinsics.as_ref().map(|e| e.extensions.as_slice())
	}

	/// Check if a module exists
	pub fn module_exists<S>(&self, name: S) -> bool
	where
		S: ToString,
	{
		let name = name.to_string();
		self.modules.get(&name).is_some()
	}

	/// get the name of a module given it's event index
	pub fn module_name(&self, module_index: u8) -> Result<String, MetadataError> {
		self.modules_by_event_index
			.get(&module_index)
			.cloned()
			.ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Event(module_index)))
	}

	/// get a module by it's index
	pub fn module_by_index(&self, module_index: ModuleIndex) -> Result<Arc<ModuleMetadata>, MetadataError> {
		Ok(match module_index {
			ModuleIndex::Call(i) => {
				let name = self
					.modules_by_call_index
					.get(&i)
					.ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Call(i)))?;
				self.modules.get(name).ok_or_else(|| MetadataError::ModuleNotFound(name.to_string()))?.clone()
			}
			ModuleIndex::Event(i) => {
				let name = self
					.modules_by_event_index
					.get(&i)
					.ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Event(i)))?;
				self.modules.get(name).ok_or_else(|| MetadataError::ModuleNotFound(name.to_string()))?.clone()
			}
			ModuleIndex::Storage(_) => {
				// TODO remove panics
				panic!("No storage index stored")
			}
		})
	}

	/// Returns a hashmap of a Hash -> StorageMetadata
	/// Hash is prefix of storage entries in metadata
	pub fn storage_lookup_table(&self) -> StorageLookupTable {
		let mut lookup = HashMap::new();
		for module in self.modules.values() {
			for (_name, storage_meta) in module.storage.iter() {
				let key = Self::generate_key(&storage_meta.prefix);
				lookup.insert(key, StorageInfo::new(storage_meta.clone(), module.clone()));
			}
		}
		StorageLookupTable::new(lookup)
	}

	fn generate_key<S: AsRef<str>>(prefix: S) -> Vec<u8> {
		prefix.as_ref().split_ascii_whitespace().map(|s| twox_128(s.as_bytes()).to_vec()).flatten().collect()
	}

	/// print out a detailed but human readable description of the module
	/// metadata
	pub fn detailed_pretty(&self) -> String {
		let mut string = String::new();
		for (name, module) in &self.modules {
			string.push_str(name.as_str());
			string.push('\n');
			for (storage, meta) in &module.storage {
				string.push_str(" S  ");
				string.push_str(storage.as_str());
				string.push_str(format!(" TYPE {:?}", meta.ty).as_str());
				string.push_str(format!(" MOD {:?}", meta.modifier).as_str());
				string.push('\n');
			}
			for call in module.calls.keys() {
				string.push_str(" C  ");
				string.push_str(call.as_str());
				string.push('\n');
			}
			for event in module.events.values() {
				string.push_str(" E  ");
				string.push_str(event.name.as_str());
				string.push('\n');
			}
		}
		string
	}

	/// print out a human readable description of the metadata
	pub fn pretty(&self) -> String {
		let mut string = String::new();
		for (name, module) in &self.modules {
			string.push_str(name.as_str());
			string.push('\n');
			for storage in module.storage.keys() {
				string.push_str(" s  ");
				string.push_str(storage.as_str());
				string.push('\n');
			}
			for call in module.calls.values() {
				string.push_str(" c  ");
				string.push_str(&call.to_string());
				string.push('\n');
			}
			for event in module.events.values() {
				string.push_str(" e  ");
				string.push_str(event.name.as_str());
				string.push('\n');
			}
		}
		string
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtrinsicMetadata {
	version: u8,
	extensions: Vec<RustTypeMarker>,
}

impl ExtrinsicMetadata {
	pub fn new(version: u8, extensions: Vec<RustTypeMarker>) -> Self {
		Self { version, extensions }
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMetadata {
	/// index of the module within StorageMetadata 'Entries'
	index: u8,
	/// name of the module
	name: String,
	/// Name of storage entry -> Metadata of storage entry
	pub storage: HashMap<String, StorageMetadata>,
	/// Calls in the module, CallName -> encoded calls
	calls: HashMap<String, CallMetadata>,
	events: HashMap<u8, ModuleEventMetadata>,
	// constants
}

impl ModuleMetadata {
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Return a storage entry by its key
	pub fn storage(&self, key: &'static str) -> Result<&StorageMetadata, MetadataError> {
		self.storage.get(key).ok_or(MetadataError::StorageNotFound(key))
	}

	/// an iterator over all possible events for this module
	pub fn events(&self) -> impl Iterator<Item = &ModuleEventMetadata> {
		self.events.values()
	}

	// TODO Transfer to Subxt
	/// iterator over all possible calls in this module
	pub fn calls(&self) -> impl Iterator<Item = &CallMetadata> {
		self.calls.values()
	}

	/// iterator over all storage keys in this module
	pub fn storage_keys(&self) -> impl Iterator<Item = (&String, &StorageMetadata)> {
		self.storage.iter()
	}

	/// get an event by its index in the module
	pub fn event(&self, index: u8) -> Result<&ModuleEventMetadata, MetadataError> {
		self.events.get(&index).ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Event(index)))
	}

	pub fn call(&self, index: u8) -> Result<&CallMetadata, MetadataError> {
		self.calls().find(|c| c.index == index).ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Call(index)))
	}
}

#[derive(Clone, Debug, PartialEq)]
/// Metadata for Calls in Substrate
pub struct CallMetadata {
	/// Name of the function of the call
	name: String,
	/// encoded byte index of call
	index: u8,
	/// Arguments that the function accepts
	arguments: Vec<CallArgMetadata>,
}

impl CallMetadata {
	/// Returns an iterator for all arguments for this call
	pub fn arguments(&self) -> impl Iterator<Item = &CallArgMetadata> {
		self.arguments.iter()
	}
	pub fn name(&self) -> String {
		self.name.clone()
	}
}

impl fmt::Display for CallMetadata {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut arg_str = String::from("");
		for a in self.arguments.iter() {
			arg_str.push_str(&format!("{}, ", a));
		}
		write!(f, "fn {}({})", self.name, arg_str)
	}
}

#[derive(Clone, Debug, PartialEq)]
/// Metadata for Function Arguments to a Call
pub struct CallArgMetadata {
	/// name of argument
	pub name: String,
	/// Type of the Argument
	pub ty: RustTypeMarker,
}

impl fmt::Display for CallArgMetadata {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.name, self.ty)
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum StorageHasher {
	Blake2_128,
	Blake2_256,
	Blake2_128Concat,
	Twox128,
	Twox256,
	Twox64Concat,
	Identity,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StorageType {
	Plain(RustTypeMarker),
	Map {
		hasher: StorageHasher,
		key: RustTypeMarker,
		value: RustTypeMarker,
		unused: bool,
	},
	DoubleMap {
		hasher: StorageHasher,
		key1: RustTypeMarker,
		key2: RustTypeMarker,
		value: RustTypeMarker,
		key2_hasher: StorageHasher,
	},
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum StorageEntryModifier {
	Optional,
	Default,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StorageMetadata {
	prefix: String,
	modifier: StorageEntryModifier,
	pub ty: StorageType,
	default: Vec<u8>,
	documentation: Vec<String>,
}

impl StorageMetadata {
	pub fn prefix(&self) -> &str {
		&self.prefix
	}
}

#[derive(Clone, Debug)]
pub struct StorageMap<K, V> {
	_marker: PhantomData<K>,
	prefix: Vec<u8>,
	hasher: StorageHasher,
	default: V,
}

impl<K: Encode, V: Decode + Clone> StorageMap<K, V> {
	pub fn key(&self, key: K) -> StorageKey {
		let mut bytes = self.prefix.clone();
		bytes.extend(key.encode());
		let hash = match self.hasher {
			StorageHasher::Blake2_128 => primitives::blake2_128(&bytes).to_vec(),
			StorageHasher::Blake2_256 => primitives::blake2_256(&bytes).to_vec(),
			StorageHasher::Blake2_128Concat => primitives::blake2_128(&bytes).to_vec(),
			StorageHasher::Twox128 => primitives::twox_128(&bytes).to_vec(),
			StorageHasher::Twox256 => primitives::twox_256(&bytes).to_vec(),
			StorageHasher::Twox64Concat => primitives::twox_64(&bytes).to_vec(),
			// TODO figure out which substrate hash function is the 'identity' function
			StorageHasher::Identity => panic!("Unkown Hash"),
		};
		StorageKey(hash)
	}

	pub fn default(&self) -> V {
		self.default.clone()
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleEventMetadata {
	pub name: String,
	pub(crate) arguments: HashSet<EventArg>,
}

impl ModuleEventMetadata {
	pub fn arguments(&self) -> Vec<EventArg> {
		self.arguments.iter().cloned().collect()
	}
}

/// Naive representation of event argument types, supports current set of
/// substrate EventArg types. If and when Substrate uses `type-metadata`, this
/// can be replaced.
///
/// Used to calculate the size of a instance of an event variant without having
/// the concrete type, so the raw bytes can be extracted from the encoded
/// `Vec<EventRecord<E>>` (without `E` defined).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventArg {
	Primitive(String),
	Vec(Box<EventArg>),
	Tuple(Vec<EventArg>),
}

impl FromStr for EventArg {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with("Vec<") {
			if s.ends_with('>') {
				Ok(EventArg::Vec(Box::new(s[4..s.len() - 1].parse()?)))
			} else {
				Err(Error::InvalidEventArg(s.to_string(), "Expected closing `>` for `Vec`"))
			}
		} else if s.starts_with('(') {
			if s.ends_with(')') {
				let mut args = Vec::new();
				for arg in s[1..s.len() - 1].split(',') {
					let arg = arg.trim().parse()?;
					args.push(arg)
				}
				Ok(EventArg::Tuple(args))
			} else {
				Err(Error::InvalidEventArg(s.to_string(), "Expecting closing `)` for tuple"))
			}
		} else {
			Ok(EventArg::Primitive(s.to_string()))
		}
	}
}

impl EventArg {
	/// Returns all primitive types for this EventArg
	pub fn primitives(&self) -> Vec<String> {
		match self {
			EventArg::Primitive(p) => vec![p.clone()],
			EventArg::Vec(arg) => arg.primitives(),
			EventArg::Tuple(args) => {
				let mut primitives = Vec::new();
				for arg in args {
					primitives.extend(arg.primitives())
				}
				primitives
			}
		}
	}
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("Invalid Prefix")]
	InvalidPrefix,
	#[error(" Invalid Version")]
	InvalidVersion,
	#[error("Expected Decoded")]
	ExpectedDecoded,
	#[error("Invalid Event {0}:{1}")]
	InvalidEventArg(String, &'static str),
	#[error("Invalid Type {0}")]
	InvalidType(String),
}

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	fn should_generate_correct_key() {
		let first_key = Metadata::generate_key("System Account");

		let mut key = twox_128("System".as_bytes()).to_vec();
		key.extend(twox_128("Account".as_bytes()).to_vec());
		assert_eq!(first_key, key);
	}
}
