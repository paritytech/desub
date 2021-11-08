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

//! Resolves types based on the JSON

use crate::{Extrinsics, Modules, Overrides, Result};
use desub_legacy::{regex, RustTypeMarker, TypeDetective};

#[cfg(feature = "default_definitions")]
mod default {
	pub const DEFINITIONS: &str = include_str!("./definitions/definitions.json");
	pub const OVERRIDES: &str = include_str!("./definitions/overrides.json");
	pub const EXTRINSICS: &str = include_str!("./definitions/extrinsics.json");
}

pub struct Builder {
	mods: Modules,
	overrides: Overrides,
	extrinsics: Extrinsics,
}

impl Builder {
	pub fn modules(mut self, modules: Modules) -> Self {
		self.mods = modules;
		self
	}

	pub fn modules_from_json(mut self, modules: &str) -> Result<Self> {
		self.mods = Modules::new(modules)?;
		Ok(self)
	}

	pub fn overrides(mut self, overrides: Overrides) -> Self {
		self.overrides = overrides;
		self
	}

	pub fn overrides_from_json(mut self, json: &str) -> Result<Self> {
		self.overrides = Overrides::new(json)?;
		Ok(self)
	}

	pub fn extrinsics(mut self, extrinsics: Extrinsics) -> Self {
		self.extrinsics = extrinsics;
		self
	}

	pub fn extrinsics_from_json(mut self, json: &str) -> Result<Self> {
		self.extrinsics = Extrinsics::new(json)?;
		Ok(self)
	}

	pub fn build(self) -> TypeResolver {
		TypeResolver { mods: self.mods, overrides: self.overrides, extrinsics: self.extrinsics }
	}
}

// we need a way to construct the builder when
// not using default features
#[cfg(not(feature = "default_definitions"))]
impl Builder {
	fn new(modules: Modules, extrinsics: Extrinsics, overrides: Overrides) -> Self {
		Self { mods, overrides, extrinsics }
	}
}

#[cfg(feature = "default_definitions")]
impl Default for Builder {
	fn default() -> Self {
		Self {
			mods: Modules::new(default::DEFINITIONS).expect("Included definitions should not panic"),
			overrides: Overrides::new(default::OVERRIDES).expect("Included overrides should not panic"),
			extrinsics: Extrinsics::new(default::EXTRINSICS).expect("Included extrinsics should not panic"),
		}
	}
}
#[cfg(feature = "default_definitions")]
impl Default for TypeResolver {
	fn default() -> Self {
		Builder::default().build()
	}
}

#[derive(Debug, Clone)]
pub struct TypeResolver {
	mods: Modules,
	overrides: Overrides,
	extrinsics: Extrinsics,
}

impl TypeResolver {
	/// Build the builder for `TypeResolver`
	pub fn builder() -> Builder {
		Builder::default()
	}

	/// Construct the TypeResolver from its parts
	pub fn new(modules: Modules, extrinsics: Extrinsics, overrides: Overrides) -> Self {
		Self { mods: modules, extrinsics, overrides }
	}

	/// Try to resolve a type
	/// First, tries to resolve from the overrides (overrides.json),
	/// then checks if extrinsics includes the type (extrinsics.json).
	/// If none of the above contains the type that we are looking for,
	/// we check if the modules includes it (definitions.json)
	///
	/// # Return
	/// returns None if the type cannot be resolved
	pub fn get(&self, chain: &str, spec: u32, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		log::trace!("Getting Type: {}, module: {}, spec: {}", ty, module, spec);

		if let Some(t) = self.check_overrides(module, ty, spec, chain) {
			log::trace!("Resolving to Override");
			Some(t)
		} else if let Some(t) = self.extrinsics.get(ty, spec, chain) {
			log::trace!("Resolving to Extrinsic Type");
			Some(t)
		} else {
			log::trace!("Resolving to Type Pointer");
			self.resolve_helper(module, ty)
		}
	}

	pub fn try_fallback(&self, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		self.mods.try_fallback(module, ty)
	}

	/// Get type for decoding an Extrinsic
	pub fn get_ext_ty(&self, chain: &str, spec: u32, ty: &str) -> Option<&RustTypeMarker> {
		if let Some(t) = self.extrinsics.get(ty, spec, chain) {
			match t {
				RustTypeMarker::TypePointer(t) => self.resolve_helper("runtime", t),
				t => Some(t),
			}
		} else {
			None
		}
	}

	/// Try to resolve a type pointer from the default definitions (definitions.json)
	fn resolve_helper(&self, module: &str, ty_pointer: &str) -> Option<&RustTypeMarker> {
		log::trace!("Helper resolving {}, {}", module, ty_pointer);

		if let Some(t) = self.mods.get_type(module, ty_pointer) {
			log::trace!("Type {} found in module {}", &ty_pointer, module);
			Some(t)
		} else if let Some(t) = self.mods.get_type("runtime", ty_pointer) {
			log::trace!("Type not found in {}, trying `runtime` for type {}", module, ty_pointer);
			Some(t)
		} else if let Some(t) = self.check_other_modules(ty_pointer) {
			log::trace!("trying other modules");
			Some(t)
		} else {
			None
		}
	}

	/// check if an override exists for a given type
	///
	/// if it does, return the types/type pointer
	fn check_overrides(&self, module: &str, ty: &str, spec: u32, chain: &str) -> Option<&RustTypeMarker> {
		// check if the type is a module override first
		if let Some(m) = self.overrides.get_module_types(module) {
			if let Some(ty) = m.get(ty) {
				return Some(ty);
			}
		}

		// if it isn't in modules, chain types is next
		self.overrides.get_chain_types(chain, spec)?.get(ty)
	}

	/// Checks all modules for the types
	fn check_other_modules(&self, ty_pointer: &str) -> Option<&RustTypeMarker> {
		self.mods.iter_types().find(|(n, _)| n.as_str() == ty_pointer).map(|(_, t)| t)
	}
}

impl TypeDetective for TypeResolver {
	fn get(&self, chain: &str, spec: u32, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		log::trace!("Getting type {}", ty);
		let ty = regex::sanitize_ty(ty)?;
		let module = module.to_ascii_lowercase();
		let chain = chain.to_ascii_lowercase();
		TypeResolver::get(self, &chain, spec, &module, &ty)
	}

	fn try_fallback(&self, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		let ty = regex::sanitize_ty(ty)?;
		let module = module.to_ascii_lowercase();

		TypeResolver::try_fallback(self, &module, &ty)
	}

	fn get_extrinsic_ty(&self, chain: &str, spec: u32, ty: &str) -> Option<&RustTypeMarker> {
		let ty = regex::sanitize_ty(ty)?;
		let chain = chain.to_ascii_lowercase();

		TypeResolver::get_ext_ty(self, &chain, spec, &ty)
	}
}

#[cfg(test)]
mod tests {
	use super::default::*;
	use super::*;
	use desub_legacy::{EnumField, StructField};

	#[test]
	fn should_get_type_from_module() -> Result<()> {
		let post_1031_dispatch_error = RustTypeMarker::Enum(vec![
			EnumField::new("Other".into(), Some(RustTypeMarker::Null)),
			EnumField::new("CannotLookup".into(), Some(RustTypeMarker::Null)),
			EnumField::new("BadOrigin".into(), Some(RustTypeMarker::Null)),
			EnumField::new("Module".into(), Some(RustTypeMarker::TypePointer("DispatchErrorModule".to_string()))),
			EnumField::new("ConsumerRemaining".into(), Some(RustTypeMarker::Null)),
			EnumField::new("NoProviders".into(), Some(RustTypeMarker::Null)),
			EnumField::new("Token".into(), Some(RustTypeMarker::TypePointer("TokenError".to_string()))),
			EnumField::new("Arithmetic".into(), Some(RustTypeMarker::TypePointer("ArithmeticError".to_string()))),
		]);

		let types = TypeResolver::default();
		let t = types.get("kusama", 1040, "system", "DispatchError").unwrap();
		assert_eq!(t, &post_1031_dispatch_error);
		Ok(())
	}

	#[test]
	fn should_resolve_a_type() -> Result<()> {
		let correct = RustTypeMarker::Struct(vec![
			StructField { name: "id".to_string(), ty: RustTypeMarker::TypePointer("LockIdentifier".to_string()) },
			StructField { name: "amount".to_string(), ty: RustTypeMarker::TypePointer("Balance".to_string()) },
			StructField { name: "until".to_string(), ty: RustTypeMarker::TypePointer("BlockNumber".to_string()) },
			StructField { name: "reasons".to_string(), ty: RustTypeMarker::TypePointer("WithdrawReasons".to_string()) },
		]);
		let types = TypeResolver::default();
		let resolved = types.get("kusama", 1040, "balances", "BalanceLockTo212").unwrap();
		assert_eq!(&correct, resolved);
		Ok(())
	}

	#[test]
	fn should_get_duplicated_types() -> Result<()> {
		let types = TypeResolver::default();
		let t = types.get("kusama", 1040, "contracts", "StorageKey").unwrap();
		assert_eq!(t, &RustTypeMarker::TypePointer("ContractStorageKey".to_string()));
		Ok(())
	}

	#[test]
	fn should_adhere_to_spec() -> Result<()> {
		let pre_1019_balance_lock = RustTypeMarker::Struct(vec![
			StructField { name: "id".to_string(), ty: RustTypeMarker::TypePointer("LockIdentifier".to_string()) },
			StructField { name: "amount".to_string(), ty: RustTypeMarker::TypePointer("Balance".to_string()) },
			StructField { name: "reasons".to_string(), ty: RustTypeMarker::TypePointer("Reasons".to_string()) },
		]);
		let types = TypeResolver::default();
		let t = types.get("kusama", 1000, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &pre_1019_balance_lock);
		let t = types.get("kusama", 1018, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &pre_1019_balance_lock);
		let t = types.get("kusama", 1031, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &RustTypeMarker::TypePointer("BalanceLockTo212".to_string()));
		let t = types.get("kusama", 1019, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &RustTypeMarker::TypePointer("BalanceLockTo212".to_string()));
		let t = types.get("kusama", 1032, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &RustTypeMarker::TypePointer("BalanceLockTo212".to_string()));
		let t = types.get("kusama", 1042, "balances", "BalanceLock").unwrap();
		assert_eq!(t, &RustTypeMarker::TypePointer("BalanceLockTo212".to_string()));
		Ok(())
	}

	#[test]
	fn should_differentiate_chains() {
		let types = TypeResolver::default();
		let dot_t = types.get("polkadot", 5, "runtime", "LookupSource").unwrap();
		let ksm_t = types.get("kusama", 5, "runtime", "LookupSource").unwrap();
		assert_eq!(dot_t, &RustTypeMarker::TypePointer("AccountId".to_string()));
		assert_ne!(dot_t, ksm_t);
	}

	#[test]
	fn should_deserialize_overrides() {
		let overrides = Overrides::new(OVERRIDES).unwrap();
		dbg!(overrides);
	}

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

	#[test]
	fn should_deserialize_definitions() -> Result<()> {
		let types = Modules::new(DEFINITIONS)?;
		dbg!(&types);
		Ok(())
	}
}
