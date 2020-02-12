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

use core::{RustTypeMarker, Decodable};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Default, Debug, PartialEq, Eq)]
pub struct PolkadotTypes {
    // module name -> Type Map of module
    pub modules: HashMap<String, ModuleTypes>,
}

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct ModuleTypes {
    // Type Name -> Type
    pub types: HashMap<String, RustTypeMarker>,
}

impl TypeDetective for PolkadotTypes {
    fn get(&self, module: &str, ty: &str) -> Result<RustTypeMarker, Error> {
       self.modules.get(module)
    }
}
