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

use codec::Encode;
use desub_current::{
	decoder::{self, StorageHasher},
	Metadata, Value,
};

static V14_METADATA_POLKADOT_SCALE: &[u8] = include_bytes!("data/v14_metadata_polkadot.scale");

fn metadata() -> Metadata {
	Metadata::from_bytes(V14_METADATA_POLKADOT_SCALE).expect("valid metadata")
}

fn account_id_to_value<A: AsRef<[u8]>>(account_id_bytes: A) -> Value<()> {
	Value::unnamed_composite(vec![Value::unnamed_composite(
		account_id_bytes.as_ref().iter().map(|&b| Value::u8(b)).collect(),
	)])
}

macro_rules! assert_hasher_eq {
	($actual:expr, $hasher:path, $value:expr) => {
		if let $hasher(val) = &$actual {
			assert_eq!(val.clone().without_context(), $value);
		} else {
			panic!("Passed {:?}, but expected hasher {}", $actual, stringify!($hasher));
		}
	};
}

macro_rules! bytes {
	($name:ident = $hex:literal) => {
		let hex_str = $hex.strip_prefix("0x").expect("0x should prefix hex encoded bytes");
		let bytes = hex::decode(hex_str).expect("valid bytes from hex");
		let $name = &mut &*bytes;
	};
}

// A very basic storage query; get the current timestamp.
#[test]
fn timestamp_now() {
	let meta = metadata();
	let storage = decoder::decode_storage(&meta);

	// Timestamp.Now(): u64
	bytes!(storage_key = "0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb");

	let entry = storage.decode_key(&meta, storage_key).expect("can decode storage");
	assert!(storage_key.is_empty(), "No more bytes expected");
	assert_eq!(entry.prefix, "Timestamp");
	assert_eq!(entry.name, "Now");

	// We can decode values at this location, now:
	let bytes = 123u64.encode();
	let val = decoder::decode_value_by_id(&meta, &entry.ty, &mut &*bytes).unwrap();
	assert_eq!(val.without_context(), Value::u64(123));
}

// A simple map lookup with an Identity hash (ie just the key itself)
#[test]
fn democracy_blacklist() {
	let meta = metadata();
	let storage = decoder::decode_storage(&meta);

	// Democracy.Blacklist([1u8; 32]: H256): ..
	bytes!(storage_key = "0xf2794c22e353e9a839f12faab03a911bb7612c99e31defd01cd5a28e9967e2080101010101010101010101010101010101010101010101010101010101010101");

	let entry = storage.decode_key(&meta, storage_key).expect("can decode storage");
	assert!(storage_key.is_empty(), "No more bytes expected");
	assert_eq!(entry.prefix, "Democracy");
	assert_eq!(entry.name, "Blacklist");

	let keys = entry.details.map_keys();

	// Because the hasher is Identity, we can even see the decoded original map key:
	assert_eq!(keys.len(), 1);
	assert_hasher_eq!(
		keys[0].hasher,
		StorageHasher::Identity,
		Value::unnamed_composite(vec![Value::unnamed_composite(vec![Value::u8(1); 32])])
	);
	assert!(matches!(keys[0].hasher, StorageHasher::Identity(..)));
}

// A map storage entry with a Twox64Concat key.
#[test]
fn system_blockhash() {
	let meta = metadata();
	let storage = decoder::decode_storage(&meta);

	// System.BlockHash(1000): [u8; 32]
	bytes!(storage_key = "0x26aa394eea5630e07c48ae0c9558cef7a44704b568d21667356a5a050c118746b6ff6f7d467b87a9e8030000");

	let entry = storage.decode_key(&meta, storage_key).expect("can decode storage");
	assert!(storage_key.is_empty(), "No more bytes expected");
	assert_eq!(entry.prefix, "System");
	assert_eq!(entry.name, "BlockHash");

	let keys = entry.details.map_keys();

	// Because the hasher is Twox64Concat, we can even see the decoded original map key:
	assert_eq!(keys.len(), 1);
	assert_hasher_eq!(keys[0].hasher, StorageHasher::Twox64Concat, Value::u32(1000));

	// We can decode values at this location:
	let bytes = [1u8; 32].encode();
	let val = decoder::decode_value_by_id(&meta, &entry.ty, &mut &*bytes).unwrap();
	assert_eq!(
		val.without_context(),
		// The Type appears to take the form of a newtype-wrapped [u8; 32]:
		Value::unnamed_composite(vec![Value::unnamed_composite(vec![Value::u8(1); 32])])
	);
}

// A simple map lookup like above, but using a Blake2_128Concat key
#[test]
fn balances_account() {
	let meta = metadata();
	let storage = decoder::decode_storage(&meta);

	// Balances.Account(BOB: AccountId32): PalletBalancesAccountData
	bytes!(storage_key = "0xc2261276cc9d1f8598ea4b6a74b15c2fb99d880ec681799c0cf30e8886371da94f9aea1afa791265fae359272badc1cf8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48");

	let entry = storage.decode_key(&meta, storage_key).expect("can decode storage");
	assert!(storage_key.is_empty(), "No more bytes expected");
	assert_eq!(entry.prefix, "Balances");
	assert_eq!(entry.name, "Account");

	let keys = entry.details.map_keys();

	let bobs_accountid = sp_keyring::AccountKeyring::Bob.to_account_id();
	let bobs_value = account_id_to_value(&bobs_accountid);

	assert_eq!(keys.len(), 1);
	assert_hasher_eq!(keys[0].hasher, StorageHasher::Blake2_128Concat, bobs_value);
}

// A map storage entry keyed by a tuple of 2 Twox64Concat values.
#[test]
fn imonline_authoredblocks() {
	let meta = metadata();
	let storage = decoder::decode_storage(&meta);

	// ImOnline.AuthoredBlocks(1234: u32, BOB:AccountId32): u32
	bytes!(storage_key = "0x2b06af9719ac64d755623cda8ddd9b94b1c371ded9e9c565e89ba783c4d5f5f9548491cbfe725727d2040000a647e755c30521d38eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48");

	let entry = storage.decode_key(&meta, storage_key).expect("can decode storage");
	assert!(storage_key.is_empty(), "No more bytes expected");
	assert_eq!(entry.prefix, "ImOnline");
	assert_eq!(entry.name, "AuthoredBlocks");

	let keys = entry.details.map_keys();

	let bobs_accountid = sp_keyring::AccountKeyring::Bob.to_account_id();
	let bobs_value = account_id_to_value(&bobs_accountid);

	// Because the hashers are Twox64Concat, we can check the keys we provided:
	assert_eq!(keys.len(), 2);
	assert_hasher_eq!(keys[0].hasher, StorageHasher::Twox64Concat, Value::u32(1234));
	assert_hasher_eq!(keys[1].hasher, StorageHasher::Twox64Concat, bobs_value);

	// We can decode values at this location:
	let bytes = 5678u32.encode();
	let val = decoder::decode_value_by_id(&meta, &entry.ty, &mut &*bytes).unwrap();
	assert_eq!(val.without_context(), Value::u32(5678));
}
