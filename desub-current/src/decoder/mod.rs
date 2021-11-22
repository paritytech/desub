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

//! Given some [`Metadata`] obtained from a substrate node, this module exposes the functionality to
//! decode various SCALE encoded values, such as extrinsics, that are compatible with that metadata.
//!
//! See [`decode_extrinsics`], [`decode_extrinsic`], and [`decode_unwrapped_extrinsic`] for the most
//! common extrinsic decoding needs.
//!
//! See [`decode_storage()`] and then the documentation on [`StorageDecoder`] to decode storage lookups.

mod decode_storage;
mod decode_value;
mod extrinsic_bytes;

use crate::metadata::Metadata;
use crate::value::Value;
use crate::TypeId;
use codec::{Compact, Decode};
use extrinsic_bytes::{AllExtrinsicBytes, ExtrinsicBytesError};
use serde::Serialize;
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};
use std::borrow::Cow;

// Re-export the DecodeValueError here, which we expose in our global `DecodeError` enum.
pub use decode_value::DecodeValueError;

// Re-export storage related types that are part of our public interface.
pub use decode_storage::{
	StorageDecodeError, StorageDecoder, StorageEntry, StorageEntryType, StorageHasher, StorageMapKey,
};

/// An enum of the possible errors that can be returned from attempting to decode bytes
/// using the functions in this module.
#[derive(Clone, Debug, thiserror::Error)]
pub enum DecodeError {
	#[error("Failed to parse the provided vector of extrinsics: {0}")]
	UnexpectedExtrinsicsShape(#[from] ExtrinsicBytesError),
	#[error("Failed to decode: {0}")]
	CodecError(#[from] codec::Error),
	#[error("Failed to decode type: {0}")]
	DecodeValueError(#[from] DecodeValueError),
	#[error("Failed to decode: expected more data")]
	EarlyEof(&'static str),
	#[error("Failed to decode extrinsics: {0} bytes of the input were not consumed")]
	ExcessBytes(usize),
	#[error("Failed to decode unsupported extrinsic version '{0}'")]
	CannotDecodeExtrinsicVersion(u8),
	#[error("Cannot find call corresponding to extrinsic with pallet index {0} and call index {1}")]
	CannotFindCall(u8, u8),
	#[error("Failed to decode extrinsic: cannot find type ID {0}")]
	CannotFindType(u32),
}

/// Decode a single [`Value`] from a piece of scale encoded data, given some metadata and the ID of the type that we
/// are expecting it to decode into.
pub fn decode_value_by_id<'a, Id: Into<TypeId>>(
	metadata: &'a Metadata,
	ty: Id,
	data: &mut &[u8],
) -> Result<Value<TypeId>, DecodeValueError> {
	decode_value::decode_value_by_id(data, ty, metadata.types())
}

/// Generate a [`StorageDecoder`] struct which is capable of decoding SCALE encoded storage keys. It's advisable
/// to cache this struct if you are decoding lots of storage entries, since it is non-trivial to create.
///
/// # Example
///
/// ```rust
/// use hex;
/// use desub_current::{
///     Metadata,
///     decoder::{ self, StorageHasher },
///     value::{ Value, ValueDef, Composite, Primitive },
/// };
/// use codec::Encode;
///
/// // Get hold of the metadata (normally by making an RPC call
/// // to the node you want to interact with):
/// let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
/// let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
///
/// // With the help of our metadata, we can create a storage decoder:
/// let storage_decoder = decoder::decode_storage(&metadata);
///
/// // Hex representing a lookup like `System.BlockHash(1000)`
/// // (which contains values of type `[u8; 32]`):
/// let storage_key_hex = "0x26aa394eea5630e07c48ae0c9558cef7a44704b568d21667356a5a050c118746b6ff6f7d467b87a9e8030000";
/// let storage_key_bytes = hex::decode(storage_key_hex.strip_prefix("0x").unwrap()).unwrap();
/// let storage_key_cursor = &mut &*storage_key_bytes;
///
/// // Now, decode our storage key into something meaningful:
/// let entry = storage_decoder.decode_key(&metadata, storage_key_cursor).expect("can decode storage");
/// assert!(storage_key_cursor.is_empty(), "No more bytes expected");
/// assert_eq!(entry.prefix, "System");
/// assert_eq!(entry.name, "BlockHash");
///
/// let keys = entry.details.map_keys();
///
/// // Because the hasher is Twox64Concat, we can see the decoded original map key:
/// assert_eq!(keys.len(), 1);
/// if let StorageHasher::Twox64Concat(val) = keys[0].hasher.clone() {
///     assert_eq!(val.without_context(), Value::u32(1000))
/// }
///
/// // We can also decode values at this storage location using the type info we get back:
/// let bytes = [1u8; 32].encode();
/// let val = decoder::decode_value_by_id(&metadata, &entry.ty, &mut &*bytes).unwrap();
/// # assert_eq!(
/// #     val.without_context(),
/// #     // The Type in this case is something like a newtype-wrapped [u8; 32]:
/// #     Value::unnamed_composite(vec![
/// #         Value::unnamed_composite(vec![Value::u8(1); 32])
/// #     ])
/// # );
/// ```
pub fn decode_storage(metadata: &Metadata) -> StorageDecoder {
	decode_storage::StorageDecoder::generate_from_metadata(metadata)
}

/// Decode a SCALE encoded vector of extrinsics against the metadata provided. Conceptually, extrinsics are
/// expected to be provided in a SCALE-encoded form equivalent to `Vec<(Compact<u32>,Extrinsic)>`; in other words, we
/// start with a compact encoded count of how many extrinsics exist, and then each extrinsic is prefixed by
/// a compact encoding of its byte length.
///
/// # Example
///
/// ```rust
/// use hex;
/// use desub_current::{ Metadata, decoder };
///
/// let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
/// let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
///
/// // the same extrinsic repeated 3 times:
/// let extrinsics_hex = "0x0C2004480104080c10142004480104080c10142004480104080c1014";
/// let extrinsics_bytes = hex::decode(extrinsics_hex.strip_prefix("0x").unwrap()).unwrap();
/// let extrinsics_cursor = &mut &*extrinsics_bytes;
///
/// let extrinsics = decoder::decode_extrinsics(&metadata, extrinsics_cursor).unwrap();
///
/// assert_eq!(extrinsics_cursor.len(), 0);
/// assert_eq!(extrinsics.len(), 3);
/// ```
pub fn decode_extrinsics<'a>(
	metadata: &'a Metadata,
	data: &mut &[u8],
) -> Result<Vec<Extrinsic<'a>>, (Vec<Extrinsic<'a>>, DecodeError)> {
	let extrinsic_bytes = AllExtrinsicBytes::new(*data).map_err(|e| (Vec::new(), e.into()))?;

	log::trace!("Decoding {} Total Extrinsics.", extrinsic_bytes.len());

	let mut out = Vec::with_capacity(extrinsic_bytes.len());
	let mut extrinsics_iter = extrinsic_bytes.iter();
	for res in &mut extrinsics_iter {
		let single_extrinsic = match res {
			Ok(bytes) => bytes,
			Err(e) => return Err((out, e.into())),
		};

		log::trace!("Extrinsic:{:?}", single_extrinsic.bytes());

		let bytes = &mut single_extrinsic.bytes();
		let ext = match decode_unwrapped_extrinsic(metadata, bytes) {
			Ok(ext) => ext,
			Err(e) => return Err((out, e)),
		};

		// If decoding didn't consume all extrinsic bytes, something went wrong.
		// Hand back whatever we have but note the error.
		if !bytes.is_empty() {
			return Err((out, DecodeError::ExcessBytes(bytes.len())));
		}

		out.push(ext);
	}

	// Shift our externally provided data cursor forwards to the right spot,
	// so that one can continue to decode more bytes if there are any:
	*data = extrinsics_iter.remaining_bytes();

	Ok(out)
}

/// Decode a SCALE encoded extrinsic against the metadata provided. Conceptually, an individual extrinsic is expected
/// to be represented in terms of a compact encoded count of its length in bytes, and then the actual extrinsic
/// information (the optional signature and call data).
///
/// If your extrinsic is not prefixed by its byte length, use [`decode_unwrapped_extrinsic`] to
/// decode it instead.
///
/// # Example
///
/// ```rust
/// use hex;
/// use desub_current::{ Metadata, decoder };
///
/// let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
/// let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
///
/// let extrinsic_hex = "0x2004480104080c1014";
/// let extrinsic_bytes = hex::decode(extrinsic_hex.strip_prefix("0x").unwrap()).unwrap();
/// let extrinsic_cursor = &mut &*extrinsic_bytes;
///
/// let extrinsic = decoder::decode_extrinsic(&metadata, extrinsic_cursor).unwrap();
///
/// assert_eq!(extrinsic_cursor.len(), 0);
/// assert_eq!(extrinsic.call_data.pallet_name, "Auctions");
/// assert_eq!(&*extrinsic.call_data.ty.name(), "bid");
/// ```
pub fn decode_extrinsic<'a>(metadata: &'a Metadata, data: &mut &[u8]) -> Result<Extrinsic<'a>, DecodeError> {
	// Ignore the expected extrinsic length here at the moment, since `decode_unwrapped_extrinsic` will
	// error accordingly if the wrong number of bytes are consumed.
	let _len = <Compact<u32>>::decode(data)?;

	decode_unwrapped_extrinsic(metadata, data)
}

/// Decode a SCALE encoded extrinsic against the metadata provided. Unlike [`decode_extrinsic`], this
/// assumes that the bytes provided do *not* start with a compact encoded count of the extrinsic byte length
/// (ie, the extrinsic has been "unwrapped" already, and here we deal directly with the signature and call data).
///
/// # Example
///
/// ```rust
/// use hex;
/// use desub_current::{ Metadata, decoder };
///
/// let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
/// let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
///
/// let call_data_hex = "0x480104080c1014";
/// // Prepend 04 to the call data hex to create a valid, unwrapped (no length prefix)
/// // and unsigned extrinsic:
/// let extrinsic_hex = "0x04480104080c1014";
///
/// let extrinsic_bytes = hex::decode(extrinsic_hex.strip_prefix("0x").unwrap()).unwrap();
/// let extrinsic_cursor = &mut &*extrinsic_bytes;
///
/// // Decode the "unwrapped" (no length prefix) extrinsic like so:
/// let extrinsic = decoder::decode_unwrapped_extrinsic(&metadata, extrinsic_cursor).unwrap();
///
/// assert_eq!(extrinsic_cursor.len(), 0);
/// assert_eq!(extrinsic.call_data.pallet_name, "Auctions");
/// assert_eq!(&*extrinsic.call_data.ty.name(), "bid");
/// ```
pub fn decode_unwrapped_extrinsic<'a>(metadata: &'a Metadata, data: &mut &[u8]) -> Result<Extrinsic<'a>, DecodeError> {
	if data.is_empty() {
		return Err(DecodeError::EarlyEof("unwrapped extrinsic byte length should be > 0"));
	}

	// V4 extrinsics (the format we can decode here) are laid out roughly as follows:
	//
	// first byte: abbbbbbb (a = 0 for unsigned, 1 for signed, b = version)
	//
	// signature, which is made up of (in order):
	// - sp_runtime::MultiAddress enum (sender)
	// - sp_runtime::MultiSignature enum
	// - For polkadot, these extensions (but can vary by chain, so we decode generically):
	//   - sp_runtime::generic::Era enum
	//   - compact encoded u32 (nonce; prior transaction count)
	//   - compact encoded u128 (tip paid to block producer/treasury)
	//
	// call, which is made up roughly of:
	// - u8 enum pallet index (for pallets variant)
	// - u8 call index (for inner variant)
	// - call args (types can be pulled from metadata for each arg we expect)
	//
	// So, we start by getting the version/signed from the first byte and go from there.
	let is_signed = data[0] & 0b1000_0000 != 0;
	let version = data[0] & 0b0111_1111;
	*data = &data[1..];

	// We only know how to decode V4 extrinsics at the moment
	if version != 4 {
		return Err(DecodeError::CannotDecodeExtrinsicVersion(version));
	}

	// If the extrinsic is signed, decode the signature next.
	let signature = match is_signed {
		true => Some(decode_signature(metadata, data)?),
		false => None,
	};

	// Finally, decode the call data.
	let call_data = decode_call_data(metadata, data)?;

	Ok(Extrinsic { call_data, signature })
}

/// Decode SCALE encoded call data. Conceptually, this is expected to take the form of
/// `(u8, u8, arguments)`, where the specific pallet call variant indexes are determined by
/// the `u8`s, and then arguments according to the specific variant are expected to follow.
///
/// # Example
///
/// ```rust
/// use hex;
/// use desub_current::{ Metadata, decoder };
///
/// let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
/// let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
///
/// let call_data_hex = "0x480104080c1014";
///
/// let call_data_bytes = hex::decode(call_data_hex.strip_prefix("0x").unwrap()).unwrap();
/// let call_data_cursor = &mut &*call_data_bytes;
///
/// // Decode the call data like so:
/// let call_data = decoder::decode_call_data(&metadata, call_data_cursor).unwrap();
///
/// assert_eq!(call_data_cursor.len(), 0);
/// assert_eq!(call_data.pallet_name, "Auctions");
/// assert_eq!(&*call_data.ty.name(), "bid");
/// ```
pub fn decode_call_data<'a>(metadata: &'a Metadata, data: &mut &[u8]) -> Result<CallData<'a>, DecodeError> {
	// Pluck out the u8's representing the pallet and call enum next.
	if data.len() < 2 {
		return Err(DecodeError::EarlyEof("expected at least 2 more bytes for the pallet/call index"));
	}
	let pallet_index = u8::decode(data)?;
	let call_index = u8::decode(data)?;
	log::trace!("pallet index: {}, call index: {}", pallet_index, call_index);

	// Work out which call the extrinsic data represents and get type info for it:
	let (pallet_name, variant) = match metadata.call_variant_by_enum_index(pallet_index, call_index) {
		Some(call) => call,
		None => return Err(DecodeError::CannotFindCall(pallet_index, call_index)),
	};

	// Decode each of the argument values in the extrinsic:
	let arguments = variant
		.fields()
		.iter()
		.map(|field| {
			let id = field.ty().id();
			decode_value_by_id(metadata, TypeId::from_u32(id), data).map_err(DecodeError::DecodeValueError)
		})
		.collect::<Result<Vec<_>, _>>()?;

	Ok(CallData { pallet_name: Cow::Borrowed(pallet_name), ty: Cow::Borrowed(variant), arguments })
}

/// Decode the SCALE encoded data that, once signed, is used to construct a signed extrinsic. The encoded payload has the following shape:
/// `(call_data, signed_extensions, additional_signed)`.
pub fn decode_signer_payload<'a>(metadata: &'a Metadata, data: &mut &[u8]) -> Result<SignerPayload<'a>, DecodeError> {
	let call_data = decode_call_data(metadata, data)?;
	let signed_extensions = decode_signed_extensions(metadata, data)?;
	let additional_signed = decode_additional_signed(metadata, data)?;
	let extensions = signed_extensions
		.into_iter()
		.zip(additional_signed)
		.map(|((name, extension), (_, additional))| (name, SignedExtensionWithAdditional { additional, extension }))
		.collect();

	Ok(SignerPayload { call_data, extensions })
}

/// Decode the signature part of a SCALE encoded extrinsic.
///
/// Ordinarily, one should prefer to use [`decode_extrinsic`] directly to decode the entire extrinsic at once.
pub fn decode_signature<'a>(metadata: &'a Metadata, data: &mut &[u8]) -> Result<ExtrinsicSignature<'a>, DecodeError> {
	let address = <MultiAddress<AccountId32, u32>>::decode(data)?;
	let signature = MultiSignature::decode(data)?;
	let extensions = decode_signed_extensions(metadata, data)?;

	Ok(ExtrinsicSignature { address, signature, extensions })
}

/// Decode the signed extensions part of a SCALE encoded extrinsic.
///
/// Ordinarily, one should prefer to use [`decode_extrinsic`] directly to decode the entire extrinsic at once.
#[allow(clippy::type_complexity)]
pub fn decode_signed_extensions<'a>(
	metadata: &'a Metadata,
	data: &mut &[u8],
) -> Result<Vec<(Cow<'a, str>, Value<TypeId>)>, DecodeError> {
	metadata
		.extrinsic()
		.signed_extensions()
		.iter()
		.map(|ext| {
			let val = decode_value_by_id(metadata, &ext.ty, data)?;
			let name = Cow::Borrowed(&*ext.identifier);
			Ok((name, val))
		})
		.collect()
}

/// Decode the additional signed data.
///
/// Ordinarily, one should prefer to use [`decode_signer_payload`], to decode the entire signer payload at once.
#[allow(clippy::type_complexity)]
pub fn decode_additional_signed<'a>(
	metadata: &'a Metadata,
	data: &mut &[u8],
) -> Result<Vec<(Cow<'a, str>, Value<TypeId>)>, DecodeError> {
	metadata
		.extrinsic()
		.signed_extensions()
		.iter()
		.map(|ext| {
			let val = decode_value_by_id(metadata, &ext.additional_signed, data)?;
			let name = Cow::Borrowed(&*ext.identifier);
			Ok((name, val))
		})
		.collect()
}

/// Decoded call data and associated type information.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CallData<'a> {
	/// The name of the pallet
	#[serde(borrow)]
	pub pallet_name: Cow<'a, str>,
	/// The type information for this call (including the name
	/// of the call and information about each argument)
	pub ty: Cow<'a, scale_info::Variant<scale_info::form::PortableForm>>,
	/// The decoded argument data
	pub arguments: Vec<Value<TypeId>>,
}

impl<'a> CallData<'a> {
	pub fn into_owned(self) -> CallData<'static> {
		CallData {
			pallet_name: Cow::Owned(self.pallet_name.into_owned()),
			ty: Cow::Owned(self.ty.into_owned()),
			arguments: self.arguments,
		}
	}
}

/// The result of successfully decoding an extrinsic.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Extrinsic<'a> {
	/// Decoded call data and associated type information about the call.
	#[serde(borrow)]
	pub call_data: CallData<'a>,
	/// The signature and signed extensions (if any) associated with the extrinsic
	#[serde(borrow)]
	pub signature: Option<ExtrinsicSignature<'a>>,
}

impl<'a> Extrinsic<'a> {
	pub fn into_owned(self) -> Extrinsic<'static> {
		Extrinsic { call_data: self.call_data.into_owned(), signature: self.signature.map(|s| s.into_owned()) }
	}
}

/// The signature information embedded in an extrinsic.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExtrinsicSignature<'a> {
	/// Address the extrinsic is being sent from
	#[serde(with = "desub_common::RemoteAddress")]
	pub address: MultiAddress<AccountId32, u32>,
	/// Signature to prove validity
	pub signature: MultiSignature,
	/// Signed extensions, which can vary by node. Here, we
	/// return the name and value of each.
	#[serde(borrow)]
	pub extensions: Vec<(Cow<'a, str>, Value<TypeId>)>,
}

impl<'a> ExtrinsicSignature<'a> {
	pub fn into_owned(self) -> ExtrinsicSignature<'static> {
		ExtrinsicSignature {
			address: self.address,
			signature: self.signature,
			extensions: self.extensions.into_iter().map(|(k, v)| (Cow::Owned(k.into_owned()), v)).collect(),
		}
	}
}

/// The decoded signer payload.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SignerPayload<'a> {
	/// Decoded call data and associated type information about the call.
	#[serde(borrow)]
	pub call_data: CallData<'a>,
	/// Signed extensions as well as additional data to be signed. These
	/// are packaged together in the metadata.
	pub extensions: Vec<(Cow<'a, str>, SignedExtensionWithAdditional)>,
}

impl<'a> SignerPayload<'a> {
	pub fn into_owned(self) -> SignerPayload<'static> {
		SignerPayload {
			call_data: self.call_data.into_owned(),
			extensions: self.extensions.into_iter().map(|(k, v)| (Cow::Owned(k.into_owned()), v)).collect(),
		}
	}
}

/// The decoded signed extensions and additional data.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SignedExtensionWithAdditional {
	/// The signed extension value at this position
	pub extension: Value<TypeId>,
	/// The additional signed value at this position
	pub additional: Value<TypeId>,
}
