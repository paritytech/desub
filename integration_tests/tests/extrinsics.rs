extern crate extras;

// TODO: Make test structure into a macro

use crate::test_suite;
use codec::{Decode, Encode};
use desub_core::{
    decoder::{Chain, Decoder, GenericStorage, Metadata, StorageHasher, StorageKey, StorageValue},
    SubstrateType,
};
use primitives::twox_128;

pub fn init() {
    pretty_env_logger::try_init();
}

#[test]
pub fn should_decode_ext342962() {
    init();
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block342962();
    let meta = Metadata::new(meta.as_slice());

    // block 6 of KSM CC3 is spec 1020
    decoder.register_version(1031, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder
            .decode_extrinsic(1031, e.as_slice())
            .expect("should decode");
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
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block422871();
    let meta = Metadata::new(meta.as_slice());
    decoder.register_version(1031, &meta);

    println!("{}", ext.len());
    for e in ext.iter() {
        println!("{:?}", e);
        let decoded = decoder
            .decode_extrinsic(1031, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
    }
}

#[test]
pub fn should_decode_ext50970() {
    init();
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block50970();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1031, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1031, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
    }
}

#[test]
pub fn should_decode_ext_106284() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block106284();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1042, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1042, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
pub fn should_decode_ext_1674683() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1674683();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
pub fn should_decode_ext_1677621() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1677621();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1702023() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1702023();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1714495() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1714495();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1717926() {
    init();
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1717926();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1718223() {
    init();
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1718223();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1732321() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1732321();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1731904() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1731904();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_1768321() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block1768321();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1055, &meta);

    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1055, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_6144() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block6144();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1020, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1020, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_779410_ksm() {
    init();
    
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block779410();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1042, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1042, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_899638_ksm() {
    init();
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block899638();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1042, &meta);
    for e in ext.iter() {
        println!("{:?}", e);
        println!("{:X?}", e);
        let decoded = decoder
            .decode_extrinsic(1042, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_233816_ksm() {
    init();
    
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block233816();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1030, &meta);
    for e in ext.iter() {
        println!("DECODING --------------------- \n {:X?} \n ------", e);
        let decoded = decoder
            .decode_extrinsic(1030, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_607421_ksm() {
    init();
    
    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Kusama);

    let (meta, ext) = test_suite::extrinsics_block607421();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(1039, &meta);
    for e in ext.iter() {
        println!("DECODING: \n ------ \n {:X?} \n ------", e);
        let decoded = decoder
            .decode_extrinsic(1039, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

#[test]
fn should_decode_ext_892_dot() {
    init();

    let types = extras::TypeResolver::default();
    let mut decoder = Decoder::new(types, Chain::Polkadot);

    let (meta, ext) = test_suite::extrinsics_block892_dot();
    let meta = Metadata::new(meta.as_slice());

    decoder.register_version(0, &meta);
    for e in ext.iter() {
        println!("DECODING: \n ------ \n {:X?} \n ------", e);
        let decoded = decoder
            .decode_extrinsic(0, e.as_slice())
            .expect("should decode");
        println!("{}", decoded);
        println!("{}", serde_json::to_string(&decoded).unwrap());
    }
}

