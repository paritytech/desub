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
		ExtrinsicBytesIter { remaining_len: self.len, data: self.data, cursor: 0 }
	}
}

/// An iterator that returns the set of bytes representing each extrinsic found.
/// On each iteration, we return either the extrinsic bytes, or an error containing
/// the position at which decoding failed.
pub struct ExtrinsicBytesIter<'a> {
	/// The number of extrinsics we expect to be able to decode from the bytes.
	/// this is decremented on each iteration.
	remaining_len: usize,
	data: &'a [u8],
	cursor: usize,
}

impl<'a> ExtrinsicBytesIter<'a> {
	/// Return the bytes remaining. If an iteration resulted in an error,
	/// we'll return the bytes that we failed to decode, too.
	pub fn remaining_bytes(&self) -> &'a [u8] {
		&self.data[self.cursor..]
	}
}

impl<'a> Iterator for ExtrinsicBytesIter<'a> {
	type Item = Result<ExtrinsicBytes<'a>, ExtrinsicBytesError>;
	fn next(&mut self) -> Option<Self::Item> {
		// Stop when we hit the number of item's we're supposed to have,
		// or have exhausted the data.
		if self.remaining_len == 0 || self.cursor >= self.data.len() {
			return None;
		}
		self.remaining_len -= 1;

		let (vec_len, vec_len_bytes) = match decode_compact_u32(&self.data[self.cursor..]) {
			Some(res) => res,
			None => {
				// Ensure that if we try iterating again we get back `None`:
				self.remaining_len = 0;
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
			self.remaining_len = 0;
			return Some(Err(ExtrinsicBytesError { index: self.data.len() }));
		}

		let res = &self.data[start..end];
		self.cursor += vec_len + vec_len_bytes;

		Some(Ok(ExtrinsicBytes { data: res }))
	}
}

pub struct ExtrinsicBytes<'a> {
	data: &'a [u8],
}

impl<'a> ExtrinsicBytes<'a> {
	/// The bytes representing a single extrinsic
	pub fn bytes(&self) -> &'a [u8] {
		self.data
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

	let length = u32::from(Compact::<u32>::decode(&mut data).ok()?);
	let prefix = Compact::<u32>::compact_len(&length);
	let length = usize::try_from(length).ok()?;
	Some((length, prefix))
}

#[cfg(test)]
mod test {

	use super::*;
	use codec::{Compact, Encode};

	fn iter_result_to_bytes<E>(res: Option<Result<ExtrinsicBytes<'_>, E>>) -> Option<Result<&[u8], E>> {
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

	#[test]
	fn wont_iterate_past_advertised_length() {
		let mut bytes: Vec<u8> = vec![];

		// 2 entries in block (but we have enough bytes for 3):
		bytes.extend_from_slice(&Compact(2u32).encode());

		// 3 correct entries:
		bytes.extend_from_slice(&Compact(4u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4]);
		bytes.extend_from_slice(&Compact(3u32).encode());
		bytes.extend_from_slice(&[1, 2, 3]);
		bytes.extend_from_slice(&Compact(5u32).encode());
		bytes.extend_from_slice(&[1, 2, 3, 4, 5]);

		let exts = AllExtrinsicBytes::new(&bytes).unwrap();
		assert_eq!(exts.len(), 2);

		let mut exts = exts.iter();
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3, 4][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), Some(Ok(&[1, 2, 3][..])));
		assert_eq!(iter_result_to_bytes(exts.next()), None);

		// The bytes we should have left (the third entry):
		let mut remaining_bytes: Vec<u8> = vec![];
		remaining_bytes.extend_from_slice(&Compact(5u32).encode());
		remaining_bytes.extend_from_slice(&[1, 2, 3, 4, 5]);

		assert_eq!(exts.remaining_bytes(), &remaining_bytes);
	}
}
