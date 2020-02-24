extern crate extras;

use desub_core::{decoder::{Decoder, Metadata}, test_suite};
use codec::{Compact, Decode};
// use std::mem;

#[test]
pub fn should_decode() {
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types);

    let meta = Metadata::new(test_suite::runtime_v9_block6().as_slice());
    println!("{}", meta.pretty());
    // println!("{:#?}", meta);
    // block 6 of KSM CC3 is spec 1020
    decoder.register_version(1020, meta);
    let ext = test_suite::extrinsics_block10994();

    for e in ext.iter() {
        println!("{:X?}", e);
    }

    println!("{:x?}", &ext[1][3..]);
    println!("{:?}", &ext[1][3..]);
    for d in ext[0][3..11].iter() {
        print!("{:08b}", d);
    }
    println!();
    for d in ext[0][3..11].iter().rev() {
        print!("{:08b}", d);
    }

    println!();
    let stamp: Compact<u64> = Decode::decode(&mut &ext[0][4..11]).unwrap();
    println!("{:?}", stamp);
}
