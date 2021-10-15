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

mod decode_type;
mod extrinsic_bytes;

use crate::metadata::Metadata;
use crate::value::Value;
use codec::{Compact, Decode};
use decode_type::{decode_type, decode_type_by_id, DecodeTypeError};
use extrinsic_bytes::{AllExtrinsicBytes, ExtrinsicBytesError};
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};

/**
Given some [`Metadata`] obtained from a substrate node, this allows you to decode
various SCALE encoded values from that node.

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

let extrinsics = decoder.decode_extrinsics(&extrinsics_bytes).unwrap();

assert_eq!(extrinsics.len(), 3);
for ext in extrinsics {
	assert_eq!(ext.pallet, "Auctions".to_string());
	assert_eq!(ext.call, "bid".to_string());
}
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

let extrinsic = decoder.decode_extrinsic(&extrinsic_bytes).unwrap();

assert_eq!(extrinsic.pallet, "Auctions".to_string());
assert_eq!(extrinsic.call, "bid".to_string());
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

// Decode the "unwrapped" (no length prefix) extrinsic like so:
let extrinsic = decoder.decode_unwrapped_extrinsic(&extrinsic_bytes).unwrap();

assert_eq!(extrinsic.pallet, "Auctions".to_string());
assert_eq!(extrinsic.call, "bid".to_string());
```
*/
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
	#[error("Failed to decode unsupported extrinsic version '{0}'")]
	CannotDecodeExtrinsicVersion(u8),
	#[error("Cannot find call corresponding to extrinsic with pallet index {0} and call index {1}")]
	CannotFindCall(u8, u8),
	#[error("Failed to decode extrinsic: cannot find type ID {0}")]
	CannotFindType(u32),
	/// Returned from either [`Decoder::decode_extrinsic`] or [`Decoder::decode_extrinsics`], this consists
	/// of the extrinsic we think we decoded, and the number of bytes left in the slice provided (which we
	/// expected to entirely consume, but did not).
	#[error("Decoding an extrinsic should consume all bytes, but {0} bytes remain")]
	LateEof(usize, Extrinsic),
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
	pub fn decode_extrinsics(&self, data: &[u8]) -> Result<Vec<Extrinsic>, (Vec<Extrinsic>, DecodeError)> {
		let extrinsic_bytes = AllExtrinsicBytes::new(data).map_err(|e| (Vec::new(), e.into()))?;

		log::trace!("Decoding {} Total Extrinsics.", extrinsic_bytes.len());

		let mut out = Vec::with_capacity(extrinsic_bytes.len());
		for (idx, res) in extrinsic_bytes.iter().enumerate() {
			let single_extrinsic = match res {
				Ok(bytes) => bytes,
				Err(e) => return Err((out, e.into())),
			};

			log::trace!("Extrinsic {}:{:?}", idx, single_extrinsic.bytes());

			let ext = match self.decode_unwrapped_extrinsic(single_extrinsic.bytes()) {
				Ok(ext) => ext,
				Err(DecodeError::LateEof(remaining, ext)) => {
					// Returned from `decode_extrinsics`, we want the remaining bytes to be relative
					// to the data slice passed in, and not the single extrinsic slice.
					return Err((out, DecodeError::LateEof(single_extrinsic.remaining() + remaining, ext)));
				}
				Err(e) => return Err((out, e)),
			};

			out.push(ext);
		}
		Ok(out)
	}

	/// Decode a SCALE encoded extrinsic against the metadata provided. Conceptually, an individual extrinsic is expected
	/// to be represented in terms of a compact encoded count of its length in bytes, and then the actual extrinsic
	/// information (the optional signature and call data).
	///
	/// If your extrinsic is not prefixed by its byte length, use [`Decoder::decode_unwrapped_extrinsic`] to
	/// decode it instead.
	pub fn decode_extrinsic(&self, mut data: &[u8]) -> Result<Extrinsic, DecodeError> {
		let data = &mut data;

		// Ignore the expected extrinsic length here at the moment, since `decode_unwrapped_extrinsic` will
		// error accordingly if the wrong number of bytes are consumed.
		let _len = <Compact<u32>>::decode(data)?;

		self.decode_unwrapped_extrinsic(*data)
	}

	/// Decode a SCALE encoded extrinsic against the metadata provided. Unlike [`Decoder::decode_extrinsic`], this
	/// assumes that the bytes provided do *not* start with a compact encoded count of the extrinsic byte length
	/// (ie, the extrinsic has been "unwrapped" already, and here we deal directly with the dignature and call data).
	pub fn decode_unwrapped_extrinsic(&self, mut data: &[u8]) -> Result<Extrinsic, DecodeError> {
		// If we use a mutable reference to the data, `decode` functions will autoamtically
		// update the bytes pointed to as they decode, to "move the cursor along".
		let data = &mut data;

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
			true => Some(decode_v4_signature(data, &self.metadata)?),
			false => None,
		};

		// Pluck out the u8's representing the pallet and call enum next.
		if data.len() < 2 {
			return Err(DecodeError::EarlyEof("expected at least 2 more bytes for the pallet/call index"));
		}
		let pallet_index = u8::decode(data)?;
		let call_index = u8::decode(data)?;
		log::trace!("pallet index: {}, call index: {}", pallet_index, call_index);

		// Work out which call the extrinsic data represents and get type info for it:
		let (pallet_name, call) = match self.metadata.call_by_variant_index(pallet_index, call_index) {
			Some(call) => call,
			None => return Err(DecodeError::CannotFindCall(pallet_index, call_index)),
		};

		// Decode each of the argument values in the extrinsic:
		let mut arguments = vec![];
		for arg in call.args() {
			let ty = self.metadata.types().resolve(arg.id()).ok_or_else(|| DecodeError::CannotFindType(arg.id()))?;
			let val = match decode_type(data, ty, self.metadata.types()) {
				Ok(val) => val,
				Err(err) => return Err(err.into()),
			};
			arguments.push(val);
		}

		let ext = Extrinsic { pallet: pallet_name.to_owned(), call: call.name().to_owned(), signature, arguments };

		// If there's data left to consume, it likely means we screwed up decoding:
		if !data.is_empty() {
			return Err(DecodeError::LateEof(data.len(), ext));
		}

		// Return a composite type representing the extrinsic arguments:
		Ok(ext)
	}
}

#[derive(Debug, Clone)]
pub struct Extrinsic {
	/// The name of the pallet that the extrinsic called into
	pub pallet: String,
	/// The name of the call made
	pub call: String,
	/// The signature and signed extensions (if any) associated with the extrinsic
	pub signature: Option<ExtrinsicSignature>,
	/// The arguments to pass to the call
	pub arguments: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicSignature {
	/// Address the extrinsic is being sent from
	pub address: MultiAddress<AccountId32, u32>,
	/// Signature to prove validity
	pub signature: MultiSignature,
	/// Signed extensions, which can vary by node. Here, we
	/// return the name and value of each.
	pub extensions: Vec<(String, Value)>,
}

fn decode_v4_signature<'a>(data: &mut &'a [u8], metadata: &Metadata) -> Result<ExtrinsicSignature, DecodeError> {
	let address = <MultiAddress<AccountId32, u32>>::decode(data)?;
	let signature = MultiSignature::decode(data)?;
	let extensions = metadata
		.extrinsic()
		.signed_extensions()
		.iter()
		.map(|ext| {
			let val = decode_type_by_id(data, &ext.ty, metadata.types())?;
			let name = ext.identifier.to_string();
			Ok((name, val))
		})
		.collect::<Result<_, DecodeError>>()?;

	Ok(ExtrinsicSignature { address, signature, extensions })
}
