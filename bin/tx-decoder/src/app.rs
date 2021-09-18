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
use desub_extras::runtimes;

use sqlx::postgres::{PgPoolOptions, PgPool, PgConnection};
use futures::StreamExt;
use anyhow::Error;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use argh::FromArgs;
use async_std::task;
use parking_lot::Mutex;

use std::{convert::TryInto, borrow::Cow, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

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
	pub verbose: bool,
	#[argh(switch, short = 'p')]
	/// show decoding progress.
	pub progress: bool,
}

struct AppState<'a> {
	app: &'a App,
	decoder: &'a Mutex<Decoder>,
	pool: &'a PgPool,
	pb: Option<&'a ProgressBar>
}

impl<'a> AppState<'a> {
	fn new(
		app: &'a App,
		decoder: &'a Mutex<Decoder>,
		pool: &'a PgPool,
		pb: Option<&'a ProgressBar>
	) -> Self {
		Self { app, decoder, pool, pb }
	}

	fn print_blocks(&self, versions: Vec<u32>) -> Result<(usize, usize), Error> {
		let error_count = AtomicUsize::new(0);
		let length = AtomicUsize::new(0);
		versions.into_par_iter().try_for_each(|version| {
			let mut conn = task::block_on(self.pool.acquire())?;
			let previous = {
				let mut decoder = self.decoder.lock();
				task::block_on(register_metadata(&mut conn, &mut decoder, version.try_into()?))?
			};
			let (err, len) = task::block_on(self.print_blocks_by_spec(&mut conn, version as i32, previous as i32))?;
			error_count.fetch_add(err, Ordering::SeqCst);
			length.fetch_add(len, Ordering::SeqCst);
			Ok::<_, Error>(())
		})?;
		Ok((error_count.into_inner(), length.into_inner()))
	}

	async fn print_blocks_by_spec(&self, conn: &mut PgConnection, version: i32, previous: i32) -> Result<(usize, usize), Error> {
		let mut blocks = blocks_by_spec(conn, version);
		let mut len = 0;
		let mut error_count = 0;
		let decoder = self.decoder.lock();
		while let Some(Ok(block)) = blocks.next().await {
			let spec = if is_upgrade_block(&self.app.network, block.block_num.try_into()?) { previous } else { version };
			if Self::decode(&decoder, block, spec).is_err() {
				error_count +=1;
			}
			len += 1;
			self.pb.map(|p| p.inc(1));
		}
		Ok((error_count, len))
	}

	fn decode(decoder: &Decoder, block: BlockModel, spec: SpecVersion) -> Result<(), Error> {
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

	fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
		self.pb.map(|p| p.set_message(msg));
	}

	fn set_length(&self, len: u64) {
		self.pb.map(|p| p.set_length(len));
	}

	fn finish_and_clear(&self) {
		self.pb.map(|p| p.finish_and_clear());
	}
}

pub async fn app(app: App) -> Result<(), Error> {
	let pool = PgPoolOptions::new()
		.max_connections(num_cpus::get() as u32)
		.connect(&app.database_url)
		.await?;

	let mut conn = pool.acquire().await?;

	let types = desub_extras::TypeResolver::default();
	let decoder = Arc::new(Mutex::new(Decoder::new(types, app.network.clone())));

	if let Some(block) = &app.block {
		let (_, version) = metadata_by_block(&mut conn, *block).await?;
		let mut decoder = decoder.lock();
		let previous = register_metadata(&mut conn, &mut decoder, version).await?;
		let block = single_block(&mut conn, *block as i32).await?;
		let version = if is_upgrade_block(&app.network, block.block_num.try_into()?) { previous } else { version as u32 };
		AppState::decode(&decoder, block, version.try_into()?)?;
	}

	let pb = if app.progress {
		Some(construct_progress_bar(1000))
	} else { None };

	let state = AppState::new(&app, &decoder, &pool, pb.as_ref());

	if let Some(spec) = app.spec {
		let now = std::time::Instant::now();
		let count = blocks_in_spec(&mut conn, spec).await?;
		state.set_message(format!("decoding blocks for spec {}", spec));
		state.set_length(count as u64);
		let (error_count, len) = state.print_blocks(vec![spec.try_into()?])?;
		state.finish_and_clear();
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), len, error_count);
	}

	if let Some(to) = app.to {
		let spec_versions = spec_versions_upto(&mut conn, to).await?;
		let now = std::time::Instant::now();
		let count = count_upto_spec(&mut conn, to).await?;
		state.set_message(format!("decoding blocks up to spec {}", to));
		state.set_length(count as u64);
		let (error_count, length) = state.print_blocks(spec_versions)?;
		state.finish_and_clear();
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), length, error_count);
	}

	if app.all {
		let spec_versions = spec_versions(&mut conn).await?;
		let now = std::time::Instant::now();
		let count = total_block_count(&mut conn).await?;
		pb.as_ref().map(|p| p.set_message("decoding all blocks"));
		pb.as_ref().map(|p| p.set_length(count as u64));
		let (error_count, length) = state.print_blocks(spec_versions)?;
		state.finish_and_clear();
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), length, error_count);
	}
	Ok(())
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

fn is_upgrade_block(chain: &Chain, number: u64) -> bool {
	match chain {
		Chain::Kusama => runtimes::kusama_upgrade_block(&number).is_some(),
		Chain::Polkadot => runtimes::polkadot_upgrade_block(&number).is_some(),
		Chain::Westend => runtimes::westend_upgrade_block(&number).is_some(),
		_ => false
	}
}

/// Register the metadata with Decoder
/// returns the previous spec version.
async fn register_metadata(conn: &mut PgConnection, decoder: &mut Decoder, version: SpecVersion) -> Result<u32, Error> {
	let (past, present) = past_and_present_version(conn, version).await?;

	if !decoder.has_version(present) {
		let meta = metadata(conn, present.try_into()?).await?;
		decoder.register_version(present, meta);
	}

	if !decoder.has_version(past) {
		let meta = metadata(conn, past.try_into()?).await?;
		decoder.register_version(past, meta);
	}

	Ok(past)
}
