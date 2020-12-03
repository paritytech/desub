const EXT_PATH: &'static str = "./data/extrinsics/";
use desub_core::decoder::Chain;
use std::{fs::File, io::prelude::*};

use paste::paste;

#[macro_export]
macro_rules! extrinsic_test {
    (
        $(
            [$spec:expr, $chain: expr, $block:expr]
        )*
    ) => {
        $(
            paste! {
                pub fn [<extrinsics_block_ $block>]() -> (Vec<u8>, Vec<Vec<u8>>) {
                    let mut exts: Vec<Vec<u8>> = Vec::new();
                    if std::path::Path::new("./integration_tests").exists() {
                        std::env::set_current_dir("./integration_tests").unwrap();
                    }
                    let path = format!("{}{}/", EXT_PATH, $chain);
                    let path = &format!("{}spec{}_block{}/", path, $spec, $block);
                    println!(
                        "{}/{}",
                        path,
                        std::env::current_dir().unwrap().to_str().unwrap()
                    );

                    // get the number of files with prefix ${path}_EXTRINSIC in the directory.
                    let num_ext = std::fs::read_dir(&path)
                        .unwrap()
                        .map(|d| d.unwrap().file_name().into_string().unwrap())
                        .filter(|ext| ext.starts_with("EXTRINSIC"))
                        .count();
                    for i in 0..num_ext {
                        let ext_path = &format!(
                            "{}EXTRINSIC_spec_{}_block_{}_index_{}.bin",
                            &path, $spec, $block, i
                        );
                        let mut f = File::open(ext_path).expect("Opening extrinsic failed");
                        let mut ext = Vec::new();
                        f.read_to_end(&mut ext).expect("Reading file failed");
                        exts.push(ext)
                    }

                    let mut f = File::open(&format!(
                        "{}spec_{}_block_{}_METADATA.bin",
                        &path, $spec, $block
                    )).expect("Opening Metadata file failed");

                    let mut meta = Vec::new();
                    f.read_to_end(&mut meta).expect("Reading file failed");

                    (meta, exts)
                }
            }
        )*
    };
}

extrinsic_test!(
    ["1031", Chain::Kusama, "342962"]
    ["1031", Chain::Kusama, "422871"]
    ["1031", Chain::Kusama, "50970"]
    ["1042", Chain::Kusama, "106284"]
    ["1055", Chain::Kusama, "1674683"]
    ["1055", Chain::Kusama, "1677621"]
    ["1055", Chain::Kusama, "1702023"]
    ["1055", Chain::Kusama, "1714495"]
    ["1055", Chain::Kusama, "1717926"]
    ["1055", Chain::Kusama, "1718223"]
    ["1055", Chain::Kusama, "1732321"]
    ["1055", Chain::Kusama, "1731904"]
    ["1055", Chain::Kusama, "1768321"]
    ["1020", Chain::Kusama, "6144"]
    ["1042", Chain::Kusama, "779410"]
    ["1042", Chain::Kusama, "899638"]
    ["1030", Chain::Kusama, "233816"]
    ["1039", Chain::Kusama, "607421"]
    ["0", Chain::Polkadot, "892"]
    ["1", Chain::Westend, "1191"]
);

/// Get the runtime metadata from KusamaCC3 from block 3,901,874
/// Block hash 0x1d65a4c67817c4f32f99f7247f070a2f3fd58baf81d4e533c9be9d1aa8c4e65a
///
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v11() -> Vec<u8> {
    let mut f = File::open("./data/metadata_v11.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

/// Get the runtime metadata from KusamaCC3 around block 361,0000
/// Block hash
/// 0x627a6a8e7698dd360bd44e7816e7f8c5321fa31e0a3f39324d93ec5716a57fb5
///
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v9() -> Vec<u8> {
    let mut f = File::open("./data/metadata_v9.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

/// Get the runtime metadata from KusamaCC3 around block 361,0000
/// Block 500,000
/// Block hash 0x166f5cc1a51a702b79171455c0f3aa3cc6ba010075c1aaa86e1b9e8067510806
///
//
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v9_block500k() -> Vec<u8> {
    let mut f = File::open("./data/metadata_v9_block500000.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

/// Get the runtime metadata from KusamaCC3 for metadata version 10
/// Block hash
/// 0x627a6a8e7698dd360bd44e7816e7f8c5321fa31e0a3f39324d93ec5716a57fb5
///
/// # Panics
/// Panics on std::io::Error
pub fn runtime_v10() -> Vec<u8> {
    let mut f = File::open("./data/metadata_v10.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}

pub fn runtime_v12_block_4643974() -> Vec<u8> {
    let mut f = File::open("./data/metadata_v12_block4643974.bin").expect("Opening file failed");
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
    let mut f = File::open("./data/metadata_v9_block6.bin").expect("Opening file failed");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Reading file failed");
    buffer
}
