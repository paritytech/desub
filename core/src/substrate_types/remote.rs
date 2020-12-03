// Copyright 2019 Parity Technologies (UK) Ltd.
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

use primitives::crypto::AccountId32;
use serde::{Deserialize, Serialize};

use super::{Address, Conviction, Data, Vote};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Address")]
pub enum RemoteAddress {
    /// It's an account ID (pubkey).
    Id(AccountId32),
    /// It's an account index.
    Index(u32),
    /// It's some arbitrary raw bytes.
    Raw(Vec<u8>),
    /// It's a 32 byte representation.
    Address32([u8; 32]),
    /// It's a 20 byte representation.
    Address20([u8; 20]),
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vote")]
pub struct RemoteVote {
    pub aye: bool,
    #[serde(with = "RemoteConviction")]
    pub conviction: Conviction,
}

/// Either underlying data blob if it is at most 32 bytes, or a hash of it. If the data is greater
/// than 32-bytes then it will be truncated when encoding.
///
/// Can also be `None`.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Data")]
pub enum RemoteData {
    /// No data here.
    None,
    /// The data is stored directly.
    Raw(Vec<u8>),
    /// Only the Blake2 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    BlakeTwo256([u8; 32]),
    /// Only the SHA2-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Sha256([u8; 32]),
    /// Only the Keccak-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Keccak256([u8; 32]),
    /// Only the SHA3-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    ShaThree256([u8; 32]),
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
