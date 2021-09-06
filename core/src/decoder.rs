// Copyright 2019 Parity Technologies (UK) Ltd.
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
	CallMetadata, Metadata, MetadataError, ModuleIndex, ModuleMetadata, StorageEntryModifier, StorageHasher,
	StorageType,
};
pub use frame_metadata::v14::StorageEntryType;

use crate::{
	error::Error,
	substrate_types::{self, StructField, SubstrateType},
	CommonTypes, RustTypeMarker, TypeDetective,
};
use codec::{Compact, CompactLen, Decode};
use std::{collections::HashMap, convert::TryFrom, str::FromStr};

type SpecVersion = u32;
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
			_ => Err(Error::Fail(
				"Network must be one of: 'kusama', 'polkadot', 'moonriver', or their
				token abbreviations."
					.into(),
			)),
		}
	}
}

struct DecodeState<'a> {
	module: &'a ModuleMetadata,
	call: Option<&'a CallMetadata>,
	cursor: &'a mut usize,
	spec: SpecVersion,
	data: &'a [u8],
}

impl<'a> DecodeState<'a> {
	fn new(
		module: &'a ModuleMetadata,
		call: Option<&'a CallMetadata>,
		cursor: &'a mut usize,
		spec: SpecVersion,
		data: &'a [u8],
	) -> Self {
		Self { module, call, cursor, spec, data }
	}

	fn module_name(&'a self) -> &'a str {
		self.module.name()
	}

	// Gets the call at the current index. Increments cursor by 1.
	// Sets the call for the state.
	fn call(&'a mut self) -> Result<&'a CallMetadata, MetadataError> {
		let call = self.data[*self.cursor];
		let call = self.module.call(call)?;
		self.call = Some(&call.clone());
		Ok(call)
	}

	/// Get the current number at this point in the cursors life.
	/// Increment the cursor by 1.
	fn index(&'a mut self) -> u8 {
		let number = self.data[*self.cursor];
		*self.cursor += 1;
		number
	}

	/// Get the scale length at the current point in time.
	/// Increment cursor accordingly to the length.
	fn scale_length(&'a mut self) -> Result<usize, Error> {
		let length = Decoder::scale_length(&self.data[*self.cursor..])?;
		*self.cursor += length.1;
		Ok(length.0)
	}

	fn decode<T: Decode>(&self, inc: usize) -> Result<T, Error> {
		let ty: T = Decode::decode(&mut &self.data[*self.cursor..])?;
		*self.cursor += inc;
		Ok(ty)
	}
	/*
		fn inc(&mut self, inc: usize) {
			*self.cursor += inc;
		}
	*/
	fn replace_cursor(&'a mut self, new: usize) {
		*self.cursor = new;
	}
}

impl Decoder {
	/// Create new Decoder with specified types
	pub fn new(types: impl TypeDetective + 'static, chain: Chain) -> Self {
		Self { versions: HashMap::default(), types: Box::new(types), chain: chain.to_string() }
	}

	/// Check if a metadata version has already been registered
	pub fn has_version(&self, version: SpecVersion) -> bool {
		self.versions.contains_key(&version)
	}

	/// Insert a Metadata with Version attached
	/// If version exists, it's corresponding metadata will be updated
	pub fn register_version<M: Into<Metadata>>(&mut self, version: SpecVersion, metadata: M) {
		let meta: Metadata = metadata.into();
		self.versions.insert(version, meta);
	}

	/// internal api to get runtime version
	/// panics if a version is not found
	///
	/// get runtime version in less than linear time with binary search
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
				let mut cursor = 0;
				let mut state = DecodeState::new(&storage_info.module, None, &mut cursor, spec, value);
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
				let mut cursor = 0;
				let state = DecodeState::new(&storage_info.module, None, &mut cursor, spec, value);
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
				let mut cursor = 0;
				let mut state = DecodeState::new(&storage_info.module, None, &mut cursor, spec, value);
				let value = self.decode_single(&mut state, val_rtype, false)?;
				let storage = GenericStorage::new(key, Some(StorageValue::new(value)));
				Ok(storage)
			}
		}
	}

	pub fn decode_extrinsics(&self, spec: SpecVersion, data: &[u8]) -> Result<Vec<GenericExtrinsic>, Error> {
		let mut ext = Vec::new();
		let meta = self.versions.get(&spec).expect("Spec does not exist"); // TODO: remove panic

		// first byte -> vector length
		// second byte -> extrinsic version
		// third byte -> Outer enum index
		// fourth byte -> inner enum index (function index)
		// can check if signed via a simple & too
		let mut cursor = 0;
		let length = Self::scale_length(&data[cursor..])?;
		cursor += length.1;
		log::debug!("Extrinsic Length: {:?}", length);

		while cursor < data.len() {
			self.decode_extrinsic(&data[..], &meta, spec, &mut ext, &mut cursor)?;
			log::info!("Success! {}", serde_json::to_string_pretty(&ext).unwrap());
			log::debug!(
				"cursor={}, data[cursor]={}, data[cursor..]={}:{:?}, &data[..] = {}:{:?}",
				cursor,
				data[cursor],
				hex::encode(&data[cursor..]),
				&data[cursor..],
				hex::encode(data),
				data
			);
		}
		Ok(ext)
	}

	/// Decode an extrinsic
	fn decode_extrinsic<'a>(
		&self,
		data: &'a [u8],
		meta: &'a Metadata,
		spec: SpecVersion,
		ext: &mut Vec<GenericExtrinsic>,
		cursor: &'a mut usize,
	) -> Result<(), Error> {
		let version = data[*cursor];
		let is_signed = version & 0b1000_0000 != 0;
		let version = version & 0b0111_1111;
		log::trace!("Extrinsic Version: {}", version);
		*cursor += 1;
		let signature = if is_signed {
			let module = meta.module("runtime")?;
			let mut state = DecodeState::new(&module, None, cursor, spec, data);
			Some(self.decode_signature(&mut state)?)
		} else {
			None
		};

		if let Some(s) = &signature {
			log::debug!("signature={}", s);
		}

		let module = meta
			.module_by_index(ModuleIndex::Call(data[*cursor]))
			.map_err(|e| Error::DetailedMetaFail(e, *cursor, hex::encode(data)))?;
		*cursor += 1;
		let mut state = DecodeState::<'a>::new(&module, None, cursor, spec, data);

		let types = self.decode_call(&mut state)?;
		log::debug!("Finished cursor length={}", *cursor);
		let call = state.call.expect("EMPTY CALL");
		ext.push(GenericExtrinsic::new(signature, types, call.name(), module.name().into()));
		Ok(())
	}

	/// Decode the signature part of an UncheckedExtrinsic
	fn decode_signature<'a>(&self, state: &'a mut DecodeState<'a>) -> Result<SubstrateType, Error> {
		log::trace!("SIGNED EXTRINSIC");
		log::trace!("Getting signature for spec: {}, chain: {}", state.spec, self.chain.as_str());
		let signature = self
			.types
			.get_extrinsic_ty(self.chain.as_str(), state.spec, "signature")
			.expect("Signature must not be empty");

		// Ok(Some(self.decode_single("runtime", spec, signature, data, cursor, false)?))
		Ok(self.decode_single(state, signature, false)?)
	}

	fn decode_call<'a>(&self, state: &'a mut DecodeState<'a>) -> Result<Vec<(String, SubstrateType)>, Error> {
		// TODO: tuple of argument name -> value
		let mut types: Vec<(String, SubstrateType)> = Vec::new();
		let call = state.call()?;
		for arg in call.arguments() {
			log::trace!("arg = {:?}", arg);
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
	fn decode_single<'a>(
		&self,
		state: &'a mut DecodeState<'a>,
		ty: &'a RustTypeMarker,
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
					let mut saved_cursor = *state.cursor;
					let resolved = self.decode_single(state, new_type, is_compact);
					if resolved.is_err() {
						if let Some(fallback) =
							self.types.try_fallback(self.chain.as_str(), state.spec, state.module_name(), v)
						{
							state.replace_cursor(saved_cursor);
							return Ok(self.decode_single(state, fallback, is_compact)?);
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
			// TODO: test
			RustTypeMarker::Set(v) => {
				log::trace!("Set::cursor = {}", *state.cursor);
				// a set item must be an u8
				// can decode this right away
				let index = state.index();
				SubstrateType::Set(v[index as usize].clone())
			}
			RustTypeMarker::Tuple(v) => {
				log::trace!("Tuple::cursor={}", *state.cursor);
				let ty = v
					.iter()
					.map(|v| self.decode_single(state, v, is_compact))
					.collect::<Result<Vec<SubstrateType>, Error>>();
				SubstrateType::Composite(ty?)
			}
			RustTypeMarker::Enum(v) => {
				log::trace!("Enum::cursor={}", *state.cursor);
				let index = state.index();
				let variant = &v[index as usize];
				let value = variant.value.as_ref().map(|v| self.decode_single(state, &v, is_compact)).transpose()?;

				SubstrateType::Enum(substrate_types::EnumField {
					name: variant.name.clone(),
					value: value.map(|v| Box::new(v)),
				})
			}
			RustTypeMarker::Array { size, ty } => {
				log::trace!("Array::cursor={}", *state.cursor);
				let mut decoded_arr = Vec::with_capacity(*size);
				if *size == 0_usize {
					log::trace!("Returning Empty Vector");
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
					log::trace!("Vec::cursor={}", *state.cursor);
					let length = state.scale_length()?;
					// we can just decode this as an "array" now
					self.decode_single(state, &RustTypeMarker::Array { size: length, ty: v.clone() }, is_compact)?
				}
				CommonTypes::Option(v) => {
					log::trace!("Option::cursor={}", *state.cursor);
					match state.data[*state.cursor] {
						// None
						0x00 => {
							*state.cursor += 1;
							SubstrateType::Option(Box::new(None))
						}
						// Some
						0x01 => {
							*state.cursor += 1;
							let ty = self.decode_single(state, v, is_compact)?;
							SubstrateType::Option(Box::new(Some(ty)))
						}
						_ => {
							panic!("Cannot deduce correct Option<T> enum variant");
						}
					}
				}
				CommonTypes::Result(v, e) => {
					log::trace!("Result::cursor={}", *state.cursor);
					match state.data[*state.cursor] {
						// Ok
						0x00 => {
							*state.cursor += 1;
							let ty = self.decode_single(state, v, is_compact)?;
							SubstrateType::Result(Box::new(Ok(ty)))
						}
						// Err
						0x01 => {
							*state.cursor += 1;
							let ty = self.decode_single(state, e, is_compact)?;
							SubstrateType::Result(Box::new(Err(ty)))
						}
						_ => {
							panic!("Cannot deduce correct Result<T> Enum Variant");
						}
					}
				}
				// TODO: test
				CommonTypes::Compact(v) => {
					log::trace!("Compact::cursor={}", state.cursor);
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
				log::trace!("decoding u8");
				let num: u8 = if is_compact {
					let num: Compact<u8> = Decode::decode(&mut &state.data[*state.cursor..])?;
					*state.cursor += Compact::compact_len(&u8::from(num));
					num.into()
				} else {
					let num: u8 = state.decode(1)?;
					num
				};
				num.into()
			}
			RustTypeMarker::U16 => {
				log::trace!("Decoding u16");
				let num: u16 = if is_compact {
					let num: Compact<u16> = Decode::decode(&mut &state.data[*state.cursor..])?;
					*state.cursor += Compact::compact_len(&u16::from(num));
					num.into()
				} else {
					let num: u16 = state.decode(2)?;
					num
				};
				num.into()
			}
			RustTypeMarker::U32 => {
				log::trace!("Decoding u32");
				log::trace!("{:?}", &state.data[*state.cursor..]);
				let num: u32 = if is_compact {
					let num: Compact<u32> = Decode::decode(&mut &state.data[*state.cursor..])?;
					let len = Compact::compact_len(&u32::from(num));
					log::trace!("Compact len: {}", len);
					*state.cursor += len;
					num.into()
				} else {
					let num: u32 = state.decode(4)?;
					num
				};
				num.into()
			}
			RustTypeMarker::U64 => {
				log::trace!("Decoding u64");
				let num: u64 = if is_compact {
					let num: Compact<u64> = Decode::decode(&mut &state.data[*state.cursor..])?;
					*state.cursor += Compact::compact_len(&u64::from(num));
					num.into()
				} else {
					let num: u64 = state.decode(8)?;
					num
				};
				num.into()
			}
			RustTypeMarker::U128 => {
				log::trace!("Decoding u128");
				log::trace!("cursor = {}, data = {:?}", state.cursor, &state.data[*state.cursor..]);
				let num: u128 = if is_compact {
					let num: Compact<u128> = Decode::decode(&mut &state.data[*state.cursor..])?;
					*state.cursor += Compact::compact_len(&u128::from(num));
					num.into()
				} else {
					let num: u128 = state.decode(16)?;
					num
				};
				num.into()
			}
			RustTypeMarker::USize => {
				panic!("usize decoding not possible!")
				/* let size = std::mem::size_of::<usize>();
				let num: usize =
					Decode::decode(&mut &data[*cursor..=*cursor+size])?;
				*cursor += std::mem::size_of::<usize>();
				num.into()
				 */
			}
			RustTypeMarker::I8 => {
				log::trace!("Decoding i8");
				let num: i8 = if is_compact { unimplemented!() } else { state.decode(1)? };
				num.into()
			}
			RustTypeMarker::I16 => {
				log::trace!("Decoding i16");
				let num: i16 = if is_compact { unimplemented!() } else { state.decode(2)? };
				num.into()
			}
			RustTypeMarker::I32 => {
				log::trace!("Decoding i32");
				let num: i32 = if is_compact { unimplemented!() } else { state.decode(4)? };
				num.into()
			}
			RustTypeMarker::I64 => {
				log::trace!("Decoding i64");
				let num: i64 = if is_compact {
					// let num: Compact<i64> = Decode::decode(&mut &data[*cursor..*cursor+8])?;
					// num.into()
					unimplemented!()
				} else {
					state.decode(8)?
				};
				num.into()
			}
			RustTypeMarker::I128 => {
				log::trace!("Decoding i128");
				let num: i128 = if is_compact { unimplemented!() } else { state.decode(16)? };
				num.into()
			}
			RustTypeMarker::ISize => {
				panic!("isize decoding impossible!")
				/*
				let idx = std::mem::size_of::<isize>();
				let num: isize =
					Decode::decode(&mut &data[*cursor..=*cursor + idx])?;
				*cursor += std::mem::size_of::<isize>();
				num.into()
				*/
			}
			RustTypeMarker::F32 => {
				/*
				let num: f32 = Decode::decode(&mut &data[*cursor..=*cursor + 4])?;
				*cursor += 5;
				num.into()
				 */
				panic!("f32 decoding impossible!");
			}
			RustTypeMarker::F64 => {
				/*
				let num: f64 = Decode::decode(&mut &data[*cursor..=*cursor + 8])?;
				*cursor += 9;
				num.into()
				 */
				panic!("f64 decoding impossible!");
			}
			RustTypeMarker::String => unimplemented!(),
			RustTypeMarker::Bool => {
				log::trace!("Decoding boolean");
				let boo: bool = state.decode(1)?;
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
	/// Supported types:
	/// - H256
	/// - H512
	// TODO: test this with the substrate types used
	fn decode_sub_type<'a>(
		&self,
		state: &'a mut DecodeState<'a>,
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
					// TODO
					// self.decode_single(", state.spec, &extensions, state.data, state.cursor, is_compact).map(Option::Some)
					self.decode_single(state, &extensions, is_compact).map(Option::Some)
				} else {
					let ty = self
						.types
						.get_extrinsic_ty(self.chain.as_str(), state.spec, "SignedExtra")
						.ok_or_else(|| Error::from("Could not find type `SignedExtra`"))?;
					self.decode_single(state, ty, is_compact).map(Option::Some)
					// TODO
					// self.decode_single("", state.spec, ty, state.data, state.cursor, is_compact).map(Option::Some)
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
				log::trace!("Data::cursor={}", *state.cursor);
				let identity_data: substrate_types::Data = Decode::decode(&mut &state.data[*state.cursor..])?;
				match &identity_data {
					substrate_types::Data::None => (),
					substrate_types::Data::Raw(v) => *state.cursor += v.len(),
					_ => *state.cursor += 32,
				};
				// for the enum byte
				*state.cursor += 1;
				Ok(Some(SubstrateType::Data(identity_data)))
			}
			"Call" | "GenericCall" => {
				log::trace!("Decoding Call | GenericCall");
				let types = self.decode_call(state)?;
				Ok(Some(SubstrateType::Call(types)))
			}
			"GenericVote" => {
				log::trace!("Decoding GenericVote");
				let vote: pallet_democracy::Vote = state.decode(1)?;
				Ok(Some(SubstrateType::GenericVote(vote)))
			}
			// Old Address Format for backwards-compatibility https://github.com/paritytech/substrate/pull/7380
			"Lookup" | "GenericAddress" | "GenericLookupSource" | "GenericAccountId" => {
				// a specific type that is <T as StaticSource>::Lookup concatenated to just 'Lookup'
				log::trace!("Decoding Lookup | GenericAddress | GenericLookupSource | GenericAccountId");
				log::trace!("cursor={}, data length={}", state.cursor, state.data.len());

				let val: substrate_types::Address = decode_old_address(state.data, state.cursor)?;
				log::trace!("Decode Sucessful {:?}", &val);
				Ok(Some(SubstrateType::Address(val)))
			}
			"GenericMultiAddress" => {
				let val: substrate_types::Address = Decode::decode(&mut &state.data[*state.cursor..])?;
				let cursor_offset = match &val {
					substrate_types::Address::Id(_) => 32,
					substrate_types::Address::Index(_) => 1,
					substrate_types::Address::Raw(v) => v.len(),
					substrate_types::Address::Address32(_) => 32,
					substrate_types::Address::Address20(_) => 20,
				};
				*state.cursor += cursor_offset;
				Ok(Some(SubstrateType::Address(val)))
			}
			"Era" => {
				log::trace!("ERA DATA: {:X?}", &state.data[*state.cursor..]);
				let val: runtime_primitives::generic::Era = Decode::decode(&mut &state.data[*state.cursor..])?;
				log::trace!("Resolved Era: {:?}", val);
				match val {
					// although phase and period are both u64, era is Encoded
					// in only two bytes
					runtime_primitives::generic::Era::Immortal => *state.cursor += 1,
					runtime_primitives::generic::Era::Mortal(_, _) => *state.cursor += 2,
				};
				Ok(Some(SubstrateType::Era(val)))
			}
			"H256" => {
				let val: primitives::H256 = state.decode(32)?;
				Ok(Some(SubstrateType::H256(val)))
			}
			"H512" => {
				let val: primitives::H512 = state.decode(64)?;
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
		let u32_length = u32::from(Compact::<u32>::decode(&mut data)?);
		let length_of_prefix: usize = Compact::compact_len(&u32_length);
		let usize_length =
			usize::try_from(u32_length).map_err(|_| Error::from("Failed convert decoded size into usize."))?;
		Ok((usize_length, length_of_prefix))
	}

	/// internal api to decode a vector of struct IdentityFields
	fn decode_structlike<'a>(
		&self,
		fields: &'a [crate::StructField],
		state: &'a mut DecodeState<'a>,
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

/// Decodes old address pre-refactor (https://github.com/paritytech/substrate/pull/7380)
/// and converts it to a MultiAddress, where "old" here means anything before v0.8.26 or 26/2026/46 on polkadot/kusama/westend respectively.
fn decode_old_address(data: &[u8], cursor: &mut usize) -> Result<substrate_types::Address, Error> {
	/// Kept around for backwards-compatibility with old address struct
	fn need_more_than<T: PartialOrd>(a: T, b: T) -> Result<T, Error> {
		if a < b {
			Ok(b)
		} else {
			Err("Invalid range".into())
		}
	}

	let inc;
	let addr = match data[*cursor] {
		x @ 0x00..=0xef => {
			inc = 0;
			substrate_types::Address::Index(x as u32)
		}
		0xfc => {
			inc = 2;
			substrate_types::Address::Index(need_more_than(0xef, u16::decode(&mut &data[(*cursor + 1)..])?)? as u32)
		}
		0xfd => {
			inc = 4;
			substrate_types::Address::Index(need_more_than(0xffff, u32::decode(&mut &data[(*cursor + 1)..])?)?)
		}
		0xfe => {
			inc = 8;
			substrate_types::Address::Index(need_more_than(
				0xffff_ffff_u32,
				Decode::decode(&mut &data[(*cursor + 1)..])?,
			)?)
		}
		0xff => {
			inc = 32;
			substrate_types::Address::Id(Decode::decode(&mut &data[(*cursor + 1)..])?)
		}
		_ => return Err(Error::Fail("Invalid Address".to_string())),
	};
	*cursor += inc + 1; // +1 for byte 0x00-0xff
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

		fn try_fallback(&self, _chain: &str, _spec: u32, _module: &str, _ty: &str) -> Option<&RustTypeMarker> {
			None
		}

		fn get_extrinsic_ty(&self, _chain: &str, _spec: u32, _ty: &str) -> Option<&RustTypeMarker> {
			None
		}
	}

	#[test]
	fn should_insert_metadata() {
		let mut decoder = Decoder::new(GenericTypes, Chain::Kusama);
		decoder.register_version(test_suite::mock_runtime(0).spec_version, &meta_test_suite::test_metadata());
		decoder.register_version(test_suite::mock_runtime(1).spec_version, &meta_test_suite::test_metadata());
		decoder.register_version(test_suite::mock_runtime(2).spec_version, &meta_test_suite::test_metadata());
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(0).spec_version));
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(1).spec_version));
		assert!(decoder.versions.contains_key(&test_suite::mock_runtime(2).spec_version))
	}

	#[test]
	fn should_get_version_metadata() {
		// let types = PolkadotTypes::new().unwrap();
		let mut decoder = Decoder::new(GenericTypes, Chain::Kusama);
		let rt_version = test_suite::mock_runtime(0);
		let meta = meta_test_suite::test_metadata();
		decoder.register_version(rt_version.spec_version.clone(), &meta);
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
			let res = decoder.decode_single("", 1031, &$x, val.as_slice(), &mut 0, false).unwrap();

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
}
