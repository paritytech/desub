use std::collections::HashMap;
use sp_core::twox_128;
use crate::metadata::{ Metadata, StorageLocation };
use crate::TypeId;
use std::borrow::Cow;

// Re-export types referenced within publicly exported
// structs in this module for convenience.
pub use frame_metadata::v14::{ StorageHasher };
pub type StorageEntryType = frame_metadata::v14::StorageEntryType<scale_info::form::PortableForm>;

/// This struct is capable of decoding SCALE encoded storage
pub struct StorageDecoder {
    /// We can find the prefix for a given storage entry if we
    /// know the twox_128 hash of it:
    entries_by_hashed_prefix: HashMap<[u8; 16], StorageEntries>
}

pub struct StorageEntries {
    /// The index of the storage entry as stored in the metadata used to
    /// generate this.
    index: usize,
    /// Within this pallet/prefix, we can find the sub-index of each storage entry
    /// if we know the twox_128 hash of it:
    entry_by_hashed_name: HashMap<[u8; 16], usize>
}

pub enum StorageDecodeError {

}

impl StorageDecoder {
    pub (super) fn generate_from_metadata(metadata: &Metadata) -> StorageDecoder {
        let mut entries_by_hashed_prefix = metadata
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
    pub fn decode_key<'a>(&self, metadata: &Metadata, key: &mut &[u8]) -> Result<StorageEntry<'a>, StorageDecodeError> {
        let location = self.decode_prefix_and_name_to_location(key)?;
        let storage_entry = metadata.storage_entry(location);
        todo!()
    }

    // Reverse the prefix+name hashing (which takes the form of `twox_128(prefix) + twox_128(name)`)
    // into a specific storage location, which we can lookup in the Metadata to decode the remaining
    // bytes.
    fn decode_prefix_and_name_to_location(&self, data: &mut &[u8]) -> Result<StorageLocation, StorageDecodeError> {
        todo!()
    }
}

pub struct StorageEntry<'a> {
    pub prefix: Cow<'a, str>,
	pub name: Cow<'a, str>,
    pub entry_type: StorageKeyData<'a>
}

pub enum StorageKeyData<'a> {
    Plain(TypeId),
    Map(Vec<StorageKey<'a>>)
}

pub struct StorageKey<'a> {
    pub bytes: Cow<'a, [u8]>,
    pub ty: TypeId,
    pub hasher: StorageHasher,
}