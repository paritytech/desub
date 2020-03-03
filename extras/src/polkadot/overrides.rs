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

use super::ModuleTypes;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: open this file or pass it via CLI to reduce binary size
pub const OVERRIDES: &str = include_str!("./dot_definitions/overrides.json");

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct SingleOverride {
    /// the spec these overrides are relevant for
    #[serde(rename = "minmax")]
    min_max: [Option<usize>; 2],
    /// types that are being overriden
    /// points to the types that should be used instead in definitions.json
    types: ModuleTypes,
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Overrides {
    /// Type Overrides for modules (where duplication between modules exist)
    #[serde(rename = "TYPES_MODULES")]
    types_modules: HashMap<String, ModuleTypes>,
    /// Overrides based on metadata versions
    /// this is for support of old testnets (Alexander)
    // this can be safely ignored for the most part
    #[serde(rename = "TYPES_META")]
    types_meta: Vec<SingleOverride>,
    /// these are override types for Polkadot & Kusama chains
    /// NOTE The SessionKeys definition for Polkadot and Substrate (OpaqueKeys
    /// implementation) are different. Detect Polkadot and inject the `Keys`
    /// definition as applicable. (4 keys in substrate vs 5 in Polkadot/CC3).
    #[serde(rename = "TYPES_SPEC")]
    // chain(e.g kusama/polkadot) -> Vector of overrides
    types_spec: HashMap<String, Vec<SingleOverride>>,
}

impl Overrides {
    pub fn new(raw_json: &str) -> Result<Overrides, Error> {
        let types: Overrides = serde_json::from_str(raw_json)?;
        Ok(types)
    }

    /// get a module types based upon spec
    pub fn get_chain_types(&self, chain: &str, spec: u32) -> Option<&ModuleTypes> {
        self.types_spec
            .get(chain)?
            .iter()
            .find(|f| Self::is_in_range(spec, f))
            .map(|o| &o.types)
    }

    pub fn get_module_types(&self, module: &str) -> Option<&ModuleTypes> {
        self.types_modules.get(module)
    }

    fn is_in_range(spec: u32, over_ride: &SingleOverride) -> bool {
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
    fn should_deserialize_overrides() {
        let overrides = Overrides::new(OVERRIDES).unwrap();
        dbg!(overrides);
    }

    #[test]
    fn should_deserialize_into_single_override() {
        let json = r#"
        {
            "minmax": [
                1020,
                1031
            ],
            "types": {
                "BalanceLock": "BalanceLockTo212",
                "DispatchError": "DispatchErrorTo198",
                "Keys": "SessionKeys5",
                "SlashingSpans": "SlashingSpansTo204"
            }
        }
        "#;

        let single_override: SingleOverride = serde_json::from_str(json).unwrap();
        dbg!(single_override);
    }

    #[test]
    fn should_deserialize_into_spec() {
        let json = r#"
        {
        "kusama": [
            {
                "minmax": [
                1019,
                1031
                ],
                "types": {
                "BalanceLock": "BalanceLockTo212",
                "DispatchError": "DispatchErrorTo198",
                "Keys": "SessionKeys5",
                "SlashingSpans": "SlashingSpansTo204"
                }
            },
            {
                "minmax": [
                1032,
                1042
                ],
                "types": {
                "BalanceLock": "BalanceLockTo212",
                "Keys": "SessionKeys5",
                "SlashingSpans": "SlashingSpansTo204"
                }
            },
            {
                "minmax": [
                1043,
                null
                ],
                "types": {
                "BalanceLock": "BalanceLockTo212",
                "Keys": "SessionKeys5"
                }
            }
            ],
            "polkadot": [
            {
                "minmax": [
                1000,
                null
                ],
                "types": {
                "Keys": "SessionKeys5"
                }
            }
            ]
        }
        "#;

        let types_spec: HashMap<String, Vec<SingleOverride>> =
            serde_json::from_str(json).unwrap();
        dbg!(types_spec);
    }

    #[test]
    fn should_deserialize_types_meta() {
        let json = r#"
        [
            {
            "minmax": [
                0,
                4
            ],
            "types": {
                "BlockNumber": "u64",
                "Index": "u64",
                "EventRecord": "EventRecordTo76",
                "ValidatorPrefs": "ValidatorPrefsTo145"
            }
            }
        ]
        "#;
        let types_meta: Vec<SingleOverride> = serde_json::from_str(json).unwrap();
        dbg!(types_meta);
    }
}
