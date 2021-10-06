// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Decodable variant of the RuntimeMetadata.
//!
//! This really doesn't belong here, but is necessary for the moment. In the future
//! it should be removed entirely to an external module for shimming on to the
//! codec-encoded metadata.

#![allow(clippy::all)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde::Serialize;
#[cfg(feature = "std")]
use codec::{Decode, Input, Error};
use codec::{Encode, Output};
use sp_std::vec::Vec;
use frame_metadata::decode_different::*;

#[cfg(feature = "std")]
type StringBuf = String;

/// Current prefix of metadata
pub const META_RESERVED: u32 = 0x6174656d; // 'meta' warn endianness

/// All the metadata about a function.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct FunctionMetadata {
	pub name: DecodeDifferentStr,
	pub arguments: DecodeDifferentArray<FunctionArgumentMetadata>,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about a function argument.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct FunctionArgumentMetadata {
	pub name: DecodeDifferentStr,
	pub ty: DecodeDifferentStr,
}

/// Newtype wrapper for support encoding functions (actual the result of the function).
#[derive(Clone, Eq)]
pub struct FnEncode<E>(pub fn() -> E) where E: Encode + 'static;

impl<E: Encode> Encode for FnEncode<E> {
	fn encode_to<W: Output + ?Sized>(&self, dest: &mut W) {
		self.0().encode_to(dest);
	}
}

impl<E: Encode> codec::EncodeLike for FnEncode<E> {}

impl<E: Encode + PartialEq> PartialEq for FnEncode<E> {
	fn eq(&self, other: &Self) -> bool {
		self.0().eq(&other.0())
	}
}

impl<E: Encode + sp_std::fmt::Debug> sp_std::fmt::Debug for FnEncode<E> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		self.0().fmt(f)
	}
}

#[cfg(feature = "std")]
impl<E: Encode + serde::Serialize> serde::Serialize for FnEncode<E> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		self.0().serialize(serializer)
	}
}

/// All the metadata about an outer event.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct OuterEventMetadata {
	pub name: DecodeDifferentStr,
	pub events: DecodeDifferentArray<
		(&'static str, FnEncode<&'static [EventMetadata]>),
		(StringBuf, Vec<EventMetadata>)
	>,
}

/// All the metadata about an event.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct EventMetadata {
	pub name: DecodeDifferentStr,
	pub arguments: DecodeDifferentArray<&'static str, StringBuf>,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about one storage entry.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct StorageEntryMetadata {
	pub name: DecodeDifferentStr,
	pub modifier: StorageEntryModifier,
	pub ty: StorageEntryType,
	pub default: ByteGetter,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about one module constant.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ModuleConstantMetadata {
	pub name: DecodeDifferentStr,
	pub ty: DecodeDifferentStr,
	pub value: ByteGetter,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about a module error.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ErrorMetadata {
	pub name: DecodeDifferentStr,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about errors in a module.
pub trait ModuleErrorMetadata {
	fn metadata() -> &'static [ErrorMetadata];
}

impl ModuleErrorMetadata for &'static str {
	fn metadata() -> &'static [ErrorMetadata] {
		&[]
	}
}

/// A technical trait to store lazy initiated vec value as static dyn pointer.
pub trait DefaultByte: Send + Sync {
	fn default_byte(&self) -> Vec<u8>;
}

/// Wrapper over dyn pointer for accessing a cached once byte value.
#[derive(Clone)]
pub struct DefaultByteGetter(pub &'static dyn DefaultByte);

/// Decode different for static lazy initiated byte value.
pub type ByteGetter = DecodeDifferent<DefaultByteGetter, Vec<u8>>;

impl Encode for DefaultByteGetter {
	fn encode_to<W: Output + ?Sized>(&self, dest: &mut W) {
		self.0.default_byte().encode_to(dest)
	}
}

impl codec::EncodeLike for DefaultByteGetter {}

impl PartialEq<DefaultByteGetter> for DefaultByteGetter {
	fn eq(&self, other: &DefaultByteGetter) -> bool {
		let left = self.0.default_byte();
		let right = other.0.default_byte();
		left.eq(&right)
	}
}

impl Eq for DefaultByteGetter { }

#[cfg(feature = "std")]
impl serde::Serialize for DefaultByteGetter {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		self.0.default_byte().serialize(serializer)
	}
}

impl sp_std::fmt::Debug for DefaultByteGetter {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		self.0.default_byte().fmt(f)
	}
}

/// Hasher used by storage maps
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageHasher {
	Blake2_128,
	Blake2_256,
	Blake2_128Concat,
	Twox128,
	Twox256,
	Twox64Concat,
	Identity,
}

/// A storage entry type.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageEntryType {
	Plain(DecodeDifferentStr),
	Map {
		hasher: StorageHasher,
		key: DecodeDifferentStr,
		value: DecodeDifferentStr,
		// is_linked flag previously, unused now to keep backwards compat
		unused: bool,
	},
	DoubleMap {
		hasher: StorageHasher,
		key1: DecodeDifferentStr,
		key2: DecodeDifferentStr,
		value: DecodeDifferentStr,
		key2_hasher: StorageHasher,
	},
}

/// A storage entry modifier.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageEntryModifier {
	Optional,
	Default,
}

/// All metadata of the storage.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct StorageMetadata {
	/// The common prefix used by all storage entries.
	pub prefix: DecodeDifferent<&'static str, StringBuf>,
	pub entries: DecodeDifferent<&'static [StorageEntryMetadata], Vec<StorageEntryMetadata>>,
}

/// Metadata prefixed by a u32 for reserved usage
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct RuntimeMetadataPrefixed(pub u32, pub RuntimeMetadata);

/// Metadata of the extrinsic used by the runtime.
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ExtrinsicMetadata {
	/// Extrinsic version.
	pub version: u8,
	/// The signed extensions in the order they appear in the extrinsic.
	pub signed_extensions: Vec<DecodeDifferentStr>,
}

/// The metadata of a runtime.
/// The version ID encoded/decoded through
/// the enum nature of `RuntimeMetadata`.
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum RuntimeMetadata {
	/// Unused; enum filler.
	V0(RuntimeMetadataDeprecated),
	/// Version 1 for runtime metadata. No longer used.
	V1(RuntimeMetadataDeprecated),
	/// Version 2 for runtime metadata. No longer used.
	V2(RuntimeMetadataDeprecated),
	/// Version 3 for runtime metadata. No longer used.
	V3(RuntimeMetadataDeprecated),
	/// Version 4 for runtime metadata. No longer used.
	V4(RuntimeMetadataDeprecated),
	/// Version 5 for runtime metadata. No longer used.
	V5(RuntimeMetadataDeprecated),
	/// Version 6 for runtime metadata. No longer used.
	V6(RuntimeMetadataDeprecated),
	/// Version 7 for runtime metadata. No longer used.
	V7(RuntimeMetadataDeprecated),
	/// Version 8 for runtime metadata. No longer used.
	V8(RuntimeMetadataDeprecated),
	/// Version 9 for runtime metadata. No longer used.
	V9(RuntimeMetadataDeprecated),
	/// Version 10 for runtime metadata. No longer used.
	V10(RuntimeMetadataDeprecated),
	/// Version 11 for runtime metadata.
	V11(RuntimeMetadataV11),
}

/// Enum that should fail.
#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize))]
pub enum RuntimeMetadataDeprecated { }

impl Encode for RuntimeMetadataDeprecated {
	fn encode_to<W: Output + ?Sized>(&self, _dest: &mut W) {}
}

impl codec::EncodeLike for RuntimeMetadataDeprecated {}

#[cfg(feature = "std")]
impl Decode for RuntimeMetadataDeprecated {
	fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
		Err("Decoding is not supported".into())
	}
}

/// The metadata of a runtime.
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct RuntimeMetadataV11 {
	/// Metadata of all the modules.
	pub modules: DecodeDifferentArray<ModuleMetadata>,
	/// Metadata of the extrinsic.
	pub extrinsic: ExtrinsicMetadata,
}

/// All metadata about an runtime module.
#[derive(Clone, PartialEq, Eq, Encode)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ModuleMetadata {
	pub name: DecodeDifferentStr,
	pub storage: Option<DecodeDifferent<FnEncode<StorageMetadata>, StorageMetadata>>,
	pub calls: ODFnA<FunctionMetadata>,
	pub event: ODFnA<EventMetadata>,
	pub constants: DFnA<ModuleConstantMetadata>,
	pub errors: DFnA<ErrorMetadata>,
}

type ODFnA<T> = Option<DFnA<T>>;
type DFnA<T> = DecodeDifferent<FnEncode<&'static [T]>, Vec<T>>;

impl Into<frame_metadata::OpaqueMetadata> for RuntimeMetadataPrefixed {
	fn into(self) -> frame_metadata::OpaqueMetadata {
		frame_metadata::OpaqueMetadata(self.encode())
	}
}

impl Into<RuntimeMetadataPrefixed> for RuntimeMetadataV11 {
	fn into(self) -> RuntimeMetadataPrefixed {
		RuntimeMetadataPrefixed(META_RESERVED, RuntimeMetadata::V11(self))
	}
}
