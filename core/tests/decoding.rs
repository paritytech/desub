extern crate extras;

use desub_core::{decoder::{Decoder, Metadata}, SubstrateType, test_suite};

#[test]
pub fn should_decode_ext342962() {

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block342962();
    let meta = Metadata::new(meta.as_slice());

    // block 6 of KSM CC3 is spec 1020
    decoder.register_version(1031, meta);
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder.decode_extrinsic(1032, e.as_slice()).expect("should decode");
        println!("{:?}", decoded);
    }

    // assert_eq!(vec![("now".to_string(), SubstrateType::U64(1577070096000))], decoded);
    // 1577070096000 is the UNIX timestamp in milliseconds of
    // Monday, December 23, 2019 3:01:36 AM
    // when block 342,962 was processed
}

#[test]
pub fn should_decode_ext422871() {

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block422871();
    let meta = Metadata::new(meta.as_slice());
    decoder.register_version(1031, meta);

    println!("{}", ext.len());
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder.decode_extrinsic(1031, e.as_slice()).expect("should decode");
        println!("{:?}", decoded);
    }

}

#[test]
pub fn should_decode_ext50970() {
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block50970();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1031, meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1031, e.as_slice()).expect("should decode");
        println!("{:?}", decoded);
    }
}
