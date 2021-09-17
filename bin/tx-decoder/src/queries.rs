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

#[derive(FromRow)]
struct Count {
	count: i64
}

/// returns how many blocks exist for a spec version.
pub async fn blocks_in_spec(conn: &mut PgConnection, spec: i32) -> Result<i64, Error> {
	Ok(sqlx::query_as::<_, Count>("SELECT COUNT(*) FROM blocks WHERE spec = $1")
		.bind(spec)
		.fetch_one(conn)
		.await?
		.count
	)
}

pub async fn total_block_count(conn: &mut PgConnection) -> Result<i64, Error> {
	Ok(sqlx::query_as::<_, Count>("SELECT COUNT(*) FROM blocks")
		.fetch_one(conn)
		.await?
		.count
	)
}

pub async fn count_upto_spec(conn: &mut PgConnection, spec: i32) -> Result<i64, Error> {
	Ok(sqlx::query_as::<_, Count>("SELECT COUNT(*) FROM blocks WHERE spec < $1")
		.bind(spec)
		.fetch_one(conn)
		.await?
		.count
	)
}

/// returns all blocks in the database of a specific spec as a stream
pub fn blocks_by_spec(conn: &mut PgConnection, spec: i32) -> impl Stream<Item = Result<BlockModel, Error>> + '_ {
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
struct Version {
	pub version: i32
}

pub async fn spec_versions(conn: &mut PgConnection) -> Result<Vec<u32>, Error> {
	sqlx::query_as!(Version, "SELECT version FROM metadata")
		.fetch_all(conn)
		.await
		.map_err(Into::into)
		.map(|r| r.iter().map(|v| v.version as u32).collect())

}

pub async fn spec_versions_upto(conn: &mut PgConnection, upto: i32) -> Result<Vec<u32>, Error> {
	sqlx::query_as!(Version, "SELECT version FROM metadata WHERE version < $1", upto)
		.fetch_all(conn)
		.await
		.map_err(Into::into)
		.map(|r| r.iter().map(|v| v.version as u32).collect())
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


