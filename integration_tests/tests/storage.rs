use crate::test_suite;
use codec::Encode;
use desub_core::{
	decoder::{Chain, Decoder, Metadata},
	SubstrateType,
};
use frame_system::AccountInfo;
use pallet_balances::AccountData;
use primitives::twox_128;

fn mock_account_info() -> AccountInfo<u32, AccountData<u128>> {
	let mock_account_data: AccountData<u128> =
		AccountData { free: 100, reserved: 200, misc_frozen: 300, fee_frozen: 400 };
	let mock_account_info: AccountInfo<u32, AccountData<u128>> = AccountInfo::default();
	mock_account_info
}

/// T::BlockNumber in meta V11 Block 1768321
fn get_plain_value() -> (Vec<u8>, Option<Vec<u8>>) {
	let mut key = twox_128("System".as_bytes()).to_vec();
	key.extend(twox_128("Number".as_bytes()).iter());
	let value = 1768321u32.encode();
	(key, Some(value))
}

#[test]
fn should_decode_plain() {
	let _ = pretty_env_logger::try_init();

	let types = extras::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = test_suite::runtime_v11();
	let meta = Metadata::new(meta.as_slice());
	decoder.register_version(2023, &meta);

	let res = decoder.decode_storage(2023, get_plain_value()).unwrap();
	assert_eq!(&SubstrateType::U32(1768321), res.value().unwrap().ty());
}

#[test]
fn should_decode_map() {
	let _ = pretty_env_logger::try_init();

	let types = extras::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = test_suite::runtime_v11();
	let meta = Metadata::new(meta.as_slice());
	decoder.register_version(2023, &meta);

	let account = mock_account_info();
	let encoded_account = account.encode();
	let storage_key = hex::decode("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da932a5935f6edc617ae178fef9eb1e211fbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f").unwrap();

	let res = decoder.decode_storage(2023, (storage_key, Some(encoded_account))).unwrap();
	println!("{:?}", res);
}

#[test]
fn should_decode_double_map() {
	let _ = pretty_env_logger::try_init();
	let types = extras::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = test_suite::runtime_v11();
	let meta = Metadata::new(meta.as_slice());
	decoder.register_version(2023, &meta);
	// THIS STORAGE KEY IS WRONG for "ImOnline AuthoredBlocks" type
	let storage_key = hex::decode("2b06af9719ac64d755623cda8ddd9b94b1c371ded9e9c565e89ba783c4d5f5f9b4def25cfda6ef3a00000000e535263148daaf49be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f").unwrap();
	let authored_blocks: u32 = 250;

	let res = decoder.decode_storage(2023, (storage_key, Some(authored_blocks.encode()))).unwrap();
	println!("{:?}", res);
}
