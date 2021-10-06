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

use codec::{ Decode };
use frame_metadata::{
	RuntimeMetadataPrefixed,
	RuntimeMetadata
};
use super::version_14;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
    #[error("Cannot decode bytes into metadata: {0}")]
    DecodeError(#[from] DecodeError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeError {
    #[error("metadata version {0} is not supported")]
    UnsupportedVersion(u8),
    #[error("{0}")]
    DecodeError(#[from] codec::Error),
	#[error("unexpected type; expecting a Variant type, but got {got}")]
	ExpectedVariantType { got: String }
}

pub struct Metadata {
	pub (crate) pallets: Vec<MetadataPallet>
}

impl Metadata {
	pub fn new(bytes: &[u8]) -> Result<Self, MetadataError> {
        log::trace!("Decoding metadata");
        let meta = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &*bytes)
            .map_err(|e| MetadataError::DecodeError(e.into()))?;

		match meta {
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V0(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(0)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V1(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(1)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V2(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(2)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V3(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(3)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V4(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(4)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V5(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(5)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V6(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(6)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V7(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(7)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V8(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(8)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V9(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(9)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V10(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(10)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V11(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(11)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V12(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(12)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V13(_)) => {
				Err(MetadataError::DecodeError(DecodeError::UnsupportedVersion(13)))
			},
			RuntimeMetadataPrefixed(_, RuntimeMetadata::V14(meta_v14)) => {
				log::trace!("V14 metadata found.");
				version_14::decode(meta_v14).map_err(|e| e.into())
			}
		}
	}
}

pub type TypeDef = scale_info::TypeDef<scale_info::form::PortableForm>;
pub type Type = scale_info::Type<scale_info::form::PortableForm>;

pub struct MetadataPallet {
	name: String,
	calls: Vec<MetadataCall>
}

pub struct MetadataCall {
	name: String,
	args: Vec<TypeDef>
}

// use scale_info::PortableRegistry;
// use std::io::Write;
// /// Output a human readable name for the type provided
// fn type_name<W: Write>(ty: &Type, registry: &PortableRegistry, w: &mut W) -> Result<(), std::io::Error> {
// 	let def = ty.type_def();

// 	let to_type = |ty: &<scale_info::form::PortableForm as scale_info::form::Form>::Type | {
// 		registry
// 			.resolve(ty.id())
// 			.expect("type ID to exist in registry")
// 	};

// 	match def {
// 		TypeDef::Array(inner) => {
// 			w.write_all(b"[")?;
// 			type_name(to_type(inner.type_param()), registry, w)?;
// 			w.write_all(b"; ")?;
// 			w.write_all(inner.len().to_string().as_bytes())?;
// 			w.write_all(b"]")?;
// 		},
// 		TypeDef::BitSequence(bits) => {
// 			w.write_all(b"BitSequence")?;
// 		},
// 		TypeDef::Compact(inner) => {
// 			w.write_all(b"Compact<")?;
// 			type_name(to_type(inner.type_param()), registry, w)?;
// 			w.write_all(b">")?;
// 		},
// 		TypeDef::Composite(inner) => {
// 			w.write_all(b"{ ")?;
// 			for field in inner.fields() {
// 				if let Some(name) = field.name() {
// 					w.write_all(name.as_bytes())?;
// 					w.write_all(b": ")?;
// 				}
// 				type_name(to_type(field.ty()), registry, w)?;
// 				w.write_all(b", ")?;
// 			}
// 			w.write_all(b" }")?;
// 		},
// 		TypeDef::Primitive(prim) => {
// 			use scale_info::TypeDefPrimitive;
// 			match prim {
// 				TypeDefPrimitive::Bool => w.write_all(b"bool")?,
// 				TypeDefPrimitive::Char => w.write_all(b"char")?,
// 				TypeDefPrimitive::Str => w.write_all(b"str")?,
// 				TypeDefPrimitive::U8 => w.write_all(b"u8")?,
// 				TypeDefPrimitive::U16 => w.write_all(b"u16")?,
// 				TypeDefPrimitive::U32 => w.write_all(b"u32")?,
// 				TypeDefPrimitive::U64 => w.write_all(b"u64")?,
// 				TypeDefPrimitive::U128 => w.write_all(b"u128")?,
// 				TypeDefPrimitive::U256 => w.write_all(b"u256")?,
// 				TypeDefPrimitive::I8 => w.write_all(b"i8")?,
// 				TypeDefPrimitive::I16 => w.write_all(b"i16")?,
// 				TypeDefPrimitive::I32 => w.write_all(b"i32")?,
// 				TypeDefPrimitive::I64 => w.write_all(b"i64")?,
// 				TypeDefPrimitive::I128 => w.write_all(b"i128")?,
// 				TypeDefPrimitive::I256 => w.write_all(b"i256")?,
// 			}
// 		},
// 		TypeDef::Sequence(seq) => {

// 		}
// 	};

// 	w.flush()
// }

/*
//! A generic metadata structure that delegates decoding of metadata to its
//! native metadata version/structure in substrate runtime.
//! Everything is converted to a generalized representation of the metadata via the
//! `Metadata` struct
//!
//! # Note
//! Must be updated whenever the metadata version is updated
//! by adding a 'version_xx' file

pub use frame_metadata::decode_different::DecodeDifferent;

// use super::storage::{StorageInfo, StorageLookupTable};
use crate::substrate_type::SubstrateType;
use codec::{Decode, Encode, EncodeAsRef, HasCompact};
use primitives::{storage::StorageKey, twox_128};
use serde::{Deserialize, Serialize};

use std::{
	collections::{HashMap, HashSet},
	convert::{ TryFrom, TryInto },
	fmt,
	marker::PhantomData,
	str::FromStr,
	sync::Arc,
};

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

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetadataError {
    #[error("Cannot decode bytes into metadata: {0}")]
    DecodeError(#[from] DecodeError),
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

#[derive(Debug, Clone, thiserror::Error)]
enum DecodeError {
    #[error("metadata version {0} is not supported")]
    BadVersion(u8),
    #[error("{0}")]
    DecodeError(#[from] codec::Error)
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
	pub modules_by_call_index: HashMap<u8, String>,
	/// Optional extrinsic metadata. Only chains which use meta
	/// version 11+ support this.
	extrinsics: Option<ExtrinsicMetadata>,
}

impl TryFrom<Vec<u8>> for Metadata {
    type Error = MetadataError;
	fn try_from(bytes: Vec<u8>) -> Result<Metadata, Self::Error> {
		Metadata::new(bytes.as_slice())
	}
}

impl TryFrom<&[u8]> for Metadata {
    type Error = MetadataError;
	fn try_from(bytes: &[u8]) -> Result<Metadata, Self::Error> {
		Metadata::new(bytes)
	}
}

impl<'a> Metadata {
	/// Create a new Metadata type from raw encoded bytes
	///
	/// # Panics
	/// Panics is the metadata version is not supported,
	/// or the versiondebug is invalid
	///
	/// Panics if decoding into metadata prefixed fails
	pub fn new(bytes: &[u8]) -> Result<Self, MetadataError> {
        // We expect to decode to `RuntimeMetadataPrefixed(u32, RuntimeMetadata::V14(..))`.
        // The first 4 bytes (index 0,1,2,3) are therefore the u32, and byte 5 (index 4) is
        // the enum tag byte for `RuntimeMetadata`, and we expect the value 14 here, to align
        // with the V14 metadata we expect.
		let version = bytes[4];

        if version != 14 {
            return Err(MetadataError::DecodeError(DecodeError::BadVersion(version)));
        }

        log::debug!("Decoding V14 Metadata");
        let meta = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &*bytes)
            .map_err(|e| MetadataError::DecodeError(e.into()))?;
        meta.try_into().map_err(|e| )
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

	pub fn signed_extensions(&self) -> Option<&[SubstrateType]> {
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
	pub fn module_by_index(&'a self, module_index: ModuleIndex) -> Result<&'a ModuleMetadata, MetadataError> {
		Ok(match module_index {
			ModuleIndex::Call(i) => {
				let name = self
					.modules_by_call_index
					.get(&i)
					.ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Call(i)))?;
				self.modules.get(name).ok_or_else(|| MetadataError::ModuleNotFound(name.to_string()))?
			}
			ModuleIndex::Event(i) => {
				let name = self
					.modules_by_event_index
					.get(&i)
					.ok_or(MetadataError::ModuleIndexNotFound(ModuleIndex::Event(i)))?;
				self.modules.get(name).ok_or_else(|| MetadataError::ModuleNotFound(name.to_string()))?
			}
			ModuleIndex::Storage(_) => {
				// TODO remove panics
				panic!("No storage index stored")
			}
		})
	}

	// /// Returns a hashmap of a Hash -> StorageMetadata
	// /// Hash is prefix of storage entries in metadata
	// pub fn storage_lookup_table(&self) -> StorageLookupTable {
	// 	let mut lookup = HashMap::new();
	// 	for module in self.modules.values() {
	// 		for (_name, storage_meta) in module.storage.iter() {
	// 			let key = Self::generate_key(&storage_meta.prefix);
	// 			lookup.insert(key, StorageInfo::new(storage_meta.clone(), module.clone()));
	// 		}
	// 	}
	// 	StorageLookupTable::new(lookup)
	// }

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
	extensions: Vec<SubstrateType>,
}

impl ExtrinsicMetadata {
	pub fn new(version: u8, extensions: Vec<SubstrateType>) -> Self {
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
	storage: HashMap<String, StorageMetadata>,
	/// Calls in the module, CallName -> encoded calls
	calls: HashMap<String, CallMetadata>,
	events: HashMap<u8, ModuleEventMetadata>
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
	pub ty: SubstrateType,
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
	Plain(SubstrateType),
	Map {
		hasher: StorageHasher,
		key: SubstrateType,
		value: SubstrateType,
		unused: bool,
	},
	DoubleMap {
		hasher: StorageHasher,
		key1: SubstrateType,
		key2: SubstrateType,
		value: SubstrateType,
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
	type Err = EventArgError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with("Vec<") {
			if s.ends_with('>') {
				Ok(EventArg::Vec(Box::new(s[4..s.len() - 1].parse()?)))
			} else {
				Err(EventArgError::InvalidEventArg(s.to_string(), "Expected closing `>` for `Vec`"))
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
				Err(EventArgError::InvalidEventArg(s.to_string(), "Expecting closing `)` for tuple"))
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

#[derive(thiserror::Error, Debug)]
pub enum EventArgError {
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
*/