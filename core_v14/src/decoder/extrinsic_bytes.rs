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
pub struct ExtrinsicBytes<'a> {
	len: usize,
	data: &'a [u8],
}

impl<'a> ExtrinsicBytes<'a> {
	/// Treat the bytes provided as a set of Extrinsics, which conceptually has the shape
	/// `Vec<(Compact<u32>, Extrinsic)>`. Return an error if the bytes are obviously not
	/// such a shape.
	pub fn new(bytes: &'a [u8]) -> Result<ExtrinsicBytes<'a>, ExtrinsicBytesError> {
		let (vec_len, vec_len_bytes) = match decode_compact_u32(bytes) {
			Some(res) => res,
			None => return Err(ExtrinsicBytesError { index: 0 }),
		};

		Ok(ExtrinsicBytes { len: vec_len, data: &bytes[vec_len_bytes..] })
	}
}

impl<'a> ExtrinsicBytes<'a> {
	/// How many extrinsics are there?
	pub fn len(&self) -> usize {
		self.len
	}

	/// Iterate over a SCALE encoded vector of extrinsics and return the bytes associated
	/// with each one (not including the length prefix), or an error containing the position
	/// at which decoding failed.
	pub fn iter(&self) -> ExtrinsicBytesIter<'a> {
		ExtrinsicBytesIter { data: &self.data, cursor: 0 }
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
	type Item = Result<SingleExtrinsic<'a>, ExtrinsicBytesError>;
	fn next(&mut self) -> Option<Self::Item> {
		if self.data.is_empty() {
			return None;
		}

		let (vec_len, vec_len_bytes) = match decode_compact_u32(&self.data) {
			Some(res) => res,
			None => return Some(Err(ExtrinsicBytesError { index: self.cursor })),
		};
		log::trace!("Length {}, Prefix: {}", vec_len, vec_len_bytes);

		let res = &self.data[(self.cursor + vec_len_bytes)..(self.cursor + vec_len + vec_len_bytes)];
		self.cursor += vec_len + vec_len_bytes;

		Some(Ok(SingleExtrinsic { data: res, remaining: self.data.len() - self.cursor }))
	}
}

pub struct SingleExtrinsic<'a> {
	data: &'a [u8],
	remaining: usize,
}

impl<'a> SingleExtrinsic<'a> {
	/// The bytes representing a single extrinsic
	pub fn bytes(&'a self) -> &'a [u8] {
		&self.data
	}
	/// How many bytes remain to be decoded after this extrinsic?
	pub fn remaining(&self) -> usize {
		self.remaining
	}
}

/// An error containing the index into the byte slice at which decoding failed.
#[derive(Debug, Clone, Copy, thiserror::Error)]
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
