// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

const EXT_PATH: &'static str = "./test/extrinsics/";

use runtime_version::RuntimeVersion;

use std::{borrow::Cow, fs::File, io::prelude::*};

pub fn mock_runtime(num: u32) -> RuntimeVersion {
    RuntimeVersion {
        spec_name: "test-runtime".into(),
        impl_name: "test-runtime-impl".into(),
        authoring_version: num,
        spec_version: num,
        impl_version: num,
        apis: Cow::from(Vec::new()),
    }
}

/// Get some runtime metadata from KusamaCC3 around block 361,0000
/// Block hash
/// 0x627a6a8e7698dd360bd44e7816e7f8c5321fa31e0a3f39324d93ec5716a57fb5
///
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v9() -> Vec<u8> {
    let mut f = File::open("./test/metadata_v9.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

pub fn runtime_v10() -> Vec<u8> {
    let mut f = File::open("./test/metadata_v10.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

/// Get some runtime metadata from KusamaCC3 at block 6
/// Block hash
/// 0xb5ee550d20a55b76adeba7149516d367ac7cbdd95cd0864a8753d6b5dd02d3bb
///
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v9_block6() -> Vec<u8> {
    let mut f = File::open("./test/metadata_v9_block6.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

/// Get extrinsics from block 10994 on Kusama CC3
///
/// # Panics
/// Panics on std::io::Error
pub fn extrinsics_block10994() -> [Vec<u8>; 3] {
    let mut f =
        File::open("./test/extrinsics/spec1020_block10994/EXTRINSIC_spec_1020_block_10994_index_0.bin")
            .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f =
        File::open("./test/extrinsics/spec1020_block10994/EXTRINSIC_spec_1020_block_10994_index_1.bin")
            .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f =
        File::open("./test/extrinsics/spec1020_block10994/EXTRINSIC_spec_1020_block_10994_index_2.bin")
            .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");

    [ext0, ext1, ext2]
}

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

    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1031_block_50970_index_0.bin"))
            .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");

    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1031_block_50970_index_1.bin"))
                    .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");

    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1031_block_50970_index_2.bin"))
            .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");

    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1031_block_50970_index_3.bin"))
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

    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1042_block_106284_index_0.bin"))
        .expect("Opening file failed");
    let mut ext0 = Vec::new();
    f.read_to_end(&mut ext0).expect("Reading file failed");


    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1042_block_106284_index_1.bin"))
        .expect("Opening file failed");
    let mut ext1 = Vec::new();
    f.read_to_end(&mut ext1).expect("Reading file failed");


    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1042_block_106284_index_2.bin"))
        .expect("Opening file failed");
    let mut ext2 = Vec::new();
    f.read_to_end(&mut ext2).expect("Reading file failed");


    let mut f =
        File::open(format!("{}{}", path, "EXTRINSIC_spec_1042_block_106284_index_3.bin"))
        .expect("Opening file failed");
    let mut ext3 = Vec::new();
    f.read_to_end(&mut ext3).expect("Reading file failed");


    let mut f = File::open(format!("{}{}", path, "spec_1042_block_106284_METADATA.bin"))
        .expect("Opening file failed");
    let mut meta = Vec::new();
    f.read_to_end(&mut meta).expect("Reading file failed");

    (meta, [ext0, ext1, ext2, ext3])
}
