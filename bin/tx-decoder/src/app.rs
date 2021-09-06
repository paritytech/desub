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

use sqlx::postgres::PgPoolOptions;
use futures::StreamExt;
use anyhow::Error;
use std::convert::TryInto;
use argh::FromArgs;

type SpecVersion = i32;

#[derive(FromArgs, PartialEq, Debug)]
/// Decode Extrinsics And Storage from Substrate Archive
struct App {
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
	block: Option<u32>
}

pub async fn app() -> Result<(), Error> {
	let app: App = argh::from_env();
	let pool = PgPoolOptions::new()
		.connect(&app.database_url)
		.await?;

	let mut conn = pool.acquire().await?;

	let types = desub_extras::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);

	if let Some(block) = app.block {
		let (meta, spec) = metadata_by_block(&mut conn, block).await?;
		decoder.register_version(spec.try_into()?, meta);
		let block = single_block(&mut conn, block as i32).await?;
		decode(&decoder, spec, block)?;
	}

	if let Some(spec) = app.spec {
		let metadata = metadata(&mut conn, spec).await?;
		decoder.register_version(spec.try_into()?, metadata);
		let mut blocks = blocks(&mut conn, spec);
		let mut len = 0;
		let mut error_count = 0;
		let now = std::time::Instant::now();
		while let Some(Ok(block)) = blocks.next().await {
			if let Err(_) = decode(&decoder, spec, block) {
				error_count +=1;
			}
			len += 1;
		}
		println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), len, error_count);
	}

	Ok(())
}

fn decode(decoder: &Decoder, spec: SpecVersion, block: BlockModel) -> Result<(), Error> {
	println!("-<<-<<-<<-<<-<<-<<-<<-<<-<< Decoding block {}, ext length {}", block.block_num, block.ext.len());
	match decoder.decode_extrinsics(spec.try_into()?, block.ext.as_slice()) {
		Err(e) => {
			log::error!("Failed to decode block {} due to {}", block.block_num, e);
			Err(e.into())
		},
		Ok(d) => {
			log::info!("Block {} Decoded Sucesfully. {}", block.block_num, serde_json::to_string_pretty(&d)?);
			Ok(())
		}
	}
}

fn default_database_url() -> String {
	"postgres://postgres@localhost:5432/postgres".to_string()
}


