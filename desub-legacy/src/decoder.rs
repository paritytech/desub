// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version. //
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

//! A serializable/deserializable Decoder used to encode/decode substrate types
//! from compact SCALE encoded byte arrays
//! with special attention paid to generic types in runtime module trait
//! definitions if serialized, can be deserialized. This allows for portability
//! by not needing to import differently-versioned runtimes
//! as long as all the types of the runtime are registered within the decoder
//!
//! Theoretically, one could upload the deserialized decoder JSON to distribute
//! to different applications that need the type data

mod extrinsics;
pub mod metadata;
mod storage;

pub use self::extrinsics::{ExtrinsicArgument, GenericCall, GenericExtrinsic, GenericSignature};
pub use self::storage::{GenericStorage, StorageInfo, StorageKey, StorageKeyData, StorageLookupTable, StorageValue};

#[cfg(test)]
pub use self::metadata::test_suite;

pub use self::metadata::{
	CallMetadata, Error as MetadataError, Metadata, ModuleIndex, ModuleMetadata, StorageEntryModifier, StorageHasher,
	StorageType,
};
pub use frame_metadata::v14::StorageEntryType;

use crate::{
	error::Error,
	substrate_types::{self, pallet_democracy, StructField, SubstrateType},
	CommonTypes, RustTypeMarker, TypeDetective,
};
use bitvec::order::Lsb0 as BitOrderLsb0;
use codec::{Compact, CompactLen, Decode, Input};
use desub_common::SpecVersion;
use std::{
	cell::RefCell,
	collections::HashMap,
	convert::TryFrom,
	rc::Rc,
	str::FromStr,
	sync::atomic::{AtomicUsize, Ordering},
};

/// Decoder for substrate types
///
/// hold information about the Runtime Metadata
/// and maps types inside the runtime metadata to self-describing types in
/// type-metadata
#[derive(Debug)]
pub struct Decoder {
	// reference to an item in 'versions' vector
	versions: HashMap<SpecVersion, Metadata>,
	types: Box<dyn TypeDetective>,
	chain: String,
}

impl Clone for Decoder {
	fn clone(&self) -> Self {
		Self { versions: self.versions.clone(), types: dyn_clone::clone_box(&*self.types), chain: self.chain.clone() }
	}
}

/// The type of Entry
/// # Note
///
/// not entirely sure if necessary as of yet
/// however, used for the purpose for narrowing down the context a type is being
/// used in
#[derive(Debug)]
pub enum Entry {
	Call,
	Storage,
	Event,
	Constant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Chain {
	Polkadot,
	Kusama,
	Centrifuge,
	Westend,
	Rococo,
	Custom(String),
}

impl std::fmt::Display for Chain {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Chain::Polkadot => write!(f, "polkadot"),
			Chain::Kusama => write!(f, "kusama"),
			Chain::Centrifuge => write!(f, "centrifuge-chain"),
			Chain::Westend => write!(f, "westend"),
			Chain::Rococo => write!(f, "rococo"),
			Chain::Custom(s) => write!(f, "{}", s),
		}
	}
}

impl FromStr for Chain {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"polkadot" | "dot" => Ok(Chain::Polkadot),
			"kusama" | "ksm" => Ok(Chain::Kusama),
			"westend" | "wnd" => Ok(Chain::Westend),
			"centrifuge" => Ok(Chain::Centrifuge),
			"rococo" => Ok(Chain::Rococo),
			_ => Ok(Chain::Custom(s.to_string())),
		}
	}
}

#[derive(Debug)]
struct Module<'a> {
	// no module, means we are probably decoding a signature.
	// A signature is decoded before we know which module/call the extrinsic actually represents.
	module: Option<&'a ModuleMetadata>,
}

impl<'a> Module<'a> {
	fn new(module: Option<&'a ModuleMetadata>) -> Self {
		Module { module }
	}

	fn set(&mut self, module: &'a ModuleMetadata) {
		self.module = Some(module);
	}

	fn reset(&mut self) {
		self.module = None;
	}

	fn name(&self) -> &'a str {
		self.module.map(ModuleMetadata::name).unwrap_or("runtime")
	}

	fn call(&self, index: u8) -> Result<Option<&'a CallMetadata>, MetadataError> {
		self.module.map(|m| m.call(index)).transpose()
	}
}

#[derive(Debug)]
struct DecodeState<'a> {
	module: Module<'a>,
	call: Rc<RefCell<Option<CallMetadata>>>,
	metadata: &'a Metadata,
	cursor: AtomicUsize,
	spec: SpecVersion,
	data: &'a [u8],
}

impl<'a> DecodeState<'a> {
	fn new(
		module: Option<&'a ModuleMetadata>,
		call: Option<CallMetadata>,
		metadata: &'a Metadata,
		cursor: usize,
		spec: SpecVersion,
		data: &'a [u8],
	) -> Self {
		let call = Rc::new(RefCell::new(call));
		let cursor = AtomicUsize::new(cursor);
		let module = Module::new(module);
		Self { module, call, metadata, cursor, spec, data }
	}

	fn module_name(&'a self) -> &'a str {
		self.module.name()
	}

	/// Loads the module at the current index.
	/// Increments the cursor by 1.
	fn load_module(&mut self) -> Result<(), Error> {
		log::trace!("Loading module in index {}", self.index());
		let module = self
			.metadata
			.module_by_index(ModuleIndex::Call(self.index()))
			.map_err(|e| Error::DetailedMetaFail(e, self.cursor(), hex::encode(self.data)))?;
		self.increment();
		self.module.set(module);
		Ok(())
	}

	// Gets the call at the current index. Increments cursor by 1.
	// Sets the call for the state.
	// Panics if there is no module loaded
	fn call(&self) -> Result<CallMetadata, MetadataError> {
		let call = self.data[self.cursor.load(Ordering::Relaxed)];
		let call = self.module.call(call)?.expect("No module in state");
		self.increment();
		self.call.replace(Some(call.clone()));
		Ok(self.call.borrow().as_ref().expect("Just set call").clone())
	}

	/// Interprets the version at the current byte offset.
	/// Returns whether the extrinsic is signed.
	fn interpret_version(&self) -> bool {
		let version = self.do_index();
		let is_signed = version & 0b1000_0000 != 0;
		let version = version & 0b0111_1111;
		log::trace!("Extrinsic Version: {}", version);
		is_signed
	}

	/// Get the scale length at the current point in time.
	/// Increment cursor accordingly to the length.
	fn scale_length(&mut self) -> Result<usize, Error> {
		let length = Decoder::scale_length(&self.data[self.cursor.load(Ordering::Relaxed)..])?;
		log::trace!("Scale Byte Length {}, actual items: {}", length.1, length.0);
		self.cursor.fetch_add(length.1, Ordering::Relaxed);
		Ok(length.0)
	}

	fn reset(&mut self, data: &'a [u8]) {
		self.data = data;
		self.module.reset();
		self.call.replace(None);
		self.set_cursor(0);
	}

	/// Current value at cursor.
	/// In other words: data\[cursor\]
	fn index(&self) -> u8 {
		self.data[self.cursor()]
	}

	/// Current value at cursor (data\[cursor\]).
	/// Increment the cursor by 1.
	fn do_index(&self) -> u8 {
		let number = self.data[self.cursor.load(Ordering::Relaxed)];
		self.add(1);
		number
	}

	/// Decode a value, automatically incrementing `cursor`
	/// the correct number of bytes.
	fn decode<T: Decode>(&self) -> Result<T, Error> {
		let input = &mut &self.data[self.cursor.load(Ordering::Relaxed)..];
		let remaining_len = input.remaining_len()?.expect("&'a u8 is always Some()");
		let ty = Decode::decode(input)?;
		let after_remaining_len = input.remaining_len()?.expect("&'a u8 is always Some()");
		let inc = remaining_len - after_remaining_len;
		self.add(inc);
		Ok(ty)
	}

	fn add(&self, inc: usize) {
		self.cursor.fetch_add(inc, Ordering::Relaxed);
	}

	fn increment(&self) {
		self.cursor.fetch_add(1, Ordering::Relaxed);
	}

	fn set_cursor(&self, new: usize) {
		self.cursor.store(new, Ordering::Relaxed);
	}

	fn cursor(&self) -> usize {
		self.cursor.load(Ordering::Relaxed)
	}

	/// Prints out a succinct debug snapshot of the current state.
	fn observe(&self, line: u32) {
		let module = self.module.name();
		let cursor = self.cursor.load(Ordering::Relaxed);
		let value_at_cursor = &self.data[cursor];
		let data_at_cursor = &self.data[cursor..];

		log::trace!(
			"line: {}, module = {}, call = {:?}, cursor = {}, data[cursor] = {}, data[cursor..] = {:?}",
			line,
			module,
			self.call.borrow().as_ref().map(|c| c.name()),
			cursor,
			value_at_cursor,
			data_at_cursor,
		)
	}
}

struct ChunkedExtrinsic<'a> {
	data: &'a [u8],
	cursor: usize,
}

impl<'a> ChunkedExtrinsic<'a> {
	/// Create new ChunkedExtrinsic.
	fn new(data: &'a [u8]) -> Self {
		Self { data, cursor: 0 }
	}
}

impl<'a> Iterator for ChunkedExtrinsic<'a> {
	type Item = &'a [u8];
	fn next(&mut self) -> Option<&'a [u8]> {
		let (length, prefix) = Decoder::scale_length(&self.data[self.cursor..]).ok()?;
		let extrinsic = &self.data[(self.cursor + prefix)..(self.cursor + length + prefix)];
		self.cursor += length + prefix;
		Some(extrinsic)
	}
}

impl Decoder {
	/// Create new Decoder with specified types.
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		Self { versions: HashMap::default(), types: Box::new(types), chain: chain.to_string() }
	}

	/// Check if a metadata version has already been registered
	pub fn has_version(&self, version: &SpecVersion) -> bool {
		self.versions.contains_key(version)
	}

	/// Insert a Metadata with Version attached
	/// If version exists, it's corresponding metadata will be updated
	pub fn register_version(&mut self, version: SpecVersion, metadata: Metadata) -> Result<(), Error> {
		self.versions.insert(version, metadata);
		Ok(())
	}

	/// internal api to get metadata from runtime version.
	///
	/// # Note
	/// Returns None if version is nonexistant
	pub fn get_version_metadata(&self, version: SpecVersion) -> Option<&Metadata> {
		self.versions.get(&version)
	}

	fn decode_key_len(&self, key: &[u8], hasher: &StorageHasher) -> Vec<u8> {
		match hasher {
			StorageHasher::Blake2_128 | StorageHasher::Twox128 | StorageHasher::Blake2_128Concat => key[..16].to_vec(),
			StorageHasher::Blake2_256 | StorageHasher::Twox256 => key[..32].to_vec(),
			StorageHasher::Twox64Concat => key[..8].to_vec(),
			StorageHasher::Identity => todo!(),
		}
	}

	fn get_key_data(&self, key: &[u8], info: &StorageInfo, lookup_table: &StorageLookupTable) -> StorageKey {
		let key = if let Some(k) = lookup_table.extra_key_data(key) {
			k
		} else {
			return StorageKey {
				module: info.module.name().into(),
				prefix: info.meta.prefix().to_string(),
				extra: None,
			};
		};

		match &info.meta.ty {
			StorageType::Plain(_) => {
				StorageKey { module: info.module.name().into(), prefix: info.meta.prefix().to_string(), extra: None }
			}
			StorageType::Map { hasher, key: key_type, .. } => {
				let key = self.decode_key_len(key, hasher);
				StorageKey {
					module: info.module.name().into(),
					prefix: info.meta.prefix().to_string(),
					extra: Some(StorageKeyData::Map {
						key: key.to_vec(),
						hasher: hasher.clone(),
						key_type: key_type.clone(),
					}),
				}
			}
			StorageType::DoubleMap { hasher, key1, key2, key2_hasher, .. } => {
				let key1_bytes = self.decode_key_len(key, hasher);
				let key2_bytes = self.decode_key_len(&key[key1_bytes.len()..], key2_hasher);
				StorageKey {
					module: info.module.name().into(),
					prefix: info.meta.prefix().to_string(),
					extra: Some(StorageKeyData::DoubleMap {
						hasher: hasher.clone(),
						key2_hasher: key2_hasher.clone(),
						key1: key1_bytes,
						key2: key2_bytes,
						key1_type: key1.clone(),
						key2_type: key2.clone(),
					}),
				}
			}
			StorageType::NMap { .. } => unimplemented!(),
		}
	}

	/// Decode the Key/Value pair of a storage entry
	pub fn decode_storage<V: AsRef<[u8]>, O: AsRef<[u8]>>(
		&self,
		spec: SpecVersion,
		data: (V, Option<O>),
	) -> Result<GenericStorage, Error> {
		let (key, value): (&[u8], Option<O>) = (data.0.as_ref(), data.1);
		let meta = self.versions.get(&spec).expect("Spec does not exist");
		let lookup_table = meta.storage_lookup_table();
		let storage_info = lookup_table.meta_for_key(key).ok_or_else(|| {
			Error::from(format!("Storage not found key={:#X?}, spec={}, chain={}", key, spec, self.chain.as_str()))
		})?;

		if value.is_none() {
			let key = self.get_key_data(key, storage_info, &lookup_table);
			return Ok(GenericStorage::new(key, None));
		}
		let value = value.unwrap();
		let value = value.as_ref();

		match &storage_info.meta.ty {
			StorageType::Plain(rtype) => {
				log::trace!("{:?}, module {}, spec {}", rtype, storage_info.module.name(), spec);
				let mut state = DecodeState::new(Some(&storage_info.module), None, meta, 0, spec, value);
				let value = self.decode_single(&mut state, rtype, false)?;
				let key = self.get_key_data(key, storage_info, &lookup_table);
				let storage = GenericStorage::new(key, Some(StorageValue::new(value)));
				Ok(storage)
			}
			StorageType::Map { value: val_rtype, unused: _unused, .. } => {
				log::trace!(
					"Resolving storage `Map`. Value: {:?}, module {}, spec {}",
					val_rtype,
					storage_info.module.name(),
					spec
				);
				let key = self.get_key_data(key, storage_info, &lookup_table);
				let mut state = DecodeState::new(Some(&storage_info.module), None, meta, 0, spec, value);
				let value = self.decode_single(&mut state, val_rtype, false)?;
				let storage = GenericStorage::new(key, Some(StorageValue::new(value)));
				Ok(storage)
			}
			StorageType::DoubleMap { value: val_rtype, .. } => {
				log::trace!(
					"Resolving storage `DoubleMap`. Value: {:?}, module {}, spec {}",
					value,
					storage_info.module.name(),
					spec
				);
				let key = self.get_key_data(key, storage_info, &lookup_table);
				let mut state = DecodeState::new(Some(&storage_info.module), None, meta, 0, spec, value);
				let value = self.decode_single(&mut state, val_rtype, false)?;
				let storage = GenericStorage::new(key, Some(StorageValue::new(value)));
				Ok(storage)
			}
			StorageType::NMap { .. } => unimplemented!(),
		}
	}

	/// Decode a Vec<Extrinsic>. (Vec<Vec<u8>>)
	pub fn decode_extrinsics(&self, spec: SpecVersion, data: &[u8]) -> Result<Vec<GenericExtrinsic>, Error> {
		let mut ext = Vec::new();
		let (length, prefix) = Self::scale_length(data)?;
		let meta = self.versions.get(&spec).ok_or(Error::MissingSpec(spec))?;
		log::trace!("Decoding {} Total Extrinsics. CALLS: {:#?}", length, meta.modules_by_call_index);

		let mut state = DecodeState::new(None, None, meta, prefix, spec, data);
		for (idx, extrinsic) in ChunkedExtrinsic::new(&data[prefix..]).enumerate() {
			log::trace!("Extrinsic {}:{:?}", idx, extrinsic);
			state.reset(extrinsic);
			ext.push(self.decode_extrinsic(&mut state)?);
		}

		Ok(ext)
	}

	/// Decode an extrinsic
	fn decode_extrinsic(&self, state: &mut DecodeState) -> Result<GenericExtrinsic, Error> {
		let signature = if state.interpret_version() { Some(self.decode_signature(state)?) } else { None };

		state.load_module()?;
		let types = self.decode_call(state)?;
		log::debug!("Finished cursor length={}", state.cursor());
		let call = state.call.borrow().as_ref().map(|c| c.name()).unwrap_or_else(|| "unknown".into());
		Ok(GenericExtrinsic::new(signature, types, call, state.module_name().into()))
	}

	/// Decode the signature part of an UncheckedExtrinsic
	fn decode_signature(&self, state: &mut DecodeState) -> Result<SubstrateType, Error> {
		log::trace!("SIGNED EXTRINSIC");
		log::trace!("Getting signature for spec: {}, chain: {}", state.spec, self.chain.as_str());
		let signature = self
			.types
			.get_extrinsic_ty(self.chain.as_str(), state.spec, "signature")
			.expect("Signature must not be empty");
		log::trace!("Signature type is: {}", signature);
		state.observe(line!());
		self.decode_single(state, signature, false)
	}

	fn decode_call(&self, state: &mut DecodeState) -> Result<Vec<(String, SubstrateType)>, Error> {
		let mut types: Vec<(String, SubstrateType)> = Vec::new();
		let call = state.call()?;
		for arg in call.arguments() {
			state.observe(line!());
			log::trace!("Decoding {:?} for call {}", &arg.ty, call);
			let val = self.decode_single(state, &arg.ty, false)?;
			types.push((arg.name.to_string(), val));
		}
		Ok(types)
	}

	/// Internal function to handle
	/// decoding of a single rust type marker
	/// from data and the curent position within the data
	///
	/// # Panics
	/// panics if a type cannot be decoded
	#[track_caller]
	fn decode_single(
		&self,
		state: &mut DecodeState,
		ty: &RustTypeMarker,
		is_compact: bool,
	) -> Result<SubstrateType, Error> {
		let ty = match ty {
			RustTypeMarker::TypePointer(v) => {
				log::trace!("Resolving: {}", v);

				if let Some(t) = self.decode_sub_type(state, v, is_compact)? {
					t
				} else {
					let new_type =
						self.types.get(self.chain.as_str(), state.spec, state.module_name(), v).ok_or_else(|| {
							Error::from(format!(
								"Name Resolution Failure: module={}, v={}, spec={}, chain={}",
								state.module_name(),
								v,
								state.spec,
								self.chain.as_str()
							))
						})?;
					log::trace!("Resolved {:?}", new_type);
					let saved_cursor = state.cursor();
					let resolved = self.decode_single(state, new_type, is_compact);
					if resolved.is_err() {
						if let Some(fallback) = self.types.try_fallback(state.module_name(), v) {
							log::trace!("Falling back to type: {}", fallback);
							state.set_cursor(saved_cursor);
							return self.decode_single(state, fallback, is_compact);
						}
					}
					resolved?
				}
			}
			RustTypeMarker::Unit(u) => SubstrateType::Unit(u.to_string()),
			RustTypeMarker::Struct(v) => {
				log::trace!("Struct::cursor = {:?}", state.cursor);
				let ty = self.decode_structlike(v, state, is_compact)?;
				SubstrateType::Struct(ty)
			}
			RustTypeMarker::Set(v) => {
				log::trace!("Set::cursor = {}", state.cursor());
				// a set item must be an u8
				// can decode this right away
				let index = state.do_index();
				SubstrateType::Set(v[index as usize].clone())
			}
			RustTypeMarker::Tuple(v) => {
				log::trace!("Tuple::cursor={}", state.cursor());
				let ty = v
					.iter()
					.map(|v| self.decode_single(state, v, is_compact))
					.collect::<Result<Vec<SubstrateType>, Error>>();
				SubstrateType::Composite(ty?)
			}
			RustTypeMarker::Enum(v) => {
				log::trace!("Enum::cursor={}", state.cursor());
				state.observe(line!());
				let index = state.do_index();
				let variant = &v[index as usize];
				let value = variant.value.as_ref().map(|v| self.decode_single(state, v, is_compact)).transpose()?;
				log::trace!("Enum: {:?}", value);
				SubstrateType::Enum(substrate_types::EnumField {
					name: variant.name.clone(),
					value: value.map(Box::new),
				})
			}
			RustTypeMarker::Array { size, ty } => {
				log::trace!("Array::cursor={}", state.cursor());
				let mut decoded_arr = Vec::with_capacity(*size);

				if *size == 0 {
					log::trace!("Returning Empty Array");
					return Ok(SubstrateType::Composite(Vec::new()));
				} else {
					for _ in 0..*size {
						decoded_arr.push(self.decode_single(state, ty, is_compact)?)
					}
				}
				// rely on cursor increments in sub-types (U32/substrate specific types)
				SubstrateType::Composite(decoded_arr)
			}
			RustTypeMarker::Std(v) => match v {
				CommonTypes::Vec(v) => {
					log::trace!("Vec::cursor={}", state.cursor());
					let length = state.scale_length()?;
					let mut vec = Vec::new();
					if length == 0 {
						return Ok(SubstrateType::Composite(Vec::new()));
					} else {
						for _ in 0..length {
							state.observe(line!());
							let decoded = self.decode_single(state, v, is_compact)?;
							vec.push(decoded);
						}
					}
					SubstrateType::Composite(vec)
				}
				CommonTypes::Option(v) => {
					log::trace!("Option::cursor={}", state.cursor());
					match state.do_index() {
						// None
						0x00 => SubstrateType::Option(Box::new(None)),
						// Some
						0x01 => {
							let ty = self.decode_single(state, v, is_compact)?;
							SubstrateType::Option(Box::new(Some(ty)))
						}
						_ => {
							panic!("Cannot deduce correct Option<T> enum variant");
						}
					}
				}
				CommonTypes::Result(v, e) => {
					log::trace!("Result::cursor={}", state.cursor());
					match state.do_index() {
						// Ok
						0x00 => {
							let ty = self.decode_single(state, v, is_compact)?;
							SubstrateType::Result(Box::new(Ok(ty)))
						}
						// Err
						0x01 => {
							let ty = self.decode_single(state, e, is_compact)?;
							SubstrateType::Result(Box::new(Err(ty)))
						}
						_ => {
							panic!("Cannot deduce correct Result<T> Enum Variant");
						}
					}
				}
				CommonTypes::Compact(v) => {
					log::trace!("COMPACT SWITCHED! Compact::cursor={}", state.cursor());
					self.decode_single(state, v, true)?
				}
			},
			RustTypeMarker::Generic(outer, _) => {
				log::trace!("Generic Type");
				// disregard 'inner' type of a generic
				self.decode_single(state, outer, is_compact)?
			}
			RustTypeMarker::Number => {
				panic!("number decoding not possible");
			}
			RustTypeMarker::U8 => {
				let num: u8 = if is_compact {
					let num: Compact<u8> = state.decode()?;
					num.into()
				} else {
					let num: u8 = state.decode()?;
					num
				};
				num.into()
			}
			RustTypeMarker::U16 => {
				log::trace!("Decoding u16");
				let num: u16 = if is_compact {
					let num: Compact<u16> = state.decode()?;
					num.into()
				} else {
					let num: u16 = state.decode()?;
					num
				};
				num.into()
			}
			RustTypeMarker::U32 => {
				log::trace!("Decoding u32");
				state.observe(line!());
				let num: u32 = if is_compact {
					let num: Compact<u32> = state.decode()?;
					num.into()
				} else {
					let num: u32 = state.decode()?;
					log::trace!("u32:{}", num);
					num
				};
				num.into()
			}
			RustTypeMarker::U64 => {
				log::trace!("Decoding u64");
				let num = if is_compact {
					let num: Compact<u64> = state.decode()?;
					num.into()
				} else {
					let num: u64 = state.decode()?;
					num
				};
				num.into()
			}
			RustTypeMarker::U128 => {
				log::trace!("Decoding u128");
				state.observe(line!());
				let num = if is_compact {
					let num: Compact<u128> = state.decode()?;
					num.into()
				} else {
					let num: u128 = state.decode()?;
					num
				};
				num.into()
			}
			RustTypeMarker::I8 => {
				log::trace!("Decoding i8");
				let num: i8 = if is_compact { unimplemented!() } else { state.decode()? };
				num.into()
			}
			RustTypeMarker::I16 => {
				log::trace!("Decoding i16");
				let num: i16 = if is_compact { unimplemented!() } else { state.decode()? };
				num.into()
			}
			RustTypeMarker::I32 => {
				log::trace!("Decoding i32");
				let num: i32 = if is_compact { unimplemented!() } else { state.decode()? };
				num.into()
			}
			RustTypeMarker::I64 => {
				log::trace!("Decoding i64");
				let num: i64 = if is_compact {
					// let num: Compact<i64> = Decode::decode(&mut &data[*cursor..*cursor+8])?;
					// num.into()
					unimplemented!()
				} else {
					state.decode()?
				};
				num.into()
			}
			RustTypeMarker::I128 => {
				log::trace!("Decoding i128");
				let num: i128 = if is_compact { unimplemented!() } else { state.decode()? };
				num.into()
			}
			RustTypeMarker::Bool => {
				log::trace!("Decoding boolean");
				let boo: bool = state.decode()?;
				//   . - .
				//  ( o o )
				//  |  0  \
				//   \     \
				//    `~~~~~' boo!
				boo.into()
			}
			RustTypeMarker::Null => SubstrateType::Null,
		};
		Ok(ty)
	}

	/// internal API to decode 'special' substrate types.
	/// Or, types that have a special encode/decode scheme
	/// that may include packing bytes in a struct.
	/// Packing for example implies that a struct with two u32 fields
	/// may be encoded as a u8 (one byte).
	/// These types override anything defined in JSON
	/// Tries to decode a type that is native to substrate
	/// for example, H256. Returns none if type cannot be deduced
	fn decode_sub_type(
		&self,
		state: &mut DecodeState,
		ty: &str,
		is_compact: bool,
	) -> Result<Option<SubstrateType>, Error> {
		match ty {
			// checks if the metadata includes types for the SignedExtensions
			// If not defaults to whatever is in extrinsics.json
			"SignedExtra" => {
				log::trace!("Decoding SignedExtra");
				let meta =
					self.versions.get(&state.spec).ok_or(format!("Metadata for spec {} not found", state.spec))?;
				if let Some(extensions) = meta.signed_extensions() {
					let extensions = RustTypeMarker::Tuple(extensions.to_vec());
					self.decode_single(state, &extensions, is_compact).map(Option::Some)
				} else {
					let ty = self
						.types
						.get_extrinsic_ty(self.chain.as_str(), state.spec, "SignedExtra")
						.ok_or_else(|| Error::from("Could not find type `SignedExtra`"))?;
					self.decode_single(state, ty, is_compact).map(Option::Some)
				}
			}
			// identity info may be added to in the future
			"IdentityInfo" => {
				log::trace!("Decoding IdentityInfo");
				let additional = self.decode_single(
					state,
					&RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::TypePointer(
						"IdentityInfoAdditional".to_string(),
					)))),
					is_compact,
				)?;
				let display =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let legal =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let web =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let riot =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let email =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let pgp_fingerprint = self.decode_single(
					state,
					&RustTypeMarker::Std(CommonTypes::Option(Box::new(RustTypeMarker::TypePointer(
						"H160".to_string(),
					)))),
					is_compact,
				)?;
				let image =
					self.decode_sub_type(state, "Data", is_compact)?.ok_or_else(|| Error::from("Data not resolved"))?;
				let twitter = self.decode_sub_type(state, "Data", is_compact);

				Ok(Some(SubstrateType::Struct(vec![
					StructField::new(Some("additional"), additional),
					StructField::new(Some("display"), display),
					StructField::new(Some("legal"), legal),
					StructField::new(Some("web"), web),
					StructField::new(Some("riot"), riot),
					StructField::new(Some("email"), email),
					StructField::new(Some("pgpFingerprint"), pgp_fingerprint),
					StructField::new(Some("image"), image),
					StructField::new(
						Some("twitter"),
						twitter.unwrap_or(Some(SubstrateType::Null)).ok_or_else(|| Error::from("Data not resolved"))?,
					),
				])))
			}
			"Data" => {
				log::trace!("Decoding Data");
				let identity_data: substrate_types::Data = state.decode()?;
				Ok(Some(SubstrateType::Data(identity_data)))
			}
			"IdentityFields" => {
				log::trace!("Decoding Identity Fields");
				// identity field are just bitflags that can be interpreted by a frontend
				let field: u64 = state.decode()?;
				Ok(Some(SubstrateType::IdentityField(field)))
			}
			"BitVec" => {
				log::trace!("Decoding BitVec");
				let bit_vec: bitvec::vec::BitVec<BitOrderLsb0, u8> = state.decode()?;
				Ok(Some(SubstrateType::BitVec(bit_vec)))
			}
			"Call" | "GenericCall" => {
				log::trace!("Decoding Call | GenericCall");
				state.load_module()?;
				let types = self.decode_call(state)?;
				log::trace!("Call is {:?}", types);
				Ok(Some(SubstrateType::Call(types)))
			}
			"GenericVote" => {
				log::trace!("Decoding GenericVote");
				let vote: pallet_democracy::Vote = state.decode()?;
				Ok(Some(SubstrateType::GenericVote(vote)))
			}
			// Old Address Format for backwards-compatibility https://github.com/paritytech/substrate/pull/7380
			"Lookup" | "GenericAddress" | "GenericLookupSource" | "GenericAccountId" => {
				log::trace!("Decoding Lookup | GenericAddress | GenericLookupSource | GenericAccountId");
				state.observe(line!());

				let val: substrate_types::Address = decode_old_address(state)?;
				log::trace!("Decode Successful {:?}", &val);
				Ok(Some(SubstrateType::Address(val)))
			}
			"<T::Lookup as StaticLookup>::Source" => {
				log::trace!("Decoding <T::Lookup as StaticLookup>::Source");
				state.observe(line!());
				Ok(Some(self.decode_single(state, &RustTypeMarker::TypePointer("LookupSource".into()), is_compact)?))
			}
			"GenericMultiAddress" => {
				let val: substrate_types::Address = state.decode()?;
				log::trace!("Address: {:?}", val);
				Ok(Some(SubstrateType::Address(val)))
			}
			"Era" => {
				log::trace!("ERA DATA: {:X?}", &state.data[state.cursor()]);
				let val: sp_runtime::generic::Era = state.decode()?;
				log::trace!("Resolved Era: {:?}", val);
				Ok(Some(SubstrateType::Era(val)))
			}
			"H256" => {
				let val: sp_core::H256 = state.decode()?;
				Ok(Some(SubstrateType::H256(val)))
			}
			"H512" => {
				let val: sp_core::H512 = state.decode()?;
				log::trace!("H512: {}", hex::encode(val.as_bytes()));
				Ok(Some(SubstrateType::H512(val)))
			}
			_ => Ok(None),
		}
	}

	/// internal api to get the number of items in a encoded series
	/// returns a tuple of (number_of_items, length_of_prefix)
	/// length of prefix is the length in bytes that the prefix took up
	/// in the encoded data
	fn scale_length(mut data: &[u8]) -> Result<(usize, usize), Error> {
		// alternative to `DecodeLength` trait, to avoid casting from a trait
		let length = u32::from(Compact::<u32>::decode(&mut data)?);
		let prefix = Compact::<u32>::compact_len(&length);
		let length = usize::try_from(length).map_err(|_| Error::from("Failed convert decoded size into usize."))?;
		Ok((length, prefix))
	}

	/// internal api to decode a vector of struct
	fn decode_structlike(
		&self,
		fields: &[crate::StructField],
		state: &mut DecodeState,
		is_compact: bool,
	) -> Result<Vec<StructField>, Error> {
		fields
			.iter()
			.map(|field| {
				log::trace!("name={:?}, field={}", field.name, field.ty);
				let ty = self.decode_single(state, &field.ty, is_compact)?;
				Ok(StructField { name: Some(field.name.clone()), ty })
			})
			.collect::<Result<Vec<StructField>, Error>>()
	}
}

/// Decodes old address pre-refactor (<https://github.com/paritytech/substrate/pull/7380>)
/// and converts it to a MultiAddress, where "old" here means anything before v0.8.26 or 26/2026/46 on polkadot/kusama/westend respectively.
fn decode_old_address(state: &DecodeState) -> Result<substrate_types::Address, Error> {
	/// Kept around for backwards-compatibility with old address struct
	fn need_more_than<T: PartialOrd>(a: T, b: T) -> Result<T, Error> {
		if a < b {
			Ok(b)
		} else {
			Err("Invalid range".into())
		}
	}

	let inc;
	let addr = match state.do_index() {
		// do_index for byte 0x00-0xff
		x @ 0x00..=0xef => {
			inc = 0;
			substrate_types::Address::Index(x as u32)
		}
		0xfc => {
			inc = 2;
			substrate_types::Address::Index(
				need_more_than(0xef, u16::decode(&mut &state.data[(state.cursor())..])?)? as u32
			)
		}
		0xfd => {
			inc = 4;
			substrate_types::Address::Index(need_more_than(0xffff, u32::decode(&mut &state.data[(state.cursor())..])?)?)
		}
		0xfe => {
			inc = 8;
			substrate_types::Address::Index(need_more_than(
				0xffff_ffff_u32,
				Decode::decode(&mut &state.data[(state.cursor())..])?,
			)?)
		}
		0xff => {
			inc = 32;
			substrate_types::Address::Id(Decode::decode(&mut &state.data[(state.cursor())..])?)
		}
		_ => return Err(Error::Fail("Invalid Address".to_string())),
	};
	state.add(inc);
	Ok(addr)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		decoder::metadata::test_suite as meta_test_suite,
		substrate_types::{EnumField, StructField},
		test_suite, EnumField as RustEnumField,
	};
	use codec::Encode;

	#[derive(Debug, Clone)]
	struct GenericTypes;

	impl TypeDetective for GenericTypes {
		fn get(&self, _chain: &str, _spec: u32, _module: &str, _ty: &str) -> Option<&RustTypeMarker> {
			Some(&RustTypeMarker::I128)
		}

		fn try_fallback(&self, _module: &str, _ty: &str) -> Option<&RustTypeMarker> {
			None
		}

		fn get_extrinsic_ty(&self, _chain: &str, _spec: u32, _ty: &str) -> Option<&RustTypeMarker> {
			None
		}
	}

	#[test]
	fn should_insert_metadata() {
		let mut decoder = Decoder::new(GenericTypes, Chain::Kusama);
		decoder.register_version(test_suite::mock_runtime(0).spec_version, meta_test_suite::test_metadata()).unwrap();
		decoder.register_version(test_suite::mock_runtime(1).spec_version, meta_test_suite::test_metadata()).unwrap();
		decoder.register_version(test_suite::mock_runtime(2).spec_version, meta_test_suite::test_metadata()).unwrap();
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(0).spec_version));
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(1).spec_version));
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(2).spec_version))
	}

	#[test]
	fn should_get_version_metadata() {
		let mut decoder = Decoder::new(GenericTypes, Chain::Kusama);
		let rt_version = test_suite::mock_runtime(0);
		let meta = meta_test_suite::test_metadata();
		decoder.register_version(rt_version.spec_version, meta.clone()).unwrap();
		let _other_meta = decoder.get_version_metadata(rt_version.spec_version);
		assert_eq!(Some(&meta), _other_meta.clone())
	}

	#[test]
	fn should_get_scale_length() {
		let encoded = vec![32, 4].encode();
		for v in encoded.iter() {
			print!("{:08b} ", v);
		}
		let len = Decoder::scale_length(encoded.as_slice()).unwrap();
		assert_eq!(len.0, 2);
	}

	macro_rules! decode_test {
		( $v: expr, $x:expr, $r: expr) => {{
			let val = $v.encode();
			let decoder = Decoder::new(GenericTypes, Chain::Kusama);
			let meta = meta_test_suite::test_metadata();
			let mut state = DecodeState::new(None, None, &meta, 0, 1031, val.as_slice());
			let res = decoder.decode_single(&mut state, &$x, false).unwrap();
			assert_eq!($r, res)
		}};
	}

	#[test]
	fn should_decode_option() {
		let val: Option<u32> = Some(0x1337);
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Option(Box::new(RustTypeMarker::U32))),
			SubstrateType::Option(Box::new(Some(SubstrateType::U32(0x1337))))
		);
		let val: Option<u32> = None;
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Option(Box::new(RustTypeMarker::U32))),
			SubstrateType::Option(Box::new(None))
		);
	}

	#[test]
	fn should_decode_result() {
		let val: Result<u32, u32> = Ok(0x1337);
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Result(Box::new(RustTypeMarker::U32), Box::new(RustTypeMarker::U32))),
			SubstrateType::Result(Box::new(Ok(SubstrateType::U32(0x1337))))
		);

		let val: Result<u32, u32> = Err(0x1337);
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Result(Box::new(RustTypeMarker::U32), Box::new(RustTypeMarker::U32),)),
			SubstrateType::Result(Box::new(Err(SubstrateType::U32(0x1337))))
		);
	}

	#[test]
	fn should_decode_vector() {
		let val: Vec<u32> = vec![12, 32, 0x1337, 62];
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U32))),
			SubstrateType::Composite(vec![
				SubstrateType::U32(12),
				SubstrateType::U32(32),
				SubstrateType::U32(0x1337),
				SubstrateType::U32(62)
			])
		);

		let val: Vec<u128> = vec![12, 32, 0x1337, 62];
		decode_test!(
			val,
			RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U128))),
			SubstrateType::Composite(vec![
				SubstrateType::U128(12),
				SubstrateType::U128(32),
				SubstrateType::U128(0x1337),
				SubstrateType::U128(62)
			])
		);
	}

	#[test]
	fn should_decode_array() {
		let val: [u32; 4] = [12, 32, 0x1337, 62];
		decode_test!(
			val,
			RustTypeMarker::Array { size: 4, ty: Box::new(RustTypeMarker::U32) },
			SubstrateType::Composite(vec![
				SubstrateType::U32(12),
				SubstrateType::U32(32),
				SubstrateType::U32(0x1337),
				SubstrateType::U32(62)
			])
		)
	}

	#[test]
	fn should_decode_struct() {
		#[derive(Encode, Decode)]
		struct ToDecode {
			foo: u32,
			name: Vec<u8>,
		}
		let val = ToDecode { foo: 0x1337, name: vec![8, 16, 30, 40] };
		decode_test!(
			val,
			RustTypeMarker::Struct(vec![
				crate::StructField { name: "foo".to_string(), ty: RustTypeMarker::U32 },
				crate::StructField {
					name: "name".to_string(),
					ty: RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U8,))),
				},
			]),
			SubstrateType::Struct(vec![
				StructField { name: Some("foo".to_string()), ty: SubstrateType::U32(0x1337) },
				StructField {
					name: Some("name".to_string()),
					ty: SubstrateType::Composite(vec![
						SubstrateType::U8(8),
						SubstrateType::U8(16),
						SubstrateType::U8(30),
						SubstrateType::U8(40)
					])
				}
			])
		);
	}

	#[test]
	fn should_decode_tuple() {
		let val: (u32, u32, u32, u32) = (18, 32, 42, 0x1337);
		decode_test!(
			val,
			RustTypeMarker::Tuple(vec![
				RustTypeMarker::U32,
				RustTypeMarker::U32,
				RustTypeMarker::U32,
				RustTypeMarker::U32,
			]),
			SubstrateType::Composite(vec![
				SubstrateType::U32(18),
				SubstrateType::U32(32),
				SubstrateType::U32(42),
				SubstrateType::U32(0x1337)
			])
		)
	}

	#[test]
	fn should_decode_unit_enum() {
		#[derive(Encode, Decode)]
		enum Foo {
			Zoo,
			Wraith,
			Spree,
		}
		let val = Foo::Wraith;
		decode_test!(
			val,
			RustTypeMarker::Enum(vec![
				RustEnumField::new("Zoo".into(), None),
				RustEnumField::new("Wraith".into(), None),
				RustEnumField::new("Spree".into(), None),
			]),
			SubstrateType::Enum(EnumField::new("Wraith".into(), None))
		);
	}

	#[test]
	fn should_decode_tuple_enum() {
		#[derive(Encode, Decode)]
		struct TestStruct(i128);

		#[derive(Encode, Decode)]
		enum Foo {
			Zoo(TestStruct),
			Wraith(TestStruct),
		}
		let val = Foo::Wraith(TestStruct(0x1337));
		decode_test!(
			val,
			RustTypeMarker::Enum(vec![
				RustEnumField::new("Zoo".into(), Some(RustTypeMarker::TypePointer("TestStruct".into())),),
				RustEnumField::new("Wraith".into(), Some(RustTypeMarker::TypePointer("TestStruct".into())),),
			]),
			SubstrateType::Enum(EnumField::new("Wraith".into(), Some(Box::new(SubstrateType::I128(0x1337)))))
		);
	}

	#[test]
	fn should_decode_structlike_enum() {
		#[derive(Encode, Decode)]
		enum Foo {
			Zoo { name: Vec<u8>, id: u32 },
			Wraith { name: Vec<u16>, id: u64 },
		}
		let val = Foo::Wraith { name: vec![0x13, 0x37], id: 15 };

		decode_test!(
			val,
			RustTypeMarker::Enum(vec![
				RustEnumField::new(
					"Zoo".into(),
					Some(RustTypeMarker::Struct(vec![
						crate::StructField::new(
							"name",
							RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U8,))),
						),
						crate::StructField::new("id", RustTypeMarker::U32),
					])),
				),
				RustEnumField::new(
					"Wraith".into(),
					Some(RustTypeMarker::Struct(vec![
						crate::StructField::new(
							"name",
							RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U16,))),
						),
						crate::StructField::new("id", RustTypeMarker::U64),
					])),
				),
			]),
			SubstrateType::Enum(EnumField::new(
				"Wraith".into(),
				Some(Box::new(SubstrateType::Struct(vec![
					StructField {
						name: Some("name".into()),
						ty: SubstrateType::Composite(vec![SubstrateType::U16(0x13), SubstrateType::U16(0x37)])
					},
					StructField { name: Some("id".into()), ty: SubstrateType::U64(15) }
				])))
			))
		);
	}

	#[test]
	fn should_chunk_extrinsic() {
		let test = vec![vec![0u8, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
		let encoded: Vec<u8> = test.encode();
		let (_length, prefix) = Decoder::scale_length(encoded.as_slice()).unwrap(); // get the overall length first
		let mut chunked = ChunkedExtrinsic::new(&encoded[prefix..]);
		assert_eq!(chunked.next(), Some(vec![0, 1, 2].as_slice()));
		assert_eq!(chunked.next(), Some(vec![3, 4, 5].as_slice()));
		assert_eq!(chunked.next(), Some(vec![6, 7, 8].as_slice()));
	}
}
