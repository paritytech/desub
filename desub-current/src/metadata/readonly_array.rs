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

use std::ops::Deref;

/// A wrapper that takes a `Vec<T>` and hands back a
/// type from which you can only access a `&[T]`, to guarantee
/// that it cannot be modified.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadonlyArray<T>(Box<[T]>);

impl<T> ReadonlyArray<T> {
	pub fn from_vec(vec: Vec<T>) -> ReadonlyArray<T> {
		ReadonlyArray(vec.into_boxed_slice())
	}
}

impl<T> From<Vec<T>> for ReadonlyArray<T> {
	fn from(v: Vec<T>) -> Self {
		ReadonlyArray::from_vec(v)
	}
}

impl<T> Deref for ReadonlyArray<T> {
	type Target = [T];
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
