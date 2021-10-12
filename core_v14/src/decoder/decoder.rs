use super::decode_type::{decode_type, DecodeTypeError};
use super::extrinsic_bytes::{ExtrinsicBytes, ExtrinsicBytesError};
use crate::metadata::{Metadata};
use crate::substrate_value::SubstrateValue;
use sp_runtime::{ MultiAddress, MultiSignature, AccountId32, generic::Era };
use codec::{ Decode };

pub struct Decoder {
	metadata: Metadata,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum DecodeError<'a> {
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
	#[error("Decoding an extrinsic should consume all bytes, but {} bytes remain", .0.len())]
	LateEof(&'a [u8], GenericExtrinsic)
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

	/// Decode a SCALE encoded vector of extrinsics against the metadata provided
	pub fn decode_extrinsics<'a>(&self, data: &'a [u8]) -> Result<Vec<GenericExtrinsic>, DecodeError<'a>> {
		let extrinsic_bytes = ExtrinsicBytes::new(data)?;
		log::trace!("Decoding {} Total Extrinsics.", extrinsic_bytes.len());

		let mut out = Vec::with_capacity(extrinsic_bytes.len());
		for (idx, res) in extrinsic_bytes.iter().enumerate() {
			let bytes = res?;
			log::trace!("Extrinsic {}:{:?}", idx, bytes);
			out.push(self.decode_extrinsic(bytes)?);
		}
		Ok(out)
	}

	/// Decode a SCALE encoded extrinsic against the metadata provided
	pub fn decode_extrinsic<'a>(&self, mut data: &'a [u8]) -> Result<GenericExtrinsic, DecodeError<'a>> {

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
		// - sp_runtime::generic::Era enum
		// - compact encoded u32 (nonce; prior transaction count)
		// - compact encoded u128 (tip paid to block producer/treasury)
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
            true => Some(ExtrinsicSignature::decode(data)?),
            false => None
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
			let ty = self.metadata.resolve_type(arg).ok_or(DecodeError::CannotFindType(arg.id()))?;
			let val = match decode_type(data, &ty, &self.metadata) {
				Ok(val) => val,
				Err(err) => return Err(err.into()),
			};
			arguments.push(val);
		}

		let ext = GenericExtrinsic { pallet: pallet_name.to_owned(), call: call.name().to_owned(), signature, arguments };

		// If there's data left to consume, it likely means we screwed up decoding:
		if !data.is_empty() {
			return Err(DecodeError::LateEof(*data, ext));
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

#[derive(Decode, Debug, Clone)]
pub struct ExtrinsicSignature {
    /// Address the extrinsic is being sent from
    pub address: MultiAddress<AccountId32, u32>,
    /// Signature to prove validity
    pub signature: MultiSignature,
    /// Lifetime of this extrinsic
    pub era: Era,
    /// How many past transactions from this address?
    #[codec(compact)]
    pub nonce: u32,
    /// Tip to help get the extrinsic included faster
    #[codec(compact)]
    pub tip: u128
}
