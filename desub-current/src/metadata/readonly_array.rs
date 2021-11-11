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
