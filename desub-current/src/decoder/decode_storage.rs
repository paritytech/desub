use super::Value;
use crate::metadata::{Metadata, StorageLocation};
use crate::{Type, TypeId};
use frame_metadata::v14::StorageEntryType;
use serde::Serialize;
use sp_core::twox_128;
use std::borrow::Cow;
use std::collections::HashMap;

/// This struct is capable of decoding SCALE encoded storage
pub struct StorageDecoder {
	/// We can find the prefix for a given storage entry if we
	/// know the twox_128 hash of it:
	entries_by_hashed_prefix: HashMap<[u8; 16], StorageEntries>,
}

struct StorageEntries {
	/// The index of the storage entry as stored in the metadata used to
	/// generate this.
	index: usize,
	/// Within this pallet/prefix, we can find the sub-index of each storage entry
	/// if we know the twox_128 hash of it:
	entry_by_hashed_name: HashMap<[u8; 16], usize>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum StorageDecodeError {
	#[error("Not enough bytes in the input data to decode the storage prefix and name; got {0} bytes but expected 32")]
	NotEnoughBytesForPrefixAndName(usize),
	#[error("Expecting the same number of keys and hashers, but got {num_keys} keys and {num_hashers} hashers")]
	KeysAndHashersDontLineUp { num_keys: usize, num_hashers: usize },
	#[error("Type with id {0} expected in the metadata but not found")]
	TypeNotFound(u32),
	#[error("Couldn't decode the value associated with the hasher {0:?}: {1}")]
	CouldNotDecodeHasherValue(frame_metadata::v14::StorageHasher, super::DecodeValueError),
	#[error("Couldn't find a storage entry corresponding to the prefix hash provided in the data")]
	PrefixNotFound,
	#[error("Couldn't find a storage entry corresponding to the name hash provided in the data")]
	NameNotFound,
}

impl StorageDecoder {
	/// Call [`super::decode_storage`] to construct a [`StorageDecoder`].
	pub(super) fn generate_from_metadata(metadata: &Metadata) -> StorageDecoder {
		let entries_by_hashed_prefix = metadata
			.storage_entries()
			.enumerate()
			.map(|(index, entries)| {
				let prefix_hash = twox_128(entries.prefix().as_bytes());
				let entry_by_hashed_name = entries
					.entries()
					.enumerate()
					.map(|(entry_index, entry)| {
						let name_hash = twox_128(entry.name.as_bytes());
						(name_hash, entry_index)
					})
					.collect();
				(prefix_hash, StorageEntries { index, entry_by_hashed_name })
			})
			.collect();

		StorageDecoder { entries_by_hashed_prefix }
	}

	/// Decode the SCALE encoded bytes representing a storage entry lookup. These conceptually take the
	/// form `twox_128(prefix) + twox_128(name) + rest`, where `rest` are hashed
	pub fn decode_key<'m, 'b>(
		&self,
		metadata: &'m Metadata,
		bytes: &mut &'b [u8],
	) -> Result<StorageEntry<'m, 'b>, StorageDecodeError> {
		let location = self.decode_prefix_and_name_to_location(bytes)?;
		let storage_entry = metadata.storage_entry(location);

		let prefix_str = storage_entry.prefix;
		let name_str = &*storage_entry.metadata.name;

		match &storage_entry.metadata.ty {
			StorageEntryType::Plain(ty) => {
				// No more work to do here; our storage entry is a plain prefix+name entry,
				// so return the details of it:
				Ok(StorageEntry {
					prefix: prefix_str.into(),
					name: name_str.into(),
					ty: *ty,
					details: StorageEntryDetails::Plain,
				})
			}
			StorageEntryType::Map { hashers, key, value } => {
				// We'll consume some more data based on the hashers.
				// First, get the type information that we need ready.
				let key =
					metadata.types().resolve(key.id()).ok_or_else(|| StorageDecodeError::TypeNotFound(key.id()))?;
				let keys = storage_map_key_to_type_id_vec(metadata, key)?;
				if keys.len() != hashers.len() {
					return Err(StorageDecodeError::KeysAndHashersDontLineUp {
						num_keys: keys.len(),
						num_hashers: hashers.len(),
					});
				}

				// Work through the hashers and type info we have to generate out output
				// data, and consume bytes from the input cursor as we go.
				let mut storage_keys = vec![];
				for (hasher, ty) in hashers.iter().zip(keys) {
					pub use frame_metadata::v14::StorageHasher as FrameStorageHasher;

					// How many bytes will the hashed bit consume?
					let initial_hash_bytes = match hasher {
						FrameStorageHasher::Blake2_128
						| FrameStorageHasher::Twox128
						| FrameStorageHasher::Blake2_128Concat => 16,
						FrameStorageHasher::Blake2_256 | FrameStorageHasher::Twox256 => 32,
						FrameStorageHasher::Twox64Concat => 8,
						FrameStorageHasher::Identity => 0,
					};

					// Is the SCALE encoded Value next up after the hash bit?
					let is_value_next = match hasher {
						FrameStorageHasher::Blake2_128Concat
						| FrameStorageHasher::Twox64Concat
						| FrameStorageHasher::Identity => true,
						_other => false,
					};

					// Decode the value if so, and return the total bytes consumed so far and the resulting hasher.
					let (hasher, bytes_consumed) = if is_value_next {
						// Don't consume our `bytes` here; create a new cursor to consume and count the length
						// of the value in bytes, and then we can return this and tweak the input bytes cursor
						// in one place below.
						let value_bytes = &mut &bytes[initial_hash_bytes..];
						let start_len = value_bytes.len();
						let value = super::decode_value(metadata, ty, value_bytes)
							.map_err(|e| StorageDecodeError::CouldNotDecodeHasherValue(hasher.clone(), e))?;
						let value_len = start_len - value_bytes.len();
						(StorageHasher::expect_from_with_value(hasher, value), initial_hash_bytes + value_len)
					} else {
						(StorageHasher::expect_from(hasher), initial_hash_bytes)
					};

					// Move the byte cursor forwards and push an entry to our storage keys:
					let hash_bytes = &bytes[..bytes_consumed];
					*bytes = &bytes[bytes_consumed..];
					storage_keys.push(StorageKey { bytes: Cow::Borrowed(hash_bytes), hasher, ty: Cow::Borrowed(ty) });
				}

				Ok(StorageEntry {
					prefix: prefix_str.into(),
					name: name_str.into(),
					ty: *value,
					details: StorageEntryDetails::Map(storage_keys),
				})
			}
		}
	}

	// Reverse the prefix+name hashing (which takes the form of `twox_128(prefix) + twox_128(name)`)
	// into a specific storage location, which we can lookup in the Metadata to decode the remaining
	// bytes.
	fn decode_prefix_and_name_to_location(&self, data: &mut &[u8]) -> Result<StorageLocation, StorageDecodeError> {
		if data.len() < 32 {
			return Err(StorageDecodeError::NotEnoughBytesForPrefixAndName(data.len()));
		}
		let prefix_hash = &data[..16];
		let name_hash = &data[16..32];

		let entries = self.entries_by_hashed_prefix.get(prefix_hash).ok_or(StorageDecodeError::PrefixNotFound)?;

		let entry_index = entries.entry_by_hashed_name.get(name_hash).ok_or(StorageDecodeError::NameNotFound)?;

		// Successfully consumed the prefix and name bytes, so move our cursor.
		// In the case of errors, we leave the data "unconsumed".
		*data = &data[32..];

		Ok(StorageLocation { prefix_index: entries.index, entry_index: *entry_index })
	}
}

// Metadata info for maps/doublemaps contains a vec of hashers for each key type,
// and a Type representing the key(s). We expect the number of keys and hashers to
// line up, so let's resolve the keys into something easier to work with.
//
// See https://github.com/paritytech/subxt/blob/793c945fbd2de022f523c39a84ee02609ba423a9/codegen/src/api/storage.rs#L105
// for another example of this being handled in code.
fn storage_map_key_to_type_id_vec<'a>(
	metadata: &'a Metadata,
	key: &'a Type,
) -> Result<Vec<&'a Type>, StorageDecodeError> {
	match key.type_def() {
		// Multiple keys:
		scale_info::TypeDef::Tuple(vals) => {
			let types: Result<Vec<_>, _> = vals
				.fields()
				.iter()
				.map(|f| {
					let id = f.id();
					metadata.types().resolve(id).ok_or(StorageDecodeError::TypeNotFound(id))
				})
				.collect();
			types
		}
		// Single key:
		_ => Ok(vec![key]),
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageEntry<'m, 'b> {
	pub prefix: Cow<'m, str>,
	pub name: Cow<'m, str>,
	pub ty: TypeId,
	pub details: StorageEntryDetails<'m, 'b>,
}

impl<'m, 'b> StorageEntry<'m, 'b> {
	pub fn into_owned(self) -> StorageEntry<'static, 'static> {
		StorageEntry {
			prefix: Cow::Owned(self.prefix.into_owned()),
			name: Cow::Owned(self.name.into_owned()),
			ty: self.ty,
			details: self.details.into_owned(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StorageEntryDetails<'m, 'b> {
	Plain,
	Map(Vec<StorageKey<'m, 'b>>),
}

impl<'m, 'b> StorageEntryDetails<'m, 'b> {
	pub fn into_owned(self) -> StorageEntryDetails<'static, 'static> {
		match self {
			Self::Plain => StorageEntryDetails::Plain,
			Self::Map(keys) => StorageEntryDetails::Map(keys.into_iter().map(|k| k.into_owned()).collect()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageKey<'m, 'b> {
	pub bytes: Cow<'b, [u8]>,
	pub ty: Cow<'m, Type>,
	pub hasher: StorageHasher,
}

impl<'m, 'b> StorageKey<'m, 'b> {
	pub fn into_owned(self) -> StorageKey<'static, 'static> {
		StorageKey {
			bytes: Cow::Owned(self.bytes.into_owned()),
			ty: Cow::Owned(self.ty.into_owned()),
			hasher: self.hasher,
		}
	}
}

/// This is almost identical to [`frame_metadata::v14::StorageHasher`],
/// except it also carries the decoded [`Value`] for those hasher types
/// it can be decoded from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StorageHasher {
	Blake2_128,
	Blake2_256,
	Blake2_128Concat(Value),
	Twox128,
	Twox256,
	Twox64Concat(Value),
	Identity(Value),
}

impl StorageHasher {
	fn expect_from(hasher: &frame_metadata::v14::StorageHasher) -> Self {
		match hasher {
			frame_metadata::v14::StorageHasher::Blake2_128 => StorageHasher::Blake2_128,
			frame_metadata::v14::StorageHasher::Blake2_256 => StorageHasher::Blake2_256,
			frame_metadata::v14::StorageHasher::Twox128 => StorageHasher::Twox128,
			frame_metadata::v14::StorageHasher::Twox256 => StorageHasher::Twox256,
			frame_metadata::v14::StorageHasher::Identity
			| frame_metadata::v14::StorageHasher::Blake2_128Concat
			| frame_metadata::v14::StorageHasher::Twox64Concat => {
				panic!("Cannot convert {:?} into a StorageHasher; needs Value", hasher)
			}
		}
	}
	fn expect_from_with_value(hasher: &frame_metadata::v14::StorageHasher, value: Value) -> Self {
		match hasher {
			frame_metadata::v14::StorageHasher::Blake2_128
			| frame_metadata::v14::StorageHasher::Blake2_256
			| frame_metadata::v14::StorageHasher::Twox128
			| frame_metadata::v14::StorageHasher::Twox256 => {
				panic!("Cannot convert {:?} into a StorageHasher; no Value expected", hasher)
			}
			frame_metadata::v14::StorageHasher::Identity => StorageHasher::Identity(value),
			frame_metadata::v14::StorageHasher::Blake2_128Concat => StorageHasher::Blake2_128Concat(value),
			frame_metadata::v14::StorageHasher::Twox64Concat => StorageHasher::Twox64Concat(value),
		}
	}
}
