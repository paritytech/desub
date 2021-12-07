use crate::runtime_metadata::*;
use anyhow::Result;
use codec::Encode;
use desub_legacy::{
	decoder::{Chain, Decoder, Metadata},
	SubstrateType,
};
use sp_core::twox_128;

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

	let types = desub_json_resolver::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = runtime_v11();
	let meta = Metadata::new(meta.as_slice()).unwrap();
	decoder.register_version(2023, meta).unwrap();

	let res = decoder.decode_storage(2023, get_plain_value()).unwrap();
	assert_eq!(&SubstrateType::U32(1768321), res.value().unwrap().ty());
}

#[test]
fn should_decode_map() -> Result<()> {
	let _ = pretty_env_logger::try_init();

	let types = desub_json_resolver::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = runtime_v11();
	let meta = Metadata::new(meta.as_slice()).unwrap();
	decoder.register_version(2023, meta).unwrap();
	// AccountInfo from block 3944196
	let encoded_account = hex::decode("01000000037c127ed1d8c6010000000000000000000000000000000000000000000000000000406352bfc60100000000000000000000406352bfc601000000000000000000").unwrap();
	let storage_key = hex::decode("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da932a5935f6edc617ae178fef9eb1e211fbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f").unwrap();

	let res = decoder.decode_storage(2023, (storage_key, Some(encoded_account)))?;
	println!("{:?}", res);
	Ok(())
}

#[test]
fn should_decode_map_ksm_3944195() -> Result<()> {
	let _ = pretty_env_logger::try_init();

	let types = desub_json_resolver::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = runtime_v11();
	let meta = Metadata::new(meta.as_slice()).unwrap();
	decoder.register_version(2023, meta).unwrap();
	// BlockHash from block 3944196
	let storage_key =
		hex::decode("26aa394eea5630e07c48ae0c9558cef7a44704b568d21667356a5a050c1187465eb805861b659fd1022f3c00")
			.unwrap();
	let encoded_hash = hex::decode("38f14d3d028e2f5b9ce889a444b49e774b88bcb3fe205fa4f5a10c2e66290c59").unwrap();

	let res = decoder.decode_storage(2023, (storage_key, Some(encoded_hash)))?;
	println!("{:?}", res);
	Ok(())
}

#[test]
fn should_decode_double_map() {
	let _ = pretty_env_logger::try_init();
	let types = desub_json_resolver::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	let meta = runtime_v11();
	let meta = Metadata::new(meta.as_slice()).unwrap();
	decoder.register_version(2023, meta).unwrap();
	// THIS STORAGE KEY IS WRONG for "ImOnline AuthoredBlocks" type
	let storage_key = hex::decode("2b06af9719ac64d755623cda8ddd9b94b1c371ded9e9c565e89ba783c4d5f5f9b4def25cfda6ef3a00000000e535263148daaf49be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f").unwrap();
	let authored_blocks: u32 = 250;

	let res = decoder.decode_storage(2023, (storage_key, Some(authored_blocks.encode()))).unwrap();
	println!("{:?}", res);
}
