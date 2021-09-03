mod runtime_metadata;

const EXT_PATH: &'static str = "./data/extrinsics/";
use desub_core::decoder::Chain;
use std::{fs::File, io::prelude::*};

use paste::paste;

pub use runtime_metadata::*;

// NOTE: it is only usable in the current file.
macro_rules! decl_extrinsic_test {
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

decl_extrinsic_test! {
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
}

