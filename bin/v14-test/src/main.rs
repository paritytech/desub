use core::{ Metadata, Decoder };
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
	/// SCALE encoded V14 metadata blob
	#[structopt(parse(from_os_str))]
	metadata: PathBuf,
	/// Extrinsic hash in the form 0x1a2b3c
	extrinsic: String,
}

fn main() -> Result<(), anyhow::Error> {
	let opts = Opts::from_args();

	let metadata_bytes = std::fs::read(opts.metadata)?;

	let meta = Metadata::from_bytes(&metadata_bytes)?;
	let decoder = Decoder::with_metadata(meta);

	let ext = match opts.extrinsic.strip_prefix("0x") {
		Some(ext) => ext,
		None => anyhow::bail!("Extrinsic should start with 0x")
	};

	let bytes = match hex::decode(ext) {
		Ok(bytes) => bytes,
		Err(e) => anyhow::bail!("Cannot decode hex string into bytes: {}", e)
	};

	let decoded = match decoder.decode_extrinsic(&bytes) {
		Ok(decoded) => decoded,
		Err(e) => anyhow::bail!("Cannot decode extrinsic: {}", e)
	};

	println!("{:?}", decoded);
	Ok(())
}
