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

//! Deserializes Polkadot Type Definitions into general struct defined in `core/lib.rs`
// TODO: all type resolution should be refactored
// this is a very confusing and inefficient warranty
// but it works (for the most part)
mod definitions;
mod extrinsics;
mod overrides;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use core::{regex, Decodable, RustTypeMarker, TypeDetective};

use self::extrinsics::Extrinsics;
use self::overrides::Overrides;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct PolkadotTypes {
    pub mods: Modules,
    pub overrides: Overrides,
    pub extrinsics: Extrinsics,
}

impl PolkadotTypes {
    pub fn new() -> Result<Self, Error> {
        Ok(PolkadotTypes {
            mods: definitions::definitions(definitions::DEFS)?,
            overrides: Overrides::new(overrides::OVERRIDES)?,
            extrinsics: Extrinsics::new(extrinsics::EXTRINSICS)?,
        })
    }

    /// get a types definition
    /// goes through override check
    pub fn get(
        &self,
        module: &str,
        ty: &str,
        spec: u32,
        chain: &str,
    ) -> Option<&RustTypeMarker> {
        log::debug!("Getting Type: {}, module: {}, spec: {}", ty, module, spec);

        let ty = if let Some(un_prefixed) = regex::remove_prefix(ty) {
            un_prefixed
        } else {
            ty.to_string()
        };
        log::debug!("Getting possibly de-prefixed type: {}", ty);
        if let Some(t) = self.check_overrides(module, ty.as_str(), spec, chain) {
            log::debug!("Resolving to Override");
            Some(&t)
        } else if let Some(t) = self.check_extrinsics(ty.as_str(), spec, chain) {
            log::debug!("Resolving to Extrinsic Type");
            Some(&t)
        } else {
            log::debug!("Resolving to Type Pointer");
            self.resolve_helper(module, &ty)
        }
    }

    /// check if an override exists for a given type
    ///
    /// if it does, return the types/type pointer
    pub fn check_overrides(
        &self,
        module: &str,
        ty: &str,
        spec: u32,
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

    pub fn check_extrinsics(
        &self,
        ty: &str,
        spec: u32,
        chain: &str,
    ) -> Option<&RustTypeMarker> {
        if let Some(m) = self.extrinsics.get_chain_types(chain, spec) {
            if let Some(ty) = m.get(ty) {
                return Some(ty);
            }
        }
        None
    }

    /// Try to resolve a type pointer
    fn resolve_helper(&self, module: &str, ty_pointer: &str) -> Option<&RustTypeMarker> {
        if self.mods.modules.get(module).is_none() {
            self.mods.modules.get("runtime")?.types.get(ty_pointer)
        } else if let Some(t) = self.mods.modules.get(module)?.types.get(ty_pointer) {
            Some(t)
        } else if let Some(t) = self.mods.modules.get("runtime")?.types.get(ty_pointer) {
            Some(t)
        } else if let Some(t) = self.check_other_modules(ty_pointer) {
            Some(t)
        } else {
            None
        }
    }

    fn check_other_modules(&self, ty_pointer: &str) -> Option<&RustTypeMarker> {
        for m in self.mods.modules.values() {
            for (n, t) in m.types.iter() {
                if n == ty_pointer {
                    return Some(t);
                }
            }
        }
        None
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
    fn get(
        &self,
        module: &str,
        ty: &str,
        spec: u32,
        chain: &str,
    ) -> Option<&dyn Decodable> {
        let module = module.to_ascii_lowercase();
        let chain = chain.to_ascii_lowercase();
        let decodable = self.get(&module, ty, spec, &chain)?;
        Some(decodable as &dyn Decodable)
    }

    fn get_extrinsic_ty(
        &self,
        spec: u32,
        chain: &str,
        ty: &str,
    ) -> Option<&dyn Decodable> {
        let ty = self.check_extrinsics(ty, spec, chain);

        let ty = if let Some(t) = ty {
            match t {
                RustTypeMarker::TypePointer(t) => self.resolve_helper("runtime", t),
                t => Some(t),
            }
        } else {
            None
        };
        ty.map(|t| t as &dyn Decodable)
    }

    fn resolve(&self, module: &str, ty: &RustTypeMarker) -> Option<&RustTypeMarker> {
        let ty = match ty {
            RustTypeMarker::TypePointer(v) => {
                if let Some(un_prefixed) = regex::remove_prefix(v.as_str()) {
                    RustTypeMarker::TypePointer(un_prefixed)
                } else {
                    RustTypeMarker::TypePointer(v.clone())
                }
            }
            v => v.clone(),
        };
        self.resolve_helper(module, ty.as_type_pointer()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{EnumField, StructField, StructUnitOrTuple};

    #[test]
    fn should_get_type_from_module() -> Result<(), Error> {
        let post_1031_dispatch_error = RustTypeMarker::Enum(vec![
            EnumField::new(
                Some("Other".into()),
                StructUnitOrTuple::Tuple(RustTypeMarker::Null),
            ),
            EnumField::new(
                Some("CannotLookup".into()),
                StructUnitOrTuple::Tuple(RustTypeMarker::Null),
            ),
            EnumField::new(
                Some("BadOrigin".into()),
                StructUnitOrTuple::Tuple(RustTypeMarker::Null),
            ),
            EnumField::new(
                Some("Module".into()),
                StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                    "DispatchErrorModule".to_string(),
                )),
            ),
        ]);
        let types = PolkadotTypes::new()?;
        let t = types
            .get("system", "DispatchError", 1040, "kusama")
            .unwrap();
        assert_eq!(t, &post_1031_dispatch_error);
        Ok(())
    }

    #[test]
    fn should_resolve_a_type() -> Result<(), Error> {
        let t_pointer = RustTypeMarker::TypePointer("BalanceLockTo212".to_string());
        let correct = RustTypeMarker::Struct(vec![
            StructField {
                name: "id".to_string(),
                ty: RustTypeMarker::TypePointer("LockIdentifier".to_string()),
            },
            StructField {
                name: "amount".to_string(),
                ty: RustTypeMarker::TypePointer("Balance".to_string()),
            },
            StructField {
                name: "until".to_string(),
                ty: RustTypeMarker::TypePointer("BlockNumber".to_string()),
            },
            StructField {
                name: "reasons".to_string(),
                ty: RustTypeMarker::TypePointer("WithdrawReasons".to_string()),
            },
        ]);
        let types = PolkadotTypes::new()?;
        let resolved = types.resolve("balances", &t_pointer).unwrap();
        assert_eq!(&correct, resolved);
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
        Ok(())
    }
}
