// Copyright 2021 Parity Technologies (UK) Ltd.
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

use crate::queries::*;

use desub::decoder::{Decoder, Chain};

use sqlx::postgres::{PgPoolOptions, PgPool, PgConnection};
use futures::StreamExt;
use anyhow::Error;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use argh::FromArgs;
use async_std::task;
use parking_lot::Mutex;

use std::{convert::TryInto, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

type SpecVersion = i32;

#[derive(FromArgs, PartialEq, Debug)]
/// Decode Extrinsics And Storage from Substrate Archive
pub struct App {
	#[argh(option, default = "default_database_url()", short = 'd')]
	/// database url containing encoded information.
	database_url: String,
	#[argh(option, default = "Chain::Polkadot", short = 'n')]
	/// chain
	network: Chain,
	#[argh(option, short = 's')]
	/// decode blocks only in this spec version.
	spec: Option<i32>,
	#[argh(option, short = 'b')]
	/// decode only a specific block.
	block: Option<u32>,
	#[argh(switch, short = 'a')]
	/// decode all blocks
	all: bool,
	#[argh(option, short = 'u')]
	///	decode all blocks up to a spec version.
	to: Option<i32>,
	#[argh(switch, short = 'v')]
	/// extra information about the programs execution.
	pub verbose: bool
}

pub async fn app(app: App) -> Result<(), Error> {
	let pool = PgPoolOptions::new()
		.connect(&app.database_url)
		.await?;

	let mut conn = pool.acquire().await?;

	let types = desub_extras::TypeResolver::default();
	let decoder = Arc::new(Mutex::new(Decoder::new(types, Chain::Kusama)));

	if let Some(block) = app.block {
		let (meta, spec) = metadata_by_block(&mut conn, block).await?;
		let mut decoder = decoder.lock();
		decoder.register_version(spec.try_into()?, meta);
		let block = single_block(&mut conn, block as i32).await?;
		decode(&decoder, spec, block)?;
	}

	if let Some(spec) = app.spec {
		let metadata = metadata(&mut conn, spec).await?;
		let mut decoder = decoder.lock();
		decoder.register_version(spec.try_into()?, metadata);
		let now = std::time::Instant::now();
		let count = blocks_in_spec(&mut conn, spec).await?;
		let pb = construct_progress_bar(count as usize);
		pb.set_message(format!("decoding blocks for spec {}", spec));
		let (error_count, len) = print_blocks_by_spec(&mut conn, &decoder, spec, &pb).await;
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), len, error_count);
	}

	if let Some(to) = app.to {
		let spec_versions = spec_versions_upto(&mut conn, to).await?;
		let now = std::time::Instant::now();
		let count = count_upto_spec(&mut conn, to).await?;
		let pb = construct_progress_bar(count as usize);
		pb.set_message(format!("decoding blocks up to spec {}", to));
		let (error_count, length) = print_blocks(&pool, &decoder, spec_versions, pb)?;
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), length, error_count);
	}

	if app.all {
		let spec_versions = spec_versions(&mut conn).await?;
		let now = std::time::Instant::now();
		let count = total_block_count(&mut conn).await?;
		let pb = construct_progress_bar(count as usize);
		pb.set_message("decoding all blocks");
		let (error_count, length) = print_blocks(&pool, &decoder, spec_versions, pb)?;
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), length, error_count);
	}
	Ok(())
}

fn print_blocks(pool: &PgPool, decoder: &Mutex<Decoder>, spec_versions: Vec<u32>, pb: ProgressBar) -> Result<(usize, usize), Error> {
	let error_count = AtomicUsize::new(0);
	let length = AtomicUsize::new(0);
	spec_versions.into_par_iter().try_for_each(|version| {
		let mut conn = task::block_on(pool.acquire())?;
		let metadata = task::block_on(metadata(&mut conn, version as i32))?;
		let mut decoder = decoder.lock();
		decoder.register_version(version as u32, metadata);
		let (err, len) = task::block_on(print_blocks_by_spec(&mut conn, &decoder, version as i32, &pb));
		error_count.fetch_add(err, Ordering::SeqCst);
		length.fetch_add(len, Ordering::SeqCst);
		Ok::<_, Error>(())
	})?;
	Ok((error_count.into_inner(), length.into_inner()))
}

async fn print_blocks_by_spec(
	conn: &mut PgConnection,
	decoder: &Decoder,
	spec: SpecVersion,
	pb: &ProgressBar,
) -> (usize, usize) {

	let mut blocks = blocks_by_spec(conn, spec);
	let mut len = 0;
	let mut error_count = 0;
	while let Some(Ok(block)) = blocks.next().await {
		if decode(&decoder, spec, block).is_err() {
			error_count +=1;
		}
		len += 1;
		pb.inc(1);
	}
	(error_count, len)
}

fn decode(decoder: &Decoder, spec: SpecVersion, block: BlockModel) -> Result<(), Error> {
	log::trace!("-<<-<<-<<-<<-<<-<<-<<-<<-<< Decoding block {}, ext length {}", block.block_num, block.ext.len());
	match decoder.decode_extrinsics(spec.try_into()?, block.ext.as_slice()) {
		Err(e) => {
			log::error!("Failed to decode block {} due to {}", block.block_num, e);
			Err(e.into())
		},
		Ok(d) => {
			log::info!("Block {} Decoded Succesfully. {}", block.block_num, serde_json::to_string_pretty(&d)?);
			Ok(())
		}
	}
}

fn default_database_url() -> String {
	"postgres://postgres@localhost:5432/postgres".to_string()
}

fn construct_progress_bar(count: usize) -> ProgressBar {
	let bar = ProgressBar::new(count as u64);
	bar.set_style(
		ProgressStyle::default_bar()
			.template("{spinner:.cyan} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% ({eta}) ({pos}/{len})")
			.progress_chars("#>-"),
	);
	bar
}
