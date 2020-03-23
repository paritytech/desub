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
