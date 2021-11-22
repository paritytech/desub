// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

//! A crate to decode extrinsics, signer payloads and storage keys for substrate nodes using V14+ metadata.
//! See [`decoder`] for more information.

mod type_id;

pub mod decoder;
pub mod metadata;
pub mod value;

pub use metadata::Metadata;
pub use value::{Value, ValueDef};

pub use type_id::TypeId;

/// A re-export of the [`scale_info`] crate, since we delegate much of the type inspection to it.
pub use scale_info;

/// A re-export of [`scale_info::Type`] as used throughout this library.
pub type Type = scale_info::Type<scale_info::form::PortableForm>;

/// The [`scale_info`] type ID as used throughout this library.
type ScaleInfoTypeId = scale_info::interner::UntrackedSymbol<std::any::TypeId>; // equivalent to: <scale_info::form::PortableForm as scale_info::form::Form>::Type;
