
use std::path::PathBuf;
use structopt::StructOpt;
use core::Metadata;

#[derive(Debug, StructOpt)]
struct Opts {
	#[structopt(parse(from_os_str))]
	metadata: PathBuf
}

fn main() -> Result<(), anyhow::Error> {

	let opts = Opts::from_args();

	let metadata_bytes = std::fs::read(opts.metadata)?;

	Metadata::from_bytes(&metadata_bytes)?;

	Ok(())
}
