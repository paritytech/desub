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

/// A structure representing a set of extrinsics in terms of their raw SCALE encoded bytes.
#[derive(Clone, Copy)]
pub struct AllExtrinsicBytes<'a> {
	len: usize,
	data: &'a [u8],
}

impl<'a> AllExtrinsicBytes<'a> {
	/// Treat the bytes provided as a set of Extrinsics, which conceptually has the shape
	/// `Vec<(Compact<u32>, Extrinsic)>`. Return an error if the bytes are obviously not
	/// such a shape.
	pub fn new(bytes: &'a [u8]) -> Result<AllExtrinsicBytes<'a>, ExtrinsicBytesError> {
		let (vec_len, vec_len_bytes) = match decode_compact_u32(bytes) {
			Some(res) => res,
			None => return Err(ExtrinsicBytesError { index: 0 }),
		};

		Ok(AllExtrinsicBytes { len: vec_len, data: &bytes[vec_len_bytes..] })
	}
}

impl<'a> AllExtrinsicBytes<'a> {
	/// How many extrinsics are there? Note that this is simply the reported number of extrinsics,
	/// and if the extrinsic bytes are malformed, it may not equal the actual number of extrinsics
	/// that we are able to iterate over.
	pub fn len(&self) -> usize {
		self.len
	}

	/// Iterate over a SCALE encoded vector of extrinsics and return the bytes associated
	/// with each one (not including the length prefix), or an error containing the position
	/// at which decoding failed.
	pub fn iter(&self) -> ExtrinsicBytesIter<'a> {
		ExtrinsicBytesIter { data: self.data, cursor: 0 }
	}
}

/// An iterator that returns the set of bytes representing each extrinsic found.
/// On each iteration, we return either the extrinsic bytes, or an error containing
/// the position at which decoding failed.
pub struct ExtrinsicBytesIter<'a> {
	data: &'a [u8],
	cursor: usize,
}

impl<'a> Iterator for ExtrinsicBytesIter<'a> {
	type Item = Result<ExtrinsicBytes<'a>, ExtrinsicBytesError>;
	fn next(&mut self) -> Option<Self::Item> {
		if self.cursor >= self.data.len() {
			return None;
		}

		let (vec_len, vec_len_bytes) = match decode_compact_u32(&self.data[self.cursor..]) {
			Some(res) => res,
			None => {
				// Ensure that if we try iterating again we get back `None`:
				self.cursor = self.data.len();
				return Some(Err(ExtrinsicBytesError { index: self.cursor }));
			}
		};
		log::trace!("Length {}, Prefix: {}", vec_len, vec_len_bytes);

		let start = self.cursor + vec_len_bytes;
		let end = self.cursor + vec_len_bytes + vec_len;

		// We are trusting the lengths reported. Avoid a panic by ensuring that if there
		// aren't as many bytes as we expect, we bail with an error.
		if end > self.data.len() {
			// Ensure that if we try iterating again we get back `None`:
			self.cursor = self.data.len();
			return Some(Err(ExtrinsicBytesError { index: self.data.len() }));
		}

		let res = &self.data[start..end];
		self.cursor += vec_len + vec_len_bytes;

		Some(Ok(ExtrinsicBytes { data: res, remaining: self.data.len() - self.cursor }))
	}
}

pub struct ExtrinsicBytes<'a> {
	data: &'a [u8],
	remaining: usize,
}

impl<'a> ExtrinsicBytes<'a> {
	/// The bytes representing a single extrinsic
	pub fn bytes(&self) -> &'a [u8] {
		self.data
	}
	/// How many bytes remain to be decoded after this extrinsic?
	pub fn remaining(&self) -> usize {
		self.remaining
	}
}

/// An error containing the index into the byte slice at which decoding failed.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("Expected a compact encoded u32 at byte index {index}, but did not find one")]
pub struct ExtrinsicBytesError {
	pub index: usize,
}

/// Given a SCALE encoded `Compact<u32>` (which prefixes a SCALE encoded vector, for instance),
/// return a tuple of the length of the vector, and the number of input bytes used to represent
/// this length.
fn decode_compact_u32(mut data: &[u8]) -> Option<(usize, usize)> {
	use codec::{Compact, CompactLen, Decode};
	use std::convert::TryFrom;

	let length = u32::from(Compact::<u32>::decode(&mut data).ok()?);
	let prefix = Compact::<u32>::compact_len(&length);
	let length = usize::try_from(length).ok()?;
	Some((length, prefix))
}

#[cfg(test)]
mod test {

	use super::*;
	use codec::{Compact, Encode};

	fn iter_result_to_bytes<'a, E>(res: Option<Result<ExtrinsicBytes<'a>, E>>) -> Option<Result<&'a [u8], E>> {
		res.map(|r| r.map(|e| e.bytes()))
	}

	#[test]
	fn no_malformed_bytes_iterated_properly() {
		let mut bytes: Vec<u8> = vec![];

		// 2 entries in block (correct):
		bytes.extend_from_slice(&Compact(2u32).encode());

		// First entry; 4 bytes long (correct):
		bytes.extend_from_slice(&Compact(4u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4]);

		// Second entry; 3 bytes long (also correct):
		bytes.extend_from_slice(&Compact(3u32).encode());
		bytes.extend_from_slice(&[1, 2, 3]);

		let exts = AllExtrinsicBytes::new(&bytes).unwrap();
		assert_eq!(exts.len(), 2);

		let mut exts = exts.iter();
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3, 4][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), None);
	}

	#[test]
	fn malformed_extrinsics_length() {
		let mut bytes: Vec<u8> = vec![];

		// 3 entries in block (wrong):
		bytes.extend_from_slice(&Compact(3u32).encode());

		// First entry; 4 bytes long (correct):
		bytes.extend_from_slice(&Compact(4u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4]);

		// Second entry; 3 bytes long (also correct):
		bytes.extend_from_slice(&Compact(3u32).encode());
		bytes.extend_from_slice(&[1, 2, 3]);

		// No third entry (whoops; malformed).

		let exts = AllExtrinsicBytes::new(&bytes).unwrap();
		assert_eq!(exts.len(), 3); // Wrong length reported; we'll see when we iterate..

		let mut exts = exts.iter();
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3, 4][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), None);
	}

	#[test]
	fn malformed_extrinsic_length() {
		let mut bytes: Vec<u8> = vec![];

		// 3 entries in block (correct):
		bytes.extend_from_slice(&Compact(2u32).encode());

		// First entry; 4 bytes long (correct):
		bytes.extend_from_slice(&Compact(4u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4]);

		// Second entry; 3 bytes long (wrong):
		bytes.extend_from_slice(&Compact(3u32).encode());
		bytes.extend_from_slice(&[1, 2]);

		let exts = AllExtrinsicBytes::new(&bytes).unwrap();
		assert_eq!(exts.len(), 2);

		let mut exts = exts.iter();
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3, 4][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Err(ExtrinsicBytesError { index: 8 })));
		assert_eq!(iter_result_to_bytes(exts.next()), None);
	}

	#[test]
	fn malformed_two_lengths() {
		let mut bytes: Vec<u8> = vec![];

		// 3 entries in block (wrong):
		bytes.extend_from_slice(&Compact(3u32).encode());

		// First entry; 4 bytes long (correct):
		bytes.extend_from_slice(&Compact(4u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4]);

		// Second entry; 3 bytes long (wrong):
		bytes.extend_from_slice(&Compact(3u32).encode());
		bytes.extend_from_slice(&[1, 2]);

		// No third entry (whoops; malformed).

		let exts = AllExtrinsicBytes::new(&bytes).unwrap();
		assert_eq!(exts.len(), 3); // Wrong length reported

		let mut exts = exts.iter();
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3, 4][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Err(ExtrinsicBytesError { index: 8 })));
		assert_eq!(iter_result_to_bytes(exts.next()), None);
	}
}
