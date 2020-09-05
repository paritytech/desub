extern crate extras;
mod test_suite;

use desub_core::{decoder::{Decoder, Metadata}, SubstrateType};
use primitives::twox_128;
use codec::{Encode, Decode};

pub fn init() {
    pretty_env_logger::try_init();
}

#[test]
pub fn should_decode_ext342962() {
    init();
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block342962();
    let meta = Metadata::new(meta.as_slice());

    // block 6 of KSM CC3 is spec 1020
    decoder.register_version(1031, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder.decode_extrinsic(1031, e.as_slice()).expect("should decode");
        println!("{:?}", decoded);
        println!("{}", decoded);
    }

    // assert_eq!(vec![("now".to_string(), SubstrateType::U64(1577070096000))], decoded);
    // 1577070096000 is the UNIX timestamp in milliseconds of
    // Monday, December 23, 2019 3:01:36 AM
    // when block 342,962 was processed
}

#[test]
pub fn should_decode_ext422871() {
    init();
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block422871();
    let meta = Metadata::new(meta.as_slice());
    decoder.register_version(1031, &meta);

    println!("{}", ext.len());
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder.decode_extrinsic(1031, e.as_slice()).expect("should decode");
        println!("{}", decoded);
    }

}

#[test]
pub fn should_decode_ext50970() {
    init();
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block50970();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1031, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1031, e.as_slice()).expect("should decode");
        println!("{}", decoded);
    }
}

#[test]
pub fn should_decode_ext_106284() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block106284();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1042, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1042, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
pub fn should_decode_ext_1674683() {
    init();
 
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1674683();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
pub fn should_decode_ext_1677621() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1677621();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1702023() {
    init();
    
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1702023();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1714495() {
    init();
    
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1714495();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1717926() {
    init();
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1717926();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1718223() {
    init();
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1718223();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1732321() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1732321();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1731904() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1731904();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1768321() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block1768321();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1055, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_6144() {
    init();

    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let (meta, ext) = test_suite::extrinsics_block6144();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1020, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder.decode_extrinsic(1020, e.as_slice()).expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

use frame_system::AccountInfo; use pallet_balances::AccountData;
// hex(encoded): 010000000864000000000000000000000000000000c80000000000000000000000000000002c01000000000000000000000000000090010000000000000000000000000000
fn mock_account_info_data() -> (Vec<u8>, AccountInfo<u32, AccountData<u128>>) {
    let mock_account_data: AccountData<u128> = AccountData {
        free: 100,
        reserved: 200,
        misc_frozen: 300,
        fee_frozen: 400,
    };
    let mock_account_info: AccountInfo<u32, AccountData<u128>> = AccountInfo {
        nonce: 1,
        refcount: 8,
        data: mock_account_data,
    };
    (mock_account_info.encode(), mock_account_info)
}

    
/// T::BlockNumber in meta V11 Block 1768321
fn get_plain_value() -> (Vec<u8>, Vec<u8>){
    let mut key = twox_128("System".as_bytes()).to_vec();
    key.extend(twox_128("Number".as_bytes()).iter());
    let value = 1768321u32.encode();
    (key, value)
}

#[test]
fn should_decode_plain() {
    pretty_env_logger::try_init();
    
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types, "kusama");

    let meta = test_suite::runtime_v11();
    let meta = Metadata::new(meta.as_slice());
    decoder.register_version(2023, &meta);
    
    let res = decoder.decode_storage(2023, get_plain_value()).unwrap();
    assert_eq!(&SubstrateType::U32(1768321), res.value().unwrap().ty());
}


