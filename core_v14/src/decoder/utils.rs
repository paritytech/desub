/// Iterate over a SCALE encoded vector of extrinsics and return the bytes associated with each one.
/// Conceptually, the vector of extrinsics is encoded in the form:
///
/// `Vec<(Compact<u32>, Extrinsic)>`
///
/// Where Each extrinsic is prefixed with a compact encoding of its length in bytes. This is because
/// extrinsics are themselves just treated as opaque vectors of btyes when they are encoded.
///
/// On each iteration, we return the extrinsic bytes, or a `usize` denoting the position at
pub fn scaled_extrinsic_bytes(data: &[u8]) -> Result<ExtrinsicBytes, ExtrinsicBytesError> {
    let (vec_len, vec_len_bytes) = match decode_compact_u32(data) {
        Some(res) => res,
        None => return Err(ExtrinsicBytesError { index: 0 })
    };

    Ok(ExtrinsicBytes {
        len: vec_len,
        data: &data[vec_len_bytes..]
    })
}

/// A structure representing a set of extrinsics in terms of their raw SCALE encoded bytes.
#[derive(Clone, Copy)]
pub struct ExtrinsicBytes<'a> {
    len: usize,
    data: &'a [u8]
}

impl <'a> ExtrinsicBytes<'a> {
    /// How many extrinsics are there?
    pub fn len(&self) -> usize {
        self.len
    }

    /// Iterate over the bytes, returning each extrinsic found in the form of its bytes,
    /// or an error if we cannot decode the bytes as expected.
    pub fn iter(&self) -> ExtrinsicBytesIter<'a> {
        ExtrinsicBytesIter { data: &self.data, cursor: 0 }
    }
}

/// An iterator that returns the set of bytes representing each extrinsic found.
pub struct ExtrinsicBytesIter<'a> {
    data: &'a [u8],
    cursor: usize
}

impl <'a> Iterator for ExtrinsicBytesIter<'a> {
    type Item = Result<&'a [u8], ExtrinsicBytesError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None
        }

        let (vec_len, vec_len_bytes) = match decode_compact_u32(&self.data) {
            Some(res) => res,
            None => return Some(Err(ExtrinsicBytesError { index: self.cursor }))
        };
		log::trace!("Length {}, Prefix: {}", vec_len, vec_len_bytes);

        let res = &self.data[(self.cursor + vec_len_bytes) .. (self.cursor + vec_len + vec_len_bytes)];
        self.cursor += vec_len + vec_len_bytes;
        Some(Ok(res))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Expected a compact encoded u32 at byte index {index}, but did not find one")]
pub struct ExtrinsicBytesError {
    pub index: usize
}

/// Given a SCALE encoded `Compact<u32>` (which prefixes a SCALE encoded vector, for instance),
/// return a tuple of the length of the vector, and the number of input bytes used to represent
/// this length.
fn decode_compact_u32(mut data: &[u8]) -> Option<(usize, usize)> {
    use codec::{Compact, CompactLen, Decode};
    use std::convert::TryFrom;

    // alternative to `DecodeLength` trait, to avoid casting from a trait
    let length = u32::from(Compact::<u32>::decode(&mut data).ok()?);
    let prefix = Compact::<u32>::compact_len(&length);
    let length = usize::try_from(length).ok()?;
    Some((length, prefix))
}