extern crate extras;

use desub_core::{decoder::Decoder, test_suite};

#[test]
pub fn should_decode() {
    let types = extras::polkadot::PolkadotTypes::new().unwrap();
    let mut decoder = Decoder::new(types);

    let ext = test_suite::extrinsics_block10994();

    for e in ext.iter() {
        println!("{:X?}", e);
    }
}
