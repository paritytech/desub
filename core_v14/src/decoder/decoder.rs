use crate::metadata::{Metadata, MetadataError};
use crate::substrate_type::{CompositeType, SubstrateType};
use crate::substrate_value::SubstrateValue;
use super::extrinsic_bytes::{ ExtrinsicBytes, ExtrinsicBytesError };
use super::decode_type::{ decode_type, DecodeTypeError };

pub struct Decoder {
    metadata: Metadata
}

#[derive(Clone,Debug,thiserror::Error)]
pub enum DecodeError {
    #[error("Failed to parse the provided vector of extrinsics: {0}")]
    UnexpectedExtrinsicsShape(#[from] ExtrinsicBytesError),
    #[error("Failed to decode type: {0}")]
    DecodeTypeError(#[from] DecodeTypeError),
    #[error("Failed to decode: expected more data")]
    EarlyEof(&'static str),
    #[error("Failed to decode unsupported extrinsic version '{0}'")]
    CannotDecodeExtrinsicVersion(u8),
    #[error("Cannot find call corresponding to extrinsic with pallet index {0} and call index {1}")]
    CannotFindCall(u8, u8),
    #[error("Failed to decode extrinsic: {0}")]
    CannotFindType(#[from] MetadataError),
}

impl Decoder {
    /// Create a new decoder using the provided metadata.
    pub fn with_metadata<M: Into<Metadata>>(metadata: M) -> Decoder {
        Decoder {
            metadata: metadata.into()
        }
    }

    /// Decode a SCALE encoded vector of extrinsics against the metadata provided
    pub fn decode_extrinsics(&self, data: &[u8]) -> Result<Vec<GenericExtrinsic>, DecodeError> {
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
    pub fn decode_extrinsic(&self, mut data: &[u8]) -> Result<GenericExtrinsic, DecodeError> {
        if data.len() == 0 {
            return Err(DecodeError::EarlyEof("extrinsic length should be > 0"));
        }

        let info = interpret_extrinsic_version(data[0]);

        // We only know how to decode V4 extrinsics at the moment
        if info.version != 4 {
            return Err(DecodeError::CannotDecodeExtrinsicVersion(info.version));
        }

        // If the extrinsic is signed, decode the signature first. Remember that V4
        // extrinsics are laid out roughly as follows:
        //
        // first byte: abbbbbbb (a = 0 for unsigned, 1 for signed, b = version)
        // signature, which is made up of (in order):
        // - sp_runtime::MultiAddress enum (sender)
        // - sp_runtime::MultiSignature enum
        // - sp_runtime::generic::Era enum
        // - compact encoded u32 (nonce; prior transaction count)
        // - compact encoded u128 (tip paid to block producer/treasury)
        // call, which is made up roughly of:
        // - enum pallet index (for pallets variant)
        // - call index (for inner variant)
        // - call args (types can be pulled from metadata for each arg we expect)
        let mut signature = None;
        if info.is_signed {

        }

        if data.len() < 2 {
            return Err(DecodeError::EarlyEof("expected at least 2 more bytes for the pallet/call index"));
        }

        // Work out which call the extrinsic data represents and get type info for it:
        let pallet_index = data[0];
        let call_index = data[1];
        data = &data[2..];
        let (pallet_name, call) = match self.metadata.call_by_variant_index(pallet_index, call_index) {
            Some(call) => call,
            None => return Err(DecodeError::CannotFindCall(pallet_index, call_index))
        };

        // Decode each of the argument values in the extrinsic:
        let mut arguments = vec![];
        for arg in call.args() {
            let ty = self.metadata.resolve_type(arg)?;
            let val = match decode_type(data, &ty) {
                Ok((val, rest)) => {
                    data = rest;
                    val
                },
                Err((err, _rest)) => {
                    return Err(err.into())
                }
            };
            arguments.push(val);
        }

        // Return a composite type representing the extrinsic arguments:
        Ok(GenericExtrinsic {
            pallet: pallet_name.to_owned(),
            call: call.name().to_owned(),
            signature,
            arguments,
        })
    }
}

pub struct GenericExtrinsic {
    /// The name of the pallet that the extrinsic called into
    pub pallet: String,
    /// The name of the call made
    pub call: String,
    /// The signature (if any) associated with the extrinsic
    pub signature: Option<SubstrateValue>,
    /// The arguments to pass to the call
    pub arguments: Vec<SubstrateValue>
}

struct ExtrinsicVersionInfo {
    /// Which version is this extrinsic?
    version: u8,
    /// Does this extrinsic have a signature?
    is_signed: bool
}

fn interpret_extrinsic_version(byte: u8) -> ExtrinsicVersionInfo {
    let is_signed = byte & 0b1000_0000 != 0;
    let version = byte & 0b0111_1111;
    ExtrinsicVersionInfo { version, is_signed }
}