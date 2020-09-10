use crate::test_suite;
use primitives::twox_128;
use desub_core::decoder::Metadata;

#[test]
fn should_create_metadata_v9() {
    let meta = test_suite::runtime_v9();
    let meta: Metadata = Metadata::new(meta.as_slice());
    println!("{}", meta.pretty());
    let meta = test_suite::runtime_v9_block6();
    let _meta: Metadata = Metadata::new(meta.as_slice());
}

#[test]
fn should_create_metadata_v10() {
    let meta = test_suite::runtime_v10();
    let meta: Metadata = Metadata::new(meta.as_slice());
    println!("{}", meta.pretty());
}

#[test]
fn should_get_correct_lookup_table() {
    let meta = test_suite::runtime_v11();
    let meta: Metadata = Metadata::new(meta.as_slice());
    let lookup_table = meta.storage_lookup_table();
    let mut key = twox_128("System".as_bytes()).to_vec();
    key.extend(twox_128("Account".as_bytes()).iter());
    let storage_entry = lookup_table.lookup(&key);
    println!("{:?}", storage_entry);
    assert_eq!(storage_entry.unwrap().meta.prefix(), "System Account");
}

