const EXT_PATH: &'static str = "./test/extrinsics/";
use std::{fs::File, io::prelude::*};

fn extrinsic_test(spec: String, block: String, num: usize) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut exts: Vec<Vec<u8>> = Vec::new();
    let path = &format!("{}spec{}_block{}/", EXT_PATH, spec, block);
    for i in 0..num {
        let ext_path = &format!("{}EXTRINSIC_spec_{}_block_{}_index_{}.bin", &path, spec, block, i);
        let mut f = File::open(ext_path).expect("Opening extrinsic failed");
        let mut ext = Vec::new();
        f.read_to_end(&mut ext).expect("Reading file failed");
        exts.push(ext)
    }

    let mut f = File::open(&format!("{}spec_{}_block_{}_METADATA.bin", &path, spec, block)) 
        .expect("Opening Metadata file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, exts)
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 342962 with spec 1031
pub fn extrinsics_block342962() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1031".to_string(), "342962".to_string(), 2)
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block422871() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1031".to_string(), "422871".to_string(), 3)
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block50970() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1031".to_string(), "50970".to_string(), 4)
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block106284() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1042".to_string(), "106284".to_string(), 4)
}

pub fn extrinsics_block1674683() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1055".to_string(), "1674683".to_string(), 3)
}

pub fn extrinsics_block1677621() -> (Vec<u8>, Vec<Vec<u8>>) {
    extrinsic_test("1055".to_string(), "1677621".to_string(), 4)
}