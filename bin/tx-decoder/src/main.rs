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

mod queries;

use queries::*;

use desub::decoder::{Decoder, Chain};

use sqlx::postgres::PgPoolOptions;
use futures::StreamExt;
use anyhow::Error;
use std::convert::TryInto;

const SPEC: i32 = 1030;

#[async_std::main]
async fn main() -> Result<(), Error> {
	pretty_env_logger::init();

	let pool = PgPoolOptions::new()
		.connect("postgres://postgres:123@localhost:6432/kusama-db")
		.await?;

	let mut conn = pool.acquire().await?;

	let types = desub_extras::TypeResolver::default();
	let mut decoder = Decoder::new(types, Chain::Kusama);
	let metadata = metadata(&mut conn, SPEC).await?;
	decoder.register_version(SPEC.try_into()?, metadata);

	let mut blocks = blocks(&mut conn, SPEC);
	let mut len = 0;
	let mut error_count = 0;
	let now = std::time::Instant::now();
	while let Some(Ok(block)) = blocks.next().await {
		match decoder.decode_extrinsic(SPEC.try_into()?, block.ext.as_slice()) {
			Err(e) => {
				error_count += 1;
				len += 1;
				log::error!("Failed to decode block {} due to {}", block.block_num, e);
			},
			Ok(_) => {
				len += 1;
			}
		}
	}

	println!("Took {:?} to decode {} blocks with {} errors.", now.elapsed(), len, error_count);

	Ok(())
}
