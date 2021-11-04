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

/*!
Given some [`Metadata`] obtained from a substrate node, this module exposes the functionality to
decode various SCALE encoded values, such as extrinsics, that are compatible with that metadata.

# Examples

## Decoding a block of extrinsics

Conceptually, a block of extrinsics is the SCALE encoded version of `Vec<Vec<u8>>`,
where the inner bytes are the SCALE encoded bytes for a single unwrapped extrinsic.
Use [`Decoder::decode_extrinsics`] to decode:

```rust
use hex;
use core_v14::{ Metadata, Decoder };

let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
let decoder = Decoder::with_metadata(metadata);

// the same extrinsic repeated 3 times:
let extrinsics_hex = "0x0C2004480104080c10142004480104080c10142004480104080c1014";
let extrinsics_bytes = hex::decode(extrinsics_hex.strip_prefix("0x").unwrap()).unwrap();
let extrinsics_cursor = &mut &*extrinsics_bytes;

let extrinsics = decoder.decode_extrinsics(extrinsics_cursor).unwrap();

assert_eq!(extrinsics_cursor.len(), 0);
assert_eq!(extrinsics.len(), 3);
```

## Decoding a single extrinsic

Conceptually, a single extrinsic looks like `Vec<u8>`; it's the actual extrinsic data,
prefixed with the length of this extrinsic data. Use [`Decoder::decode_extrinsic`] to decode:

```rust
use hex;
use core_v14::{ Metadata, Decoder };

let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
let decoder = Decoder::with_metadata(metadata);

let extrinsic_hex = "0x2004480104080c1014";
let extrinsic_bytes = hex::decode(extrinsic_hex.strip_prefix("0x").unwrap()).unwrap();
let extrinsic_cursor = &mut &*extrinsic_bytes;

let extrinsic = decoder.decode_extrinsic(extrinsic_cursor).unwrap();

assert_eq!(extrinsic_cursor.len(), 0);
assert_eq!(extrinsic.call_data.pallet_name, "Auctions");
assert_eq!(&*extrinsic.call_data.ty.name(), "bid");
```

## Decoding call data

An "unwrapped" extrinsic is essentially the call data (that you can see by using the polkadot.js UI),
prefixed with either `0x04` denoting the version (4) and no signature, or `0x84` to denote
the version number and then some signature bytes. A normal extrinsic is also prefixed by a compact
encoding of its length in bytes.

So, to convert any call data into something that can be decoded as an unwrapped extrinsic, simply prepend
`0x04` to the encode bytes, and use [`Decoder::decode_unwrapped_extrinsic`] to decode it:

```rust
use hex;
use core_v14::{ Metadata, Decoder };

let metadata_scale_encoded = include_bytes!("../../tests/data/v14_metadata_polkadot.scale");
let metadata = Metadata::from_bytes(metadata_scale_encoded).unwrap();
let decoder = Decoder::with_metadata(metadata);

let call_data_hex = "0x480104080c1014";
// Prepend 04 to the call data hex to create a valid, unwrapped (no length prefix)
// and unsigned extrinsic:
let extrinsic_hex = "0x04480104080c1014";

let extrinsic_bytes = hex::decode(extrinsic_hex.strip_prefix("0x").unwrap()).unwrap();
let extrinsic_cursor = &mut &*extrinsic_bytes;

// Decode the "unwrapped" (no length prefix) extrinsic like so:
let extrinsic = decoder.decode_unwrapped_extrinsic(extrinsic_cursor).unwrap();

assert_eq!(extrinsic_cursor.len(), 0);
assert_eq!(extrinsic.call_data.pallet_name, "Auctions");
assert_eq!(&*extrinsic.call_data.ty.name(), "bid");
```
*/
mod decode_type;
mod extrinsic_bytes;

use crate::metadata::Metadata;
use crate::value::Value;
use codec::{Compact, Decode};
use decode_type::{decode_type, decode_type_by_id, DecodeTypeError};
use extrinsic_bytes::{AllExtrinsicBytes, ExtrinsicBytesError};
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};
use std::borrow::Cow;

#[derive(Debug)]
pub struct Decoder {
	metadata: Metadata,
}

/// An enum of the possible errors that can be returned from attempting to decode bytes
/// using the [`Decoder`] methods.
#[derive(Clone, Debug, thiserror::Error)]
pub enum DecodeError {
	#[error("Failed to parse the provided vector of extrinsics: {0}")]
	UnexpectedExtrinsicsShape(#[from] ExtrinsicBytesError),
	#[error("Failed to decode: {0}")]
	CodecError(#[from] codec::Error),
	#[error("Failed to decode type: {0}")]
	DecodeTypeError(#[from] DecodeTypeError),
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

impl Decoder {
	/// Create a new decoder using the provided metadata.
	pub fn with_metadata(metadata: Metadata) -> Decoder {
		Decoder { metadata }
	}

	/// Return the metadata back, consuming the decoder.
	pub fn into_metadata(self) -> Metadata {
		self.metadata
	}

	/// Decode a SCALE encoded vector of extrinsics against the metadata provided. Conceptually, extrinsics are
	/// expected to be provided in a SCALE-encoded form equivalent to `Vec<(Compact<u32>,Extrinsic)>`; in other words, we
	/// start with a compact encoded count of how many extrinsics exist, and then each extrinsic is prefixed by
	/// a compact encoding of its byte length.
	pub fn decode_extrinsics<'a>(
		&'a self,
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
			let ext = match self.decode_unwrapped_extrinsic(bytes) {
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
	/// If your extrinsic is not prefixed by its byte length, use [`Decoder::decode_unwrapped_extrinsic`] to
	/// decode it instead.
	pub fn decode_extrinsic<'a>(&'a self, data: &mut &[u8]) -> Result<Extrinsic<'a>, DecodeError> {
		// Ignore the expected extrinsic length here at the moment, since `decode_unwrapped_extrinsic` will
		// error accordingly if the wrong number of bytes are consumed.
		let _len = <Compact<u32>>::decode(data)?;

		self.decode_unwrapped_extrinsic(data)
	}

	/// Decode a SCALE encoded extrinsic against the metadata provided. Unlike [`Decoder::decode_extrinsic`], this
	/// assumes that the bytes provided do *not* start with a compact encoded count of the extrinsic byte length
	/// (ie, the extrinsic has been "unwrapped" already, and here we deal directly with the signature and call data).
	pub fn decode_unwrapped_extrinsic<'a>(&'a self, data: &mut &[u8]) -> Result<Extrinsic<'a>, DecodeError> {
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
			true => Some(self.decode_signature(data)?),
			false => None,
		};

		// Finally, decode the call data.
		let call_data = self.decode_call_data(data)?;

		Ok(Extrinsic { call_data, signature })
	}

	/// Decode SCALE encoded call data. Conceptually, this is expected to take the form of
	/// `(u8, u8, arguments)`, where the specific pallet call variant indexes are determined by
	/// the `u8`s, and then arguments according to the specific variant are expected to follow.
	pub fn decode_call_data<'a>(&'a self, data: &mut &[u8]) -> Result<CallData<'a>, DecodeError> {
		// Pluck out the u8's representing the pallet and call enum next.
		if data.len() < 2 {
			return Err(DecodeError::EarlyEof("expected at least 2 more bytes for the pallet/call index"));
		}
		let pallet_index = u8::decode(data)?;
		let call_index = u8::decode(data)?;
		log::trace!("pallet index: {}, call index: {}", pallet_index, call_index);

		// Work out which call the extrinsic data represents and get type info for it:
		let (pallet_name, variant) = match self.metadata.call_variant_by_enum_index(pallet_index, call_index) {
			Some(call) => call,
			None => return Err(DecodeError::CannotFindCall(pallet_index, call_index)),
		};

		// Decode each of the argument values in the extrinsic:
		let arguments: Vec<_> = variant
			.fields()
			.iter()
			.map(|field| {
				let type_id = field.ty().id();
				let ty = self.metadata.types().resolve(type_id).ok_or(DecodeError::CannotFindType(type_id))?;
				decode_type(data, ty, self.metadata.types()).map_err(DecodeError::DecodeTypeError)
			})
			.collect::<Result<_, _>>()?;

		Ok(CallData { pallet_name: Cow::Borrowed(pallet_name), ty: Cow::Borrowed(variant), arguments })
	}

	/// Decode the SCALE encoded data that, once signed, is used to construct a signed extrinsic. The encoded payload has the following shape:
	/// `(call_data, signed_extensions, additional_signed)`.
	pub fn decode_signer_payload<'a>(&'a self, data: &mut &[u8]) -> Result<SignerPayload<'a>, DecodeError> {
		let call_data = self.decode_call_data(data)?;
		let signed_extensions = self.decode_signed_extensions(data)?;
		let additional_signed = self.decode_additional_signed(data)?;
		let extensions = signed_extensions
			.into_iter()
			.zip(additional_signed)
			.map(|((name, extension), (_, additional))| (name, SignedExtensionWithAdditional { additional, extension }))
			.collect();

		Ok(SignerPayload { call_data, extensions })
	}

	/// Decode a SCALE encoded extrinsic signature.
	fn decode_signature<'a>(&'a self, data: &mut &[u8]) -> Result<ExtrinsicSignature<'a>, DecodeError> {
		let address = <MultiAddress<AccountId32, u32>>::decode(data)?;
		let signature = MultiSignature::decode(data)?;
		let extensions = self.decode_signed_extensions(data)?;

		Ok(ExtrinsicSignature { address, signature, extensions })
	}

	/// Decode the signed extensions.
	fn decode_signed_extensions<'a>(&'a self, data: &mut &[u8]) -> Result<Vec<(Cow<'a, str>, Value)>, DecodeError> {
		self.metadata
			.extrinsic()
			.signed_extensions()
			.iter()
			.map(|ext| {
				let val = decode_type_by_id(data, &ext.ty, self.metadata.types())?;
				let name = Cow::Borrowed(&*ext.identifier);
				Ok((name, val))
			})
			.collect()
	}

	/// Decode the additional signed data. This isn't used for decoding extrinsics, but instead for
	/// decoding the data that a user signs.
	fn decode_additional_signed<'a>(&'a self, data: &mut &[u8]) -> Result<Vec<(Cow<'a, str>, Value)>, DecodeError> {
		self.metadata
			.extrinsic()
			.signed_extensions()
			.iter()
			.map(|ext| {
				let val = decode_type_by_id(data, &ext.additional_signed, self.metadata.types())?;
				let name = Cow::Borrowed(&*ext.identifier);
				Ok((name, val))
			})
			.collect()
	}
}

/// Decoded call data and associated type information.
#[derive(Debug, Clone, PartialEq)]
pub struct CallData<'a> {
	/// The name of the pallet
	pub pallet_name: Cow<'a, str>,
	/// The type information for this call (including the name
	/// of the call and information about each argument)
	pub ty: Cow<'a, scale_info::Variant<scale_info::form::PortableForm>>,
	/// The decoded argument data
	pub arguments: Vec<Value>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Extrinsic<'a> {
	/// Decoded call data and associated type information about the call.
	pub call_data: CallData<'a>,
	/// The signature and signed extensions (if any) associated with the extrinsic
	pub signature: Option<ExtrinsicSignature<'a>>,
}

impl<'a> Extrinsic<'a> {
	pub fn into_owned(self) -> Extrinsic<'static> {
		Extrinsic { call_data: self.call_data.into_owned(), signature: self.signature.map(|s| s.into_owned()) }
	}
}

/// The signature information embedded in an extrinsic.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtrinsicSignature<'a> {
	/// Address the extrinsic is being sent from
	pub address: MultiAddress<AccountId32, u32>,
	/// Signature to prove validity
	pub signature: MultiSignature,
	/// Signed extensions, which can vary by node. Here, we
	/// return the name and value of each.
	pub extensions: Vec<(Cow<'a, str>, Value)>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct SignerPayload<'a> {
	/// Decoded call data and associated type information about the call.
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
#[derive(Debug, Clone, PartialEq)]
pub struct SignedExtensionWithAdditional {
	/// The signed extension value at this position
	pub extension: Value,
	/// The additional signed value at this position
	pub additional: Value,
}
