use std::any::Any;
use std::collections::HashMap;
use sp_core::twox_128;
use crate::metadata::{ Metadata, StorageLocation };
use crate::{ Type, TypeId };
use std::borrow::Cow;
use frame_metadata::v14::StorageEntryType;

// Re-export types referenced within publicly exported
// structs in this module for convenience.
pub use frame_metadata::v14::{ StorageHasher };

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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum StorageDecodeError {
    #[error("Not enough bytes in the input data to decode the storage prefix and name; got {0} bytes but expected 32")]
    NotEnoughBytesForPrefixAndName(usize),
    #[error("Expecting the same number of keys and hashers, but got {num_keys} keys and {num_hashers} hashers")]
    KeysAndHashersDontLineUp { num_keys: usize, num_hashers: usize },
    #[error("Type with id {0} expected in the metadata but not found")]
    TypeNotFound(u32),
    #[error("Couldn't find a storage entry corresponding to the prefix hash provided in the data")]
    PrefixNotFound,
    #[error("Couldn't find a storage entry corresponding to the name hash provided in the data")]
    NameNotFound,
}

impl StorageDecoder {
    /// Call [`super::decode_storage`] to construct a [`StorageDecoder`].
    pub (super) fn generate_from_metadata(metadata: &Metadata) -> StorageDecoder {
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
    pub fn decode_key<'a>(&self, metadata: &'a Metadata, key: &mut &[u8]) -> Result<StorageEntry<'a>, StorageDecodeError> {
        let location = self.decode_prefix_and_name_to_location(key)?;
        let storage_entry = metadata.storage_entry(location);

        let prefix_str = storage_entry.prefix;
        let name_str  = &*storage_entry.metadata.name;

        match &storage_entry.metadata.ty {
            StorageEntryType::Plain(ty) => {
                // No more work to do here; our storage entry is a plain prefix+name entry,
                // so return the details of it:
                Ok(StorageEntry {
                    prefix: prefix_str.into(),
                    name: name_str.into(),
                    details: StorageEntryDetails::Plain(*ty)
                })
            },
            StorageEntryType::Map{ hashers, key, value } => {
                // We'll consume some more data based on the hashers.
                // First, get the type information that we need ready.
                let key = metadata.types().resolve(key.id())
                    .ok_or(StorageDecodeError::TypeNotFound(key.id()))?;
                let keys = storage_map_key_to_type_id_vec(metadata, key)?;
                if keys.len() != hashers.len() {
                    return Err(StorageDecodeError::KeysAndHashersDontLineUp {
                        num_keys: keys.len(),
                        num_hashers: hashers.len()
                    })
                }

                // Zip hasher and type info, and pull out as many bytes as needed
                // from the input as we go, based on the hasher.
                let mut storage_keys = vec![];
                for (hasher, ty) in hashers.iter().zip(keys) {
                    match hasher {
                        StorageHasher::Blake2_128 => todo!(),
                        StorageHasher::Blake2_256 => todo!(),
                        StorageHasher::Blake2_128Concat => todo!(),
                        StorageHasher::Twox128 => todo!(),
                        StorageHasher::Twox256 => todo!(),
                        StorageHasher::Twox64Concat => todo!(),
                        StorageHasher::Identity => todo!(),
                    }
                }

                Ok(StorageEntry {
                    prefix: prefix_str.into(),
                    name: name_str.into(),
                    details: StorageEntryDetails::Map {
                        keys: storage_keys,
                        ty: *value
                    }
                })
            }
        }
    }

    // Reverse the prefix+name hashing (which takes the form of `twox_128(prefix) + twox_128(name)`)
    // into a specific storage location, which we can lookup in the Metadata to decode the remaining
    // bytes.
    fn decode_prefix_and_name_to_location(&self, data: &mut &[u8]) -> Result<StorageLocation, StorageDecodeError> {
        if data.len() < 32 {
            return Err(StorageDecodeError::NotEnoughBytesForPrefixAndName(data.len()))
        }
        let prefix_hash = &data[..16];
        let name_hash = &data[16..32];

        let entries = self.entries_by_hashed_prefix
            .get(prefix_hash)
            .ok_or(StorageDecodeError::PrefixNotFound)?;

        let entry_index = entries.entry_by_hashed_name
            .get(name_hash)
            .ok_or(StorageDecodeError::NameNotFound)?;

        // Successfully consumed the prefix and name bytes, so move our cursor.
        // In the case of errors, we leave the data "unconsumed".
        *data = &data[32..];

        Ok(StorageLocation {
            prefix_index: entries.index,
            entry_index: *entry_index
        })
    }
}

// Metadata info for maps/doublemaps contains a vec of hashers for each key type,
// and a Type representing the key(s). We expect the number of keys and hashers to
// line up, so let's resolve the keys into something easier to work with.
//
// See https://github.com/paritytech/subxt/blob/793c945fbd2de022f523c39a84ee02609ba423a9/codegen/src/api/storage.rs#L105
// for another example of this being handled in code.
fn storage_map_key_to_type_id_vec<'a>(metadata: &'a Metadata, key: &'a Type) -> Result<Vec<&'a Type>, StorageDecodeError> {
    match key.type_def() {
        // Multiple keys:
        scale_info::TypeDef::Tuple(vals) => {
            let types: Result<Vec<_>,_> = vals.fields()
                .iter()
                .map(|f| {
                    let id = f.id();
                    metadata.types().resolve(id).ok_or(StorageDecodeError::TypeNotFound(id))
                })
                .collect();
            types
        },
        // Single key:
        _ => {
            Ok(vec![key])
        }
    }
}

pub struct StorageEntry<'a> {
    pub prefix: Cow<'a, str>,
	pub name: Cow<'a, str>,
    pub details: StorageEntryDetails<'a>
}

pub enum StorageEntryDetails<'a> {
    Plain(TypeId),
    Map { keys: Vec<StorageKey<'a>>, ty: TypeId }
}

pub struct StorageKey<'a> {
    pub bytes: Cow<'a, [u8]>,
    pub ty: Cow<'a, Type>,
    pub hasher: StorageHasher,
}

/// A value which might be either a [`scale_info::Type`] or a `TypeId` that can be
/// resolved to a [`scale_info::Type`] via [`TypeOrId::to_type`].
pub enum TypeOrId<'a> {
    Type(Cow<'a, Type>),
    TypeId(TypeId)
}

impl <'a> TypeOrId<'a> {
    pub fn to_type<'m: 'a>(&'a self, metadata: &'m Metadata) -> Option<&'a Type> {
        use std::borrow::Borrow;
        match self {
            TypeOrId::Type(ty) => Some(ty.borrow()),
            TypeOrId::TypeId(id) => metadata.types().resolve(id.id())
        }
    }
    pub fn into_owned(self) -> TypeOrId<'static> {
        match self {
            TypeOrId::Type(ty) => {
                let ty = ty.into_owned();
                TypeOrId::Type(Cow::Owned(ty))
            },
            TypeOrId::TypeId(id) => TypeOrId::TypeId(id)
        }
    }
}