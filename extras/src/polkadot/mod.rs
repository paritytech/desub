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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use core::{Decodable, RustTypeMarker, TypeDetective};

use self::overrides::Overrides;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct PolkadotTypes {
    pub mods: Modules,
    pub overrides: Overrides,
}

impl PolkadotTypes {
    pub fn new() -> Result<Self, Error> {
        Ok(PolkadotTypes {
            mods: definitions::definitions(definitions::DEFS)?,
            overrides: Overrides::new(overrides::OVERRIDES)?,
        })
    }

    /// get a types definition
    /// goes through override check
    /// does not resolve TypePointers
    pub fn get(
        &self,
        module: &str,
        ty: &str,
        spec: usize,
        chain: &str,
    ) -> Option<&RustTypeMarker> {
        if let Some(t) = self.check_overrides(module, ty, spec, chain) {
            Some(&t)
        } else {
            self.mods.modules.get(module)?.types.get(ty)
        }
    }

    /// check if an override exists for a given type
    ///
    /// if it does, return the types/type pointer
    pub fn check_overrides(
        &self,
        module: &str,
        ty: &str,
        spec: usize,
        chain: &str,
    ) -> Option<&RustTypeMarker> {
        // check if the type is a module override first
        if let Some(m) = self.overrides.get_module_types(module) {
            if let Some(ty) = m.get(ty) {
                return Some(ty);
            }
        }

        // if it isn't in modules, chain types is next
        self.overrides.get_chain_types(chain, spec)?.get(ty)
    }

    /// try to resolve a type pointer
    pub fn resolve(ty: RustTypeMarker) -> RustTypeMarker {
        unimplemented!()
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

impl ModuleTypes {
    /// alias to HashMap::get(&self, key: K)
    pub fn get(&self, ty: &str) -> Option<&RustTypeMarker> {
        self.types.get(ty)
    }
}

impl TypeDetective for PolkadotTypes {
    type Error = Error;

    fn get(
        &self,
        module: &str,
        ty: &str,
        spec: usize,
        chain: &str,
    ) -> Result<&dyn Decodable, Error> {
        let module = module.to_ascii_lowercase();
        let chain = module.to_ascii_lowercase();
        let decodable =
            self.get(&module, ty, spec, &chain)
                .ok_or(Error::NotFound(format!(
                    "{} in module {} for spec {} on chain {}",
                    ty, module, spec, chain
                )))?;
        Ok(decodable as &dyn Decodable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{RustEnum, StructField};

    #[test]
    fn should_get_type_from_module() -> Result<(), Error> {
        let post_1031_dispatch_error = RustTypeMarker::Enum(RustEnum::Struct(vec![
            StructField { name: "Other".to_string(),        ty: RustTypeMarker::Null},
            StructField { name: "CannotLookup".to_string(), ty: RustTypeMarker::Null},
            StructField { name: "BadOrigin".to_string(),    ty: RustTypeMarker::Null},
            StructField { name: "Module".to_string(),
                         ty: RustTypeMarker::TypePointer("DispatchErrorModule".to_string())}
        ]));
        let types = PolkadotTypes::new()?;
        let t = types
            .get("system", "DispatchError", 1040, "kusama")
            .unwrap();
        assert_eq!(t, &post_1031_dispatch_error);
        Ok(())
    }

    #[test]
    fn should_get_duplicated_types() -> Result<(), Error> {
        let types = PolkadotTypes::new()?;
        let t = types
            .get("contracts", "StorageKey", 1040, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("ContractStorageKey".to_string())
        );
        Ok(())
    }

    #[test]
    fn should_adhere_to_spec() -> Result<(), Error> {
        let pre_1019_balance_lock = RustTypeMarker::Struct(vec![
            StructField {
                name: "id".to_string(),
                ty: RustTypeMarker::TypePointer("LockIdentifier".to_string()),
            },
            StructField {
                name: "amount".to_string(),
                ty: RustTypeMarker::TypePointer("Balance".to_string()),
            },
            StructField {
                name: "reasons".to_string(),
                ty: RustTypeMarker::TypePointer("Reasons".to_string()),
            },
        ]);
        let types = PolkadotTypes::new()?;
        let t = types
            .get("balances", "BalanceLock", 1000, "kusama")
            .unwrap();
        assert_eq!(t, &pre_1019_balance_lock);
        let t = types
            .get("balances", "BalanceLock", 1018, "kusama")
            .unwrap();
        assert_eq!(t, &pre_1019_balance_lock);
        let t = types
            .get("balances", "BalanceLock", 1031, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("BalanceLockTo212".to_string())
        );
        let t = types
            .get("balances", "BalanceLock", 1019, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("BalanceLockTo212".to_string())
        );
        let t = types
            .get("balances", "BalanceLock", 1032, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("BalanceLockTo212".to_string())
        );
        let t = types
            .get("balances", "BalanceLock", 1042, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("BalanceLockTo212".to_string())
        );
        let t = types
            .get("balances", "BalanceLock", 9999, "kusama")
            .unwrap();
        assert_eq!(
            t,
            &RustTypeMarker::TypePointer("BalanceLockTo212".to_string())
        );
        Ok(())
    }
}
