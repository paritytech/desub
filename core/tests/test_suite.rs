const EXT_PATH: &'static str = "./test/extrinsics/";
use std::{fs::File, io::prelude::*};

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 342962 with spec 1031
pub fn extrinsics_block342962() -> (Vec<u8>, [Vec<u8>; 2]) {
    let mut f =
        File::open("./test/extrinsics/spec1031_block342962/EXTRINSIC_spec_1031_block_342962_index_0.bin")
            .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f =
        File::open("./test/extrinsics/spec1031_block342962/EXTRINSIC_spec_1031_block_342962_index_1.bin")
            .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f = File::open(
        "./test/extrinsics/spec1031_block342962/spec_1031_block_342962_METADATA.bin",
    )
    .expect("Opening file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, [ext0, ext1])
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block422871() -> (Vec<u8>, [Vec<u8>; 3]) {
    let mut f =
        File::open("./test/extrinsics/spec1031_block422871/EXTRINSIC_spec_1031_block_422871_index_0.bin")
            .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f =
        File::open("./test/extrinsics/spec1031_block422871/EXTRINSIC_spec_1031_block_422871_index_1.bin")
            .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f =
        File::open("./test/extrinsics/spec1031_block422871/EXTRINSIC_spec_1031_block_422871_index_2.bin")
            .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");

    let mut f = File::open(
        "./test/extrinsics/spec1031_block422871/spec_1031_block_422871_METADATA.bin",
    )
    .expect("Opening file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, [ext0, ext1, ext2])
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block50970() -> (Vec<u8>, [Vec<u8>; 4]) {
    let path: String = format!("{}{}", EXT_PATH, "spec1031_block50970/");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1031_block_50970_index_0.bin"
    ))
    .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1031_block_50970_index_1.bin"
    ))
    .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1031_block_50970_index_2.bin"
    ))
    .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1031_block_50970_index_3.bin"
    ))
    .expect("Opening file failed");
    let mut ext3 = Vec::new();
    f.read_to_end(&mut ext3).expect("Reading file failed");

    let mut f = File::open(format!("{}{}", path, "spec_1031_block_50970_METADATA.bin"))
        .expect("Opening file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, [ext0, ext1, ext2, ext3])
}

/// returns raw metadata bytes and a vector of raw extrinsic bytes
/// from block 422871 with spec 1031
/// there are three extrinsics: FinalityTracker, Parachains and Timestmap
pub fn extrinsics_block106284() -> (Vec<u8>, [Vec<u8>; 4]) {
    let path: String = format!("{}{}", EXT_PATH, "spec1042_block106284/");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1042_block_106284_index_0.bin"
    ))
    .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1042_block_106284_index_1.bin"
    ))
    .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1042_block_106284_index_2.bin"
    ))
    .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");

    let mut f = File::open(format!(
        "{}{}",
        path, "EXTRINSIC_spec_1042_block_106284_index_3.bin"
    ))
    .expect("Opening file failed");
    let mut ext3 = Vec::new();
    f.read_to_end(&mut ext3).expect("Reading file failed");

    let mut f = File::open(format!("{}{}", path, "spec_1042_block_106284_METADATA.bin"))
        .expect("Opening file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, [ext0, ext1, ext2, ext3])
}
