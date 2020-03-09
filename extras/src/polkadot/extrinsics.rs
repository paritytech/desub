// Copyright 2019 Parity Technologies (UK) Ltd.
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

// TODO: open this file or pass it via CLI to reduce binary size
use serde::{Deserialize, Serialize};
use super::ModuleTypes;
use crate::error::Error;
pub const EXTRINSICS: &'static str = include_str!("./dot_definitions/extrinsics.json");

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Types {
    /// the spec these types are relevant for
    #[serde(rename = "minmax")]
    min_max: [Option<usize>; 2],
    /// types relevant to the spec
    types: ModuleTypes
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Extrinsics {
    kusama: Vec<Types>
}

impl Extrinsics {
    pub fn new(raw_json: &str) -> Result<Self, Error> {
        serde_json::from_str(raw_json).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_ext_definitions() {
        let extrinsics = Extrinsics::new(EXTRINSICS).unwrap();
        dbg!(extrinsics);
    }
}
