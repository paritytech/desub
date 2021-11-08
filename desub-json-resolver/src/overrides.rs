// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use crate::{ModuleTypes, Result, TypeRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types that are given priority over those defined in [Modules](struct.Modules.html)
#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Clone)]
pub struct Overrides {
	/// Type Overrides for modules (where duplication between modules exist)
	#[serde(rename = "TYPES_MODULES")]
	types_modules: HashMap<String, ModuleTypes>,
	/// these are override types for Polkadot & Kusama chains
	/// NOTE The SessionKeys definition for Polkadot and Substrate (OpaqueKeys
	/// implementation) are different. Detect Polkadot and inject the `Keys`
	/// definition as applicable. (4 keys in substrate vs 5 in Polkadot/CC3).
	#[serde(rename = "TYPES_SPEC")]
	// chain(e.g kusama/polkadot) -> Vector of overrides
	types_spec: HashMap<String, Vec<TypeRange>>,
}

impl Overrides {
	/// Construct overrides from JSON
	pub fn new(raw_json: &str) -> Result<Self> {
		serde_json::from_str(raw_json).map_err(Into::into)
	}

	/// get a module types based upon spec
	pub fn get_chain_types(&self, chain: &str, spec: u32) -> Option<&ModuleTypes> {
		self.types_spec.get(chain)?.iter().find(|f| crate::is_in_range(spec, f)).map(|o| &o.types)
	}

	/// get types for a substrate module
	pub fn get_module_types(&self, module: &str) -> Option<&ModuleTypes> {
		self.types_modules.get(module)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

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

		let single_override: TypeRange = serde_json::from_str(json).unwrap();
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

		let types_spec: HashMap<String, Vec<TypeRange>> = serde_json::from_str(json).unwrap();
		dbg!(types_spec);
	}
}
