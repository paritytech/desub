use std::{fs::File, io::prelude::*};

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
