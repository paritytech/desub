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

use desub_current::{
	decoder::{DecodeError, Extrinsic},
	metadata::MetadataError,
};
use desub_legacy::{decoder::metadata::Error as LegacyMetadataError, Error as LegacyError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Decoding v14 failed {source}")]
	V14 {
		#[source]
		source: DecodeError,
		ext: Vec<Extrinsic<'static>>,
	},
	#[error(transparent)]
	Legacy(#[from] LegacyError),
	#[error(transparent)]
	Codec(#[from] codec::Error),
	#[error(transparent)]
	MetadataError(#[from] MetadataError),
	#[error(transparent)]
	LegacyMetadataError(#[from] LegacyMetadataError),
	#[error("Spec Version {0} not registered with decoder")]
	SpecVersionNotFound(u32),
}
