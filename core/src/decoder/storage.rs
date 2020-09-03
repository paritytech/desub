// pub mod metadata;
// pub mod storage_value;

use std::collections::HashMap;

use codec::{Decode, Encode};
use runtime_metadata_latest::{DecodeDifferent, StorageEntryType, StorageHasher};

use super::metadata::{Metadata, StorageMetadata};

////////////////////////////////////////////////////////////////////////
//    Storage Key/Value decode
////////////////////////////////////////////////////////////////////////

/// module prefix and storage prefix both use twx_128 hasher. One twox_128
/// hasher is 32 chars in hex string, i.e, the prefix length is 32 * 2.
pub const PREFIX_LENGTH: usize = 32 * 2;

/*
/// Map of StorageKey prefix (module_prefix++storage_prefix) in hex string to StorageMetadata.
///
/// So that we can know about the StorageMetadata given a complete StorageKey.
#[derive(Debug, Clone)]
pub struct StorageMetadataLookupTable(pub HashMap<String, StorageMetadata>);

impl From<Metadata> for StorageMetadataLookupTable {
    fn from(metadata: Metadata) -> Self {
        Self(
            metadata
                .modules
                .into_iter()
                .map(|(_, module_metadata)| {
                    module_metadata
                        .storage
                        .into_iter()
                        .map(|(_, storage_metadata)| {
                            let storage_prefix = storage_metadata.prefix();
                            (hex::encode(storage_prefix.0), storage_metadata)
                        })
                })
                .flatten()
                .collect(),
        )
    }
}
*/

// Transparent type of decoded StorageKey.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransparentStorageType {
    Plain {
        /// "u32"
        value_ty: String,
    },
    Map {
        /// value of key, e.g, "be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f" for T::AccountId
        key: String,
        /// type of value, e.g., "AccountInfo<T::Index, T::AccountData>"
        value_ty: String,
    },
    DoubleMap {
        key1: String,
        key1_ty: String,
        key2: String,
        key2_ty: String,
        value_ty: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransparentStorageKey {
    pub module_prefix: String,
    pub storage_prefix: String,
    pub ty: TransparentStorageType,
}

impl TransparentStorageKey {
    pub fn get_value_type(&self) -> String {
        match &self.ty {
            TransparentStorageType::Plain { value_ty } => value_ty.into(),
            TransparentStorageType::Map { value_ty, .. } => value_ty.into(),
            TransparentStorageType::DoubleMap { value_ty, .. } => value_ty.into(),
        }
    }
}

/// Converts the inner of `DecodeDifferent::Decoded(_)` to String.
fn as_decoded_type<B: 'static, O: 'static + Into<String>>(value: DecodeDifferent<B, O>) -> String {
    match value {
        DecodeDifferent::Encode(_b) => unreachable!("TODO: really unreachable?"),
        DecodeDifferent::Decoded(o) => o.into(),
    }
}

fn build_transparent_storage_key(
    storage_metadata: &StorageMetadata,
    ty: TransparentStorageType,
) -> TransparentStorageKey {
    TransparentStorageKey {
        module_prefix: String::from(&storage_metadata.module_prefix),
        storage_prefix: String::from(&storage_metadata.storage_prefix),
        ty,
    }
}

impl StorageMetadataLookupTable {
    /// Returns the StorageMetadata given the `prefix` of a StorageKey.
    pub fn lookup(&self, prefix: &str) -> Option<&StorageMetadata> {
        self.0.get(prefix)
    }

    /// Converts `storage_key` in hex string to a _readable_ format.
    pub fn parse_storage_key(&self, storage_key: String) -> Option<TransparentStorageKey> {
        let key_length = storage_key.chars().count();
        if key_length == 32 {
            println!("--------------- unknown 32 storage key:{:?}", storage_key);
            return None;
        }

        if key_length < 64 {
            println!("--------------- unknown < 64 storage key:{:?}", storage_key);
            return None;
        }

        let storage_prefix = &storage_key[..PREFIX_LENGTH];

        if let Some(storage_metadata) = self.lookup(storage_prefix) {
            match &storage_metadata.ty {
                StorageEntryType::Plain(value) => Some(TransparentStorageKey {
                    module_prefix: String::from(&storage_metadata.module_prefix),
                    storage_prefix: String::from(&storage_metadata.storage_prefix),
                    ty: TransparentStorageType::Plain {
                        value_ty: as_decoded_type(value.clone()),
                    },
                }),
                StorageEntryType::Map {
                    hasher,
                    key,
                    value,
                    unused,
                } => match hasher {
                    StorageHasher::Twox64Concat | StorageHasher::Blake2_128Concat => {
                        let hashed_key_concat = &storage_key[PREFIX_LENGTH..];
                        let hash_length = hash_length_of(hasher);
                        let _hashed_key = &hashed_key_concat[..hash_length];
                        let key = &hashed_key_concat[hash_length..];

                        let transparent_ty = TransparentStorageType::Map {
                            key: key.into(),
                            value_ty: as_decoded_type(value.clone()),
                        };

                        Some(build_transparent_storage_key(
                            &storage_metadata,
                            transparent_ty,
                        ))
                    }
                    _ => unreachable!("All Map storage should use foo_concat hasher"),
                },
                StorageEntryType::DoubleMap {
                    hasher,
                    key1,
                    key2,
                    value,
                    key2_hasher,
                } => {
                    // hashed_key1 ++ key1 ++ hashed_key2 ++ key2
                    let hashed_key_concat = &storage_key[PREFIX_LENGTH..];
                    match hasher {
                        StorageHasher::Twox64Concat | StorageHasher::Blake2_128Concat => {
                            let key1_hash_length = hash_length_of(hasher);

                            // key1 ++ hashed_key2 ++ key2
                            let key1_hashed_key2_key2 = &hashed_key_concat[key1_hash_length..];

                            let key1_ty = as_decoded_type(key1.clone());

                            if let Some(key1_length) = get_key1_length(key1_ty.clone()) {
                                let key1 = &key1_hashed_key2_key2[..key1_length];
                                let hashed_key2_key2 = &key1_hashed_key2_key2[key1_length..];

                                match key2_hasher {
                                    StorageHasher::Twox64Concat
                                    | StorageHasher::Blake2_128Concat => {
                                        let key2_hash_length = hash_length_of(key2_hasher);
                                        let raw_key2 = &hashed_key2_key2[key2_hash_length..];

                                        let key2_ty = as_decoded_type(key2.clone());

                                        let transparent_ty = TransparentStorageType::DoubleMap {
                                            key1: key1.into(),
                                            key1_ty,
                                            key2: raw_key2.into(),
                                            key2_ty,
                                            value_ty: as_decoded_type(value.clone()),
                                        };

                                        Some(build_transparent_storage_key(
                                            &storage_metadata,
                                            transparent_ty,
                                        ))
                                    }
                                    _ => unreachable!(
                                    "All DoubleMap storage should use foo_concat hasher for key2"
                                ),
                                }
                            } else {
                                println!("ERROR: can not infer the length of key1");
                                None
                            }
                        }
                        _ => unreachable!(
                            "All DoubleMap storage should use foo_concat hasher for key1"
                        ),
                    }
                }
            }
        } else {
            println!(
                "ERROR: can not find the StorageMetadata from lookup table for storage_key: {:?},
                prefix: {:?}",
                storage_key, storage_prefix
            );
            None
        }
    }
}

/// TODO: ensure all key1 in DoubleMap are included in this table.
///
/// NOTE: The lucky thing is that key1 of double_map normally uses the fixed size encoding.
fn get_double_map_key1_length_table() -> HashMap<String, u32> {
    let mut double_map_key1_length_table = HashMap::new();
    // For the test metadata:
    // [
    //  ("Kind", "OpaqueTimeSlot"),
    //  ("T::AccountId", "[u8; 32]"),
    //  ("EraIndex", "T::AccountId"),
    //  ("EraIndex", "T::AccountId"),
    //  ("EraIndex", "T::AccountId"),
    //  ("EraIndex", "T::AccountId"),
    //  ("EraIndex", "T::AccountId"),
    //  ("SessionIndex", "AuthIndex"),
    //  ("SessionIndex", "T::ValidatorId")
    // ]
    double_map_key1_length_table.insert(String::from("T::AccountId"), 64);
    // u32 hex::encode(1u32.encode()).chars().count()
    double_map_key1_length_table.insert(String::from("SessionIndex"), 8);
    // u32
    double_map_key1_length_table.insert(String::from("EraIndex"), 8);
    // Kind = [u8; 16]
    double_map_key1_length_table.insert(String::from("Kind"), 32);
    double_map_key1_length_table.insert(String::from("Chain"), 2);
    double_map_key1_length_table
}

/// Returns the length of key1 for a DoubleMap.
///
/// For key1 ++ hashed_key2 ++ key2, we already know the length of hashed_key2, plus
/// the length of key1, we could also infer the length of key2.
fn get_key1_length(key1_ty: String) -> Option<usize> {
    let table = get_double_map_key1_length_table();
    table.get(&key1_ty).copied().map(|x| x as usize)
}

/// Returns the length of this hasher in hex.
fn hash_length_of(hasher: &StorageHasher) -> usize {
    match hasher {
        StorageHasher::Blake2_128 => 32,
        StorageHasher::Blake2_256 => 32 * 2,
        StorageHasher::Blake2_128Concat => 32,
        StorageHasher::Twox128 => 32,
        StorageHasher::Twox256 => 32 * 2,
        StorageHasher::Twox64Concat => 16,
        StorageHasher::Identity => unreachable!(),
    }
}

fn generic_decode<T: codec::Decode>(encoded: Vec<u8>) -> Result<T, codec::Error> {
    Decode::decode(&mut encoded.as_slice())
}

// Filter out (key1, key2) pairs of all DoubleMap.
pub fn filter_double_map(metadata: Metadata) -> Vec<(String, String)> {
    metadata
        .modules
        .into_iter()
        .map(|(_, module_metadata)| {
            module_metadata
                .storage
                .into_iter()
                .filter_map(|(_, storage_metadata)| {
                    if let StorageEntryType::DoubleMap {
                        ref key1, ref key2, ..
                    } = storage_metadata.ty
                    {
                        let key1_ty = as_decoded_type(key1.clone());
                        let key2_ty = as_decoded_type(key2.clone());
                        Some((key1_ty, key2_ty))
                    } else {
                        None
                    }
                })
        })
        .flatten()
        .collect()
}

pub fn filter_double_map_key1_types(metadata: Metadata) -> Vec<String> {
    let keys_map: HashMap<String, String> = filter_double_map(metadata).into_iter().collect();
    let key1_type_set = keys_map.keys();
    key1_type_set.into_iter().map(|x| x.clone()).collect()
}

fn get_value_type(ty: StorageEntryType) -> String {
    match ty {
        StorageEntryType::Plain(value) => as_decoded_type(value),
        StorageEntryType::Map { value, .. } => as_decoded_type(value),
        StorageEntryType::DoubleMap { value, .. } => as_decoded_type(value),
    }
}

pub fn filter_storage_value_types(metadata: Metadata) -> Vec<String> {
    let mut value_types = metadata
        .modules
        .into_iter()
        .map(|(_, module_metadata)| {
            module_metadata
                .storage
                .into_iter()
                .map(|(_, storage_metadata)| get_value_type(storage_metadata.ty))
        })
        .flatten()
        .collect::<Vec<_>>();

    value_types.sort();
    value_types.dedup();
    value_types
}

/*
TODO: use a script to generate this function automatically.
[
    "(BalanceOf<T>, BalanceOf<T>, T::BlockNumber)",
    "(BalanceOf<T>, Vec<T::AccountId>)",
    "(OpaqueCall, T::AccountId, BalanceOf<T>)",
    "(Perbill, BalanceOf<T>)",
    "(T::AccountId, BalanceOf<T>, bool)",
    "(T::AccountId, Data)",
    "(T::BlockNumber, T::BlockNumber)",
    "(T::BlockNumber, Vec<T::AccountId>)",
    "(T::Hash, VoteThreshold)",
    "(Vec<(T::AccountId, T::ProxyType)>, BalanceOf<T>)",
    "(Vec<T::AccountId>, BalanceOf<T>)",
    "<T as Trait<I>>::Proposal",
    "AccountData<T::Balance>",
    "AccountInfo<T::Index, T::AccountData>",
    "AccountStatus<BalanceOf<T>>",
    "ActiveEraInfo",
    "BalanceOf<T>",
    "DigestOf<T>",
    "ElectionResult<T::AccountId, BalanceOf<T>>",
    "ElectionScore",
    "ElectionStatus<T::BlockNumber>",
    "EraIndex",
    "EraRewardPoints<T::AccountId>",
    "EthereumAddress",
    "EventIndex",
    "Exposure<T::AccountId, BalanceOf<T>>",
    "Forcing",
    "LastRuntimeUpgradeInfo",
    "MaybeRandomness",
    "Multiplier",
    "Multisig<T::BlockNumber, BalanceOf<T>, T::AccountId>",
    "NextConfigDescriptor",
    "Nominations<T::AccountId>",
    "OffenceDetails<T::AccountId, T::IdentificationTuple>",
    "OpenTip<T::AccountId, BalanceOf<T>, T::BlockNumber, T::Hash>",
    "Perbill",
    "Phase",
    "PreimageStatus<T::AccountId, BalanceOf<T>, T::BlockNumber>",
    "PropIndex",
    "Proposal<T::AccountId, BalanceOf<T>>",
    "ProposalIndex",
    "ReferendumIndex",
    "ReferendumInfo<T::BlockNumber, T::Hash, BalanceOf<T>>",
    "Registration<BalanceOf<T>>",
    "Releases",
    "RewardDestination",
    "SessionIndex",
    "SetId",
    "StakingLedger<T::AccountId, BalanceOf<T>>",
    "StatementKind",
    "StoredPendingChange<T::BlockNumber>",
    "StoredState<T::BlockNumber>",
    "T::AccountId",
    "T::Balance",
    "T::BlockNumber",
    "T::Hash",
    "T::Keys",
    "T::Moment",
    "T::ValidatorId",
    "TaskAddress<T::BlockNumber>",
    "ValidatorPrefs",
    "Vec<(AuthorityId, BabeAuthorityWeight)>",
    "Vec<(EraIndex, SessionIndex)>",
    "Vec<(PropIndex, T::Hash, T::AccountId)>",
    "Vec<(T::AccountId, BalanceOf<T>)>",
    "Vec<(T::BlockNumber, EventIndex)>",
    "Vec<(T::ValidatorId, T::Keys)>",
    "Vec<BalanceLock<T::Balance>>",
    "Vec<DeferredOffenceOf<T>>",
    "Vec<EventRecord<T::Event, T::Hash>>",
    "Vec<Option<RegistrarInfo<BalanceOf<T>, T::AccountId>>>",
    "Vec<Option<Scheduled<<T as Trait>::Call, T::BlockNumber, T::\nPalletsOrigin, T::AccountId>>>",
    "Vec<ProposalIndex>",
    "Vec<ReportIdOf<T>>",
    "Vec<T::AccountId>",
    "Vec<T::AuthorityId>",
    "Vec<T::BlockNumber>",
    "Vec<T::Hash>",
    "Vec<T::ValidatorId>",
    "Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>",
    "Vec<UncleEntryItem<T::BlockNumber, T::Hash, T::AccountId>>",
    "Vec<schnorrkel::Randomness>",
    "Vec<u32>",
    "Vec<u8>",
    "VestingInfo<BalanceOf<T>, T::BlockNumber>",
    "Votes<T::AccountId, T::BlockNumber>",
    "Voting<BalanceOf<T>, T::AccountId, T::BlockNumber>",
    "bool",
    "schnorrkel::Randomness",
    "slashing::SlashingSpans",
    "slashing::SpanRecord<BalanceOf<T>>",
    "u32",
    "u64",
    "weights::ExtrinsicsWeight",
]
*/

/*
#[cfg(test)]
mod tests {
    use super::*;
    use frame_metadata::RuntimeMetadataPrefixed;
    use frame_system::AccountInfo;
    use pallet_balances::AccountData;
    use polkadot_primitives::v1::{AccountIndex, Balance};
    use std::convert::TryInto;
    fn get_metadata() -> Metadata {
        let s = include_str!("../test_data/metadata.txt");
        let s = s.trim();
        // string hex
        // decode hex string without 0x prefix
        let data = hex::decode(s).unwrap();
        let meta: RuntimeMetadataPrefixed =
            Decode::decode(&mut data.as_slice()).expect("failed to decode metadata prefixed");
        meta.try_into().expect("failed to convert to metadata")
    }
    // Filter out (key1, key2) pairs of all DoubleMap.
    fn filter_double_map() -> Vec<(String, String)> {
        let metadata = get_metadata();
        let double_map_keys = metadata
            .modules
            .into_iter()
            .map(|(_, module_metadata)| {
                module_metadata
                    .storage
                    .into_iter()
                    .filter_map(|(_, storage_metadata)| {
                        if let StorageEntryType::DoubleMap {
                            ref key1, ref key2, ..
                        } = storage_metadata.ty
                        {
                            let key1_ty = as_decoded_type(key1.clone());
                            let key2_ty = as_decoded_type(key2.clone());
                            Some((key1_ty, key2_ty))
                        } else {
                            None
                        }
                    })
            })
            .flatten()
            .collect::<Vec<_>>();
        double_map_keys
    }
    fn filter_double_map_key1_types() -> Vec<String> {
        let keys_map: HashMap<String, String> = filter_double_map().into_iter().collect();
        let key1_type_set = keys_map.keys();
        key1_type_set.into_iter().map(|x| x.clone()).collect()
    }
    fn get_value_type(ty: StorageEntryType) -> String {
        match ty {
            StorageEntryType::Plain(value) => as_decoded_type(value),
            StorageEntryType::Map { value, .. } => as_decoded_type(value),
            StorageEntryType::DoubleMap { value, .. } => as_decoded_type(value),
        }
    }
    fn filter_storage_value_types() -> Vec<String> {
        let metadata = get_metadata();
        let mut value_types = metadata
            .modules
            .into_iter()
            .map(|(_, module_metadata)| {
                module_metadata
                    .storage
                    .into_iter()
                    .map(|(_, storage_metadata)| get_value_type(storage_metadata.ty))
            })
            .flatten()
            .collect::<Vec<_>>();
        value_types.sort();
        value_types.dedup();
        value_types
    }
    // hex(encoded): 010000000864000000000000000000000000000000c80000000000000000000000000000002c01000000000000000000000000000090010000000000000000000000000000
    fn mock_account_info_data() -> (Vec<u8>, AccountInfo<AccountIndex, AccountData<Balance>>) {
        let mock_account_data: AccountData<Balance> = AccountData {
            free: 100,
            reserved: 200,
            misc_frozen: 300,
            fee_frozen: 400,
        };
        let mock_account_info: AccountInfo<AccountIndex, AccountData<Balance>> = AccountInfo {
            nonce: 1,
            refcount: 8,
            data: mock_account_data,
        };
        (mock_account_info.encode(), mock_account_info)
    }
    #[test]
    fn prase_storage_map_should_work() {
        //  twox_128("System"): 0x26aa394eea5630e07c48ae0c9558cef7
        // twox_128("Account"): 0xb99d880ec681799c0cf30e8886371da9
        //
        //      Account ID: 0xbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f
        // Blake2 128 Hash: 0x32a5935f6edc617ae178fef9eb1e211f
        let metadata = get_metadata();
        let table: StorageMetadataLookupTable = metadata.into();
        let storage_key = "26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da932a5935f6edc617ae178fef9eb1e211fbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f";
        let expected = TransparentStorageKey {
            module_prefix: "System".into(),
            storage_prefix: "Account".into(),
            ty: TransparentStorageType::Map {
                key: "be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f".into(),
                value_ty: "AccountInfo<T::Index, T::AccountData>".into(),
            },
        };
        assert_eq!(
            table.parse_storage_key(storage_key.into()).unwrap(),
            expected
        );
        // Firstly, we need to build Storage Value decode function table.
        let mut storage_value_decode_fn_map = HashMap::new();
        let try_decode_account_info = |encoded: Vec<u8>| {
            generic_decode::<AccountInfo<AccountIndex, AccountData<Balance>>>(encoded)
        };
        // let try_decode_balance = |encoded: Vec<u8>| generic_decode::<Balance>(encoded);
        storage_value_decode_fn_map.insert(
            String::from("AccountInfo<T::Index, T::AccountData>"),
            try_decode_account_info,
        );
        let storage_value = "010000000864000000000000000000000000000000c80000000000000000000000000000002c01000000000000000000000000000090010000000000000000000000000000";
        println!(
            "--------------- try_decode_storage_value:{:?}",
            try_decode_storage_value("AccountInfo<T::Index, T::AccountData>", storage_value)
        );
        if let TransparentStorageType::Map { key, value_ty } = expected.ty {
            let decode_fn = storage_value_decode_fn_map.get(&value_ty).unwrap();
            let decoded_value = decode_fn(hex::decode(storage_value).unwrap()).unwrap();
            let expected_decoded_value = mock_account_info_data().1;
            assert_eq!(decoded_value, expected_decoded_value);
        } else {
            panic!("Not Map")
        }
    }
    #[test]
    fn parse_storage_double_map_should_work() {
        //       ImOnline 0x2b06af9719ac64d755623cda8ddd9b94
        // AuthoredBlocks 0xb1c371ded9e9c565e89ba783c4d5f5f9
        // key1 twox_64_concat SessionIndex
        // key2 twxo_64_concat T::ValidatorId
        let storage_key = "2b06af9719ac64d755623cda8ddd9b94b1c371ded9e9c565e89ba783c4d5f5f9b4def25cfda6ef3a00000000e535263148daaf49be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f";
        let metadata = get_metadata();
        let table: StorageMetadataLookupTable = metadata.into();
        println!("{:?}", table.parse_storage_key(storage_key.into()));
        let expected = TransparentStorageKey {
            module_prefix: "ImOnline".into(),
            storage_prefix: "AuthoredBlocks".into(),
            ty: TransparentStorageType::DoubleMap {
                key1: "00000000".into(),
                key1_ty: "SessionIndex".into(),
                key2: "be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f".into(),
                key2_ty: "T::ValidatorId".into(),
                value_ty: "u32".into(),
            },
        };
        assert_eq!(
            table.parse_storage_key(storage_key.into()).unwrap(),
            expected
        );
    }
    #[test]
    fn test_decode_storage_value() {
        use codec::Encode;
        use frame_system::AccountInfo;
        use pallet_balances::AccountData;
        use polkadot_primitives::v1::{AccountIndex, Balance};
        use std::collections::HashMap;
        // Firstly, we need to build Storage Value decode function table by hand.
        let mut storage_value_decode_fn_map = HashMap::new();
        let try_decode_account_info = |encoded: Vec<u8>| {
            generic_decode::<AccountInfo<AccountIndex, AccountData<Balance>>>(encoded)
        };
        storage_value_decode_fn_map.insert(
            String::from("AccountInfo<T::Index, T::AccountData>"),
            try_decode_account_info,
        );
        let mock_account_data: AccountData<Balance> = AccountData {
            free: 100,
            reserved: 200,
            misc_frozen: 300,
            fee_frozen: 400,
        };
        let mock_account_info: AccountInfo<AccountIndex, AccountData<Balance>> = AccountInfo {
            nonce: 1,
            refcount: 8,
            data: mock_account_data,
        };
        let encoded_account_info = mock_account_info.encode();
        if let Some(decode_fn) =
            storage_value_decode_fn_map.get("AccountInfo<T::Index, T::AccountData>")
        {
            assert_eq!(decode_fn(encoded_account_info).unwrap(), mock_account_info);
        }
        println!("----- {:#?}", filter_storage_value_types());
        println!("----- {:#?}", filter_double_map_key1_types());
        println!("----- {:#?}", hex::encode(1u32.encode()).chars().count());
        println!(
            "------ {:#?}",
            hex::encode(b"im-online:offlin".encode()).chars().count()
        );
        println!(
            "------ {:#?}",
            hex::encode([1u8; 16].encode()).chars().count()
        );
    }
}
*/
