use crate::metadata::Metadata;
use crate::generic_extrinsic::GenericExtrinsic;
use super::utils::{ scaled_extrinsic_bytes, ExtrinsicBytesError };

pub struct Decoder {
    metadata: Metadata
}

#[derive(Clone,Debug,thiserror::Error)]
pub enum DecodeError {
    #[error("Failed to parse the provided vector of extrinsics: {0}")]
    UnexpectedExtrinsicsShape(#[from] ExtrinsicBytesError)
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
        let extrinsic_bytes = scaled_extrinsic_bytes(data)?;
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
    pub fn decode_extrinsic(&self, data: &[u8]) -> Result<GenericExtrinsic, DecodeError> {
        todo!()
    }

}
