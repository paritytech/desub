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

use core_v14::{
	decoder::DecodeError,
	metadata::MetadataError
};
use desub_legacy::{
	Error as LegacyError,
	decoder::metadata::Error as LegacyMetadataError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	V14(#[from] DecodeError),
	#[error(transparent)]
	Legacy(#[from] LegacyError),
	#[error(transparent)]
	Codec(#[from] codec::Error),
	#[error(transparent)]
	MetadataError(#[from] MetadataError),
	#[error(transparent)]
	LegacyMetadataError(#[from] LegacyMetadataError),
}
