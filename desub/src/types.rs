// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.


use serde::Serialize;
use desub_current::decoder::Extrinsic;
use desub_legacy::decoder::GenericExtrinsic;


#[derive(Serialize, Debug, PartialEq)]
pub enum LegacyOrCurrent<L, C> {
	Legacy(L),
	Current(C)
}

pub type LegacyOrCurrentExtrinsic = LegacyOrCurrent<GenericExtrinsic, Extrinsic<'static>>;
