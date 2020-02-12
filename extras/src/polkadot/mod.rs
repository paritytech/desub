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
mod definitions;
mod overrides;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::error::Error;
use core::{RustTypeMarker, Decodable, TypeDetective};

use self::overrides::Overrides;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct PolkadotTypes {
    pub mods: Modules,
    pub overrides: Overrides
}

impl PolkadotTypes {
    pub fn new() -> Result<Self, Error> {
        Ok(PolkadotTypes {
            mods:  definitions::definitions(definitions::DEFS)?,
            overrides: overrides::overrides(overrides::OVERRIDES)?
        })
    }
}

#[derive(Serialize, Default, Debug, PartialEq, Eq)]
pub struct Modules {
    // module name -> Type Map of module
    pub modules: HashMap<String, ModuleTypes>,
}

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct ModuleTypes {
    // Type Name -> Type
    pub types: HashMap<String, RustTypeMarker>,
}

impl TypeDetective for PolkadotTypes {
    type Error = Error;
    fn get(&self, module: &str, ty: &str) -> Result<&dyn Decodable, Error> {
        Ok(self.mods.modules
           .get(module)
           .ok_or(Error::NotFound(format!("Module {}", module)))?
           .types
           .get(ty)
           .ok_or(Error::NotFound(format!("Type {}", ty)))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_type() {
    }
}
