use crate::runtime_metadata::*;
use desub_legacy::decoder::Metadata;
use sp_core::twox_128;

#[test]
fn should_create_metadata_v9() {
	let meta = runtime_v9();
	let meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
	println!("{}", meta.pretty());
	let meta = runtime_v9_block6();
	let _meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
}

#[test]
fn should_create_metadata_v10() {
	let meta = runtime_v10();
	let meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
	println!("{}", meta.pretty());
}

#[test]
fn should_create_metadata_v9_block500000() {
	let _ = pretty_env_logger::try_init();
	let meta = runtime_v9_block500k();
	let meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
	println!("{}", meta.pretty());
}

#[test]
fn should_create_metadata_v12_block_4643974() {
	let _ = pretty_env_logger::try_init();
	let meta = runtime_v12_block_4643974();
	let meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
	println!("{}", meta.pretty());
}

#[test]
fn should_get_correct_lookup_table() {
	let meta = runtime_v11();
	let meta: Metadata = Metadata::new(meta.as_slice()).unwrap();
	let lookup_table = meta.storage_lookup_table();
	let mut key = twox_128("System".as_bytes()).to_vec();
	key.extend(twox_128("Account".as_bytes()).iter());
	let storage_entry = lookup_table.lookup(&key);
	println!("{:?}", storage_entry);
	assert_eq!(storage_entry.unwrap().meta.prefix(), "System Account");
}
