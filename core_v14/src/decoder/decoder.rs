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

use super::decode_type::{decode_type, decode_type_by_id, DecodeTypeError};
use super::extrinsic_bytes::{ExtrinsicBytes, ExtrinsicBytesError};
use crate::metadata::Metadata;
use crate::substrate_value::SubstrateValue;
use codec::Decode;
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};

pub struct Decoder {
	metadata: Metadata,
}

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
	LateEof(usize, GenericExtrinsic),
}

impl Decoder {
	/// Create a new decoder using the provided metadata.
	pub fn with_metadata(metadata: Metadata) -> Decoder {
		Decoder { metadata: metadata.into() }
	}

	/// Return the metadata back, consuming the decoder.
	pub fn into_metadata(self) -> Metadata {
		self.metadata
	}

	/// Decode a SCALE encoded vector of extrinsics against the metadata provided. We get back a vector
	/// of the decoded extrinsics on success, else we get an error containing the extrinsics that were decoded
	/// successfully before the error, and then the error itself.
	pub fn decode_extrinsics(
		&self,
		data: &[u8],
	) -> Result<Vec<GenericExtrinsic>, (Vec<GenericExtrinsic>, DecodeError)> {
		let extrinsic_bytes = ExtrinsicBytes::new(data).map_err(|e| (Vec::new(), e.into()))?;

		log::trace!("Decoding {} Total Extrinsics.", extrinsic_bytes.len());

		let mut out = Vec::with_capacity(extrinsic_bytes.len());
		for (idx, res) in extrinsic_bytes.iter().enumerate() {
			let single_extrinsic = match res {
				Ok(bytes) => bytes,
				Err(e) => return Err((out, e.into())),
			};

			log::trace!("Extrinsic {}:{:?}", idx, single_extrinsic.bytes());

			let ext = match self.decode_extrinsic(single_extrinsic.bytes()) {
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

	/// Decode a SCALE encoded extrinsic against the metadata provided
	pub fn decode_extrinsic(&self, mut data: &[u8]) -> Result<GenericExtrinsic, DecodeError> {
		// A mutably pointer to the slice, so that we can update out view into the bytes as
		// we decode things from it.
		let data = &mut data;

		if data.len() == 0 {
			return Err(DecodeError::EarlyEof("extrinsic length should be > 0"));
		}

		// V4 extrinsics (the format we can decode here) are laid out roughly as follows:
		//
		// first byte: abbbbbbb (a = 0 for unsigned, 1 for signed, b = version)
		//
		// signature, which is made up of (in order):
		// - sp_runtime::MultiAddress enum (sender)
		// - sp_runtime::MultiSignature enum
		// - For polkadot, these extensions (but can vary by chain):
		//   - sp_runtime::generic::Era enum
		//   - compact encoded u32 (nonce; prior transaction count)
		//   - compact encoded u128 (tip paid to block producer/treasury)
		//
		// call, which is made up roughly of:
		// - enum pallet index (for pallets variant)
		// - call index (for inner variant)
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
			let ty = self.metadata.types().resolve(arg.id()).ok_or(DecodeError::CannotFindType(arg.id()))?;
			let val = match decode_type(data, &ty, self.metadata.types()) {
				Ok(val) => val,
				Err(err) => return Err(err.into()),
			};
			arguments.push(val);
		}

		let ext =
			GenericExtrinsic { pallet: pallet_name.to_owned(), call: call.name().to_owned(), signature, arguments };

		// If there's data left to consume, it likely means we screwed up decoding:
		if !data.is_empty() {
			return Err(DecodeError::LateEof(data.len(), ext));
		}

		// Return a composite type representing the extrinsic arguments:
		Ok(ext)
	}
}

#[derive(Debug, Clone)]
pub struct GenericExtrinsic {
	/// The name of the pallet that the extrinsic called into
	pub pallet: String,
	/// The name of the call made
	pub call: String,
	/// The signature (if any) associated with the extrinsic
	pub signature: Option<ExtrinsicSignature>,
	/// The arguments to pass to the call
	pub arguments: Vec<SubstrateValue>,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicSignature {
	/// Address the extrinsic is being sent from
	pub address: MultiAddress<AccountId32, u32>,
	/// Signature to prove validity
	pub signature: MultiSignature,
	/// Signed extensions, which can vary by node. Here, we
	/// return the name and value of each.
	pub extensions: Vec<(String, SubstrateValue)>,
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
