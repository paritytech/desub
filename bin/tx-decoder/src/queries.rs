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

use sqlx::{PgConnection, FromRow};
use futures::{Stream, TryStreamExt};
use anyhow::Error;
use serde::{Serialize, Deserialize};

/// Struct modeling data returned from database when querying for a block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct BlockModel {
	pub id: i32,
	pub parent_hash: Vec<u8>,
	pub hash: Vec<u8>,
	pub block_num: i32,
	pub state_root: Vec<u8>,
	pub extrinsics_root: Vec<u8>,
	pub digest: Vec<u8>,
	pub ext: Vec<u8>,
	pub spec: i32,
}
// just returns all blocks in the database of a specific spec as a stream
pub fn blocks<'a>(conn: &'a mut PgConnection, spec: i32) -> impl Stream<Item = Result<BlockModel, Error>> + 'a {
	sqlx::query_as!(BlockModel, "SELECT * FROM blocks WHERE spec = $1", spec)
		.fetch(conn)
		.map_err(Into::into)
}

/// get a single block
pub async fn single_block(conn: &mut PgConnection, number: i32) -> Result<BlockModel, Error> {
	sqlx::query_as!(BlockModel, "SELECT * FROM blocks WHERE block_num = $1", number)
		.fetch_one(conn)
		.await
		.map_err(Into::into)
}

#[derive(FromRow)]
struct Meta {
	pub meta: Vec<u8>
}

pub async fn metadata(conn: &mut PgConnection, spec: i32) -> Result<Vec<u8>, Error> {
	sqlx::query_as!(Meta, "SELECT meta FROM metadata WHERE version = $1", spec)
		.fetch_one(conn)
		.await
		.map_err(Into::into)
		.map(|m| m.meta)
}

#[derive(FromRow)]
struct MetaAndVersion {
	pub meta: Vec<u8>,
	pub version: i32,
}

pub async fn metadata_by_block(conn: &mut PgConnection, number: u32) -> Result<(Vec<u8>, i32), Error> {
	sqlx::query_as!(MetaAndVersion,
		"SELECT meta, version FROM (
			SELECT block_num, blocks.spec, metadata.version, metadata.meta FROM blocks, metadata
			WHERE
				block_num = $1
			AND
				blocks.spec = metadata.version
		) as z;", number as i32)
	.fetch_one(conn)
	.await
	.map_err(Into::into)
	.map(|m| (m.meta, m.version))
}


