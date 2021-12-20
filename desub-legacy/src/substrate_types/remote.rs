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

use serde::{Deserialize, Serialize};

use super::pallet_democracy::{Conviction, Vote};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vote")]
pub struct RemoteVote {
	pub aye: bool,
	#[serde(with = "RemoteConviction")]
	pub conviction: Conviction,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Conviction")]
pub enum RemoteConviction {
	/// 0.1x votes, unlocked.
	None,
	/// 1x votes, locked for an enactment period following a successful vote.
	Locked1x,
	/// 2x votes, locked for 2x enactment periods following a successful vote.
	Locked2x,
	/// 3x votes, locked for 4x...
	Locked3x,
	/// 4x votes, locked for 8x...
	Locked4x,
	/// 5x votes, locked for 16x...
	Locked5x,
	/// 6x votes, locked for 32x...
	Locked6x,
}
