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
// TODO: So much of this code is redundant between extrinsics.rs and overrides.rs
// TODO: merge the similarities
use super::ModuleTypes;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub const EXTRINSICS: &'static str = include_str!("./dot_definitions/extrinsics.json");

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Types {
    /// the spec these types are relevant for
    #[serde(rename = "minmax")]
    min_max: [Option<usize>; 2],
    /// types relevant to the spec
    types: ModuleTypes,
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Extrinsics(HashMap<String, Vec<Types>>);

impl Extrinsics {
    pub fn new(raw_json: &str) -> Result<Self, Error> {
        serde_json::from_str(raw_json).map_err(Into::into)
    }

    pub fn get_chain_types(&self, chain: &str, spec: u32) -> Option<&ModuleTypes> {
        self.0
            .get(chain)?
            .iter()
            .find(|f| Self::is_in_range(spec, f))
            .map(|o| &o.types)
    }

    fn is_in_range(spec: u32, over_ride: &Types) -> bool {
        match over_ride.min_max {
            [Some(min), Some(max)] => (min..=max).contains(&(spec as usize)),
            [Some(min), None] => (spec as usize) > min,
            [None, Some(max)] => (spec as usize) < max,
            // presumably, this would be for null -> null,
            // so for every spec
            [None, None] => true,
        }
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

    #[test]
    fn should_get_types_from_json() {
        let extrinsics = Extrinsics::new(EXTRINSICS).unwrap();
        extrinsics.get_chain_types("kusama", 1031);
        extrinsics.get_chain_types("kusama", 1007);
        extrinsics.get_chain_types("kusama", 1006);
        let tys = extrinsics.get_chain_types("kusama", 1003);
        dbg!(tys);
    }
}
