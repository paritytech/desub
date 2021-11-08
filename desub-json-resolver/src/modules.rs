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

use crate::error::Error;
use desub_legacy::{regex, EnumField, RustTypeMarker, SetField, StructField};
use serde::{
	de::{self, Deserializer, MapAccess, Visitor},
	Deserialize, Serialize,
};
use serde_json::{map::Map, Value};
use std::{collections::HashMap, fmt};

/// Types for each substrate Module
#[derive(Serialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct Modules {
	/// module name -> Type Map of module
	modules: HashMap<String, ModuleTypes>,
}

impl Modules {
	/// Construct this struct from JSON
	pub fn new(raw_json: &str) -> Result<Self, Error> {
		let modules: Modules = serde_json::from_str(raw_json)?;
		Ok(modules)
	}

	pub fn get(&self, ty: &str) -> Option<&ModuleTypes> {
		self.modules.get(ty)
	}

	pub fn get_type(&self, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		self.modules.get(module)?.get(ty)
	}

	pub fn try_fallback(&self, module: &str, ty: &str) -> Option<&RustTypeMarker> {
		self.modules.get(module)?.try_fallback(ty)
	}

	/// Iterate over all the types in each module
	pub fn iter_types(&self) -> impl Iterator<Item = (&String, &RustTypeMarker)> {
		self.modules.values().map(|v| v.types.iter()).flatten()
	}
}

/// Map of types to their Type Markers
#[derive(Serialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct ModuleTypes {
	/// Type Name -> Type
	types: HashMap<String, RustTypeMarker>,
	fallbacks: HashMap<String, RustTypeMarker>,
}

impl ModuleTypes {
	pub fn get(&self, ty: &str) -> Option<&RustTypeMarker> {
		self.types.get(ty)
	}

	pub fn try_fallback(&self, ty: &str) -> Option<&RustTypeMarker> {
		self.fallbacks.get(ty)
	}

	/// Merges a ModuleTypes struct with another, to create a new HashMap
	/// The `other` struct takes priority if there are type conflicts
	pub fn merge(&self, other: &ModuleTypes) -> ModuleTypes {
		let (mut types, mut fallbacks) = (self.types.clone(), self.fallbacks.clone());
		let other = other.clone();
		types.extend(other.types.into_iter());
		fallbacks.extend(other.fallbacks.into_iter());

		ModuleTypes { types, fallbacks }
	}
}

impl<'de> Deserialize<'de> for Modules {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ModulesVisitor;

		impl<'de> Visitor<'de> for ModulesVisitor {
			type Value = Modules;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("map types")
			}

			fn visit_map<V>(self, mut map: V) -> Result<Modules, V::Error>
			where
				V: MapAccess<'de>,
			{
				let mut modules: HashMap<String, ModuleTypes> = HashMap::new();
				while let Some(key) = map.next_key::<&str>()? {
					let val: ModuleTypes = map.next_value()?;
					modules.insert(key.to_string(), val);
				}
				Ok(Modules { modules })
			}
		}
		deserializer.deserialize_map(ModulesVisitor)
	}
}

impl<'de> Deserialize<'de> for ModuleTypes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_map(ModuleTypeVisitor)
	}
}

struct ModuleTypeVisitor;

impl<'de> Visitor<'de> for ModuleTypeVisitor {
	type Value = ModuleTypes;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("Map or string")
	}

	fn visit_map<V>(self, mut map: V) -> Result<ModuleTypes, V::Error>
	where
		V: MapAccess<'de>,
	{
		let mut types: HashMap<String, RustTypeMarker> = HashMap::new();
		let mut fallbacks: HashMap<String, RustTypeMarker> = HashMap::new();

		while let Some(key) = map.next_key::<&str>()? {
			match key {
				// skip over "types" key, this encapsulates the types we actually care
				// about
				"types" => {
					let mut obj: Value = map.next_value()?;
					let obj = obj.as_object_mut().ok_or_else(|| de::Error::custom("Types must refer to an object"))?;
					for (key, ref mut val) in obj.iter_mut() {
						parse_mod_types(&mut types, &mut fallbacks, key, val).map_err(de::Error::custom)?;
					}
				}
				m => {
					let mut val: Value = map.next_value()?;
					parse_mod_types(&mut types, &mut fallbacks, m, &mut val).map_err(de::Error::custom)?;
				}
			}
		}
		Ok(ModuleTypes { types, fallbacks })
	}
}

type TypeMap = HashMap<String, RustTypeMarker>;

/// In Polkadot-JS Definitions, an _object_ can be:
/// - Struct (no identifier),
/// - Enum (`_enum` identifier)
/// - Set (`_set`)
///
/// This function decides which is what and dispatches a call
/// to the appropriate parse fn.
fn parse_mod_types(
	module_types: &mut TypeMap,
	fallbacks: &mut TypeMap,
	key: &str,
	val: &mut Value,
) -> Result<(), Error> {
	match val {
		Value::String(s) => {
			module_types.insert(key.to_string(), regex::parse(s).ok_or_else(|| Error::from(s.to_string()))?);
		}
		Value::Object(ref mut obj) => {
			if obj.len() == 1 && obj.keys().any(|k| k == "_enum" || k == "_set") {
				let ty = match obj.iter().next().map(|(s, v)| (s.as_str(), v)) {
					Some(("_enum", v)) => parse_enum(v)?,
					Some(("_set", v)) => parse_set(v.as_object().expect("set is always an object"))?,
					Some((_, _)) => return Err(Error::UnexpectedType),
					None => panic!("This should never occur, checked for object length."),
				};
				module_types.insert(key.to_string(), ty);
			} else {
				if let Some(fallback) = clean_struct(obj)? {
					fallbacks.insert(key.to_string(), fallback);
				}
				let ty = parse_struct(obj)?;
				module_types.insert(key.to_string(), ty);
			}
		}
		Value::Null => {
			module_types.insert(key.to_string(), RustTypeMarker::Null);
		}
		_ => return Err(Error::UnexpectedType),
	}
	Ok(())
}

// Removes unsupported/unnecessary keys from struct,
// and returns fallback value if it exists.
fn clean_struct(map: &mut Map<String, Value>) -> Result<Option<RustTypeMarker>, Error> {
	map.remove("_alias"); // aliases are javascript-specific

	if let Some(fallback) = map.remove("_fallback") {
		let ty = match fallback {
			Value::String(s) => regex::parse(&s).ok_or_else(|| Error::from(s.to_string()))?,
			Value::Object(o) => parse_struct(&o)?,
			Value::Array(a) => parse_tuple(&a)?,
			Value::Null => RustTypeMarker::Null,
			_ => return Err(Error::UnexpectedType),
		};
		Ok(Some(ty))
	} else {
		Ok(None)
	}
}

fn parse_set(obj: &Map<String, Value>) -> Result<RustTypeMarker, Error> {
	let mut set_vec = Vec::new();
	for (key, value) in obj.iter() {
		let num: u8 = serde_json::from_value(value.clone())?;
		let set_field = SetField::new(key, num);
		set_vec.push(set_field)
	}
	Ok(RustTypeMarker::Set(set_vec))
}

/// Process the enum and return the representation as a Rust Type
///
/// # Panics
fn parse_enum(value: &Value) -> Result<RustTypeMarker, Error> {
	if value.is_array() {
		let arr = value.as_array().expect("checked before cast; qed");
		let rust_enum = arr
			.iter()
			.map(|u| {
				let name = u.as_str().expect("Will be string according to polkadot-js defs").to_string();
				EnumField::new(name, None)
			})
			.collect::<Vec<_>>();
		Ok(RustTypeMarker::Enum(rust_enum))
	} else if value.is_object() {
		let value = value.as_object().expect("Checked before casting; qed");
		// If all the values are numbers then we need to order the enum according to those numbers.
		// Some types like `ProxyType` in the runtime may vary from chain-to-chain.
		// So afaict Polkadot-Js types solve this by attaching a number to each variant according to index.
		let rust_enum = if value.values().fold(true, |_, v| v.is_number()) {
			let mut tuples = value
				.values()
				.map(|v| v.as_u64().expect("Must be u64"))
				.zip(value.keys().map(|k| EnumField::new(k.into(), None)))
				.collect::<Vec<(u64, EnumField)>>();
			tuples.sort_by_key(|(num, _)| *num);
			tuples.into_iter().map(|t| t.1).collect::<Vec<_>>()
		} else {
			let mut rust_enum = Vec::new();
			for (key, value) in value.iter() {
				match value {
					Value::Null => rust_enum.push(EnumField::new(key.into(), Some(RustTypeMarker::Null))),
					Value::String(s) => {
						let field = regex::parse(s).ok_or_else(|| Error::from(s.to_string()))?;
						rust_enum.push(EnumField::new(key.into(), Some(field)));
					}
					Value::Object(o) => {
						let rust_struct = parse_struct(o)?;
						rust_enum.push(EnumField::new(key.into(), Some(rust_struct)));
					}
					_ => return Err(Error::UnexpectedType),
				};
			}
			rust_enum
		};
		Ok(RustTypeMarker::Enum(rust_enum))
	} else {
		panic!("Unkown type")
	}
}

/// Parses a rust struct representation from a JSON Map.
fn parse_struct(rust_struct: &Map<String, Value>) -> Result<RustTypeMarker, Error> {
	let mut fields = Vec::new();
	for (key, value) in rust_struct.iter() {
		match value {
			Value::Null => {
				let field = StructField::new(key, RustTypeMarker::Null);
				fields.push(field);
			}
			Value::String(s) => {
				// points to some other type
				let ty = regex::parse(s).ok_or_else(|| s.to_string())?;
				let field = StructField::new(key, ty);
				fields.push(field);
			}
			Value::Object(o) => {
				// struct-within-a-struct
				let inner_struct = parse_struct(o)?;
				let field = StructField::new(key, inner_struct);
				fields.push(field);
			}
			Value::Array(a) => {
				let tuples = parse_tuple(a)?;
				let field = StructField::new(key, tuples);
				fields.push(field);
			}
			_ => return Err(Error::UnexpectedType),
		}
	}
	Ok(RustTypeMarker::Struct(fields))
}

fn parse_tuple(json_tuple: &[Value]) -> Result<RustTypeMarker, Error> {
	let mut tuple = Vec::new();
	for value in json_tuple.iter() {
		match value {
			Value::Null => tuple.push(RustTypeMarker::Null),
			Value::String(s) => {
				let ty = regex::parse(s).ok_or_else(|| s.to_string())?;
				tuple.push(ty);
			}
			_ => return Err(Error::UnexpectedType),
		}
	}
	Ok(RustTypeMarker::Tuple(tuple))
}

#[cfg(test)]
mod tests {
	use super::Modules;

	use crate::error::Error;
	use crate::ModuleTypes;
	use desub_legacy::{EnumField, RustTypeMarker, SetField, StructField};
	use std::collections::HashMap;
	const RAW_JSON: &str = r#"
{
	"runtime": {
		"types": {
			"Extrinsic": "GenericExtrinsic",
			"hash": "H512",
			"BlockNumber": "u64",
			"ChangesTrieConfiguration": {
				"digestInterval": "u32",
				"digestLevels": "u32"
			},
			"DispatchInfo": {
				"weight": "Weight",
				"class": "DispatchClass",
				"paysFee": "bool"
			},
			"MultiSignature": {
				"_enum": {
					"Ed25519": "Ed25519Signature",
					"Sr25519": "Sr25519Signature",
					"Ecdsa": "EcdsaSignature"
				}
			},
			"Reasons": {
				"_enum": [
					"Fee",
					"Misc",
					"All"
				]
			},
			"WithdrawReasons": {
				"_set": {
					"TransactionPayment": 1,
					"Transfer": 2,
					"Reserve": 4,
					"Fee": 8,
					"Tip": 16
				}
			}
		}
	}
}
"#;

	#[test]
	fn should_deserialize_modules() -> Result<(), Error> {
		let deser_dot_types = Modules::new(RAW_JSON)?;
		let mut modules = HashMap::new();
		let mut types = HashMap::new();
		types.insert("Extrinsic".to_string(), RustTypeMarker::TypePointer("GenericExtrinsic".to_string()));
		types.insert("hash".to_string(), RustTypeMarker::TypePointer("H512".to_string()));
		types.insert("BlockNumber".to_string(), RustTypeMarker::U64);
		types.insert(
			"ChangesTrieConfiguration".to_string(),
			RustTypeMarker::Struct(vec![
				StructField { name: "digestInterval".to_string(), ty: RustTypeMarker::U32 },
				StructField { name: "digestLevels".to_string(), ty: RustTypeMarker::U32 },
			]),
		);
		types.insert(
			"DispatchInfo".to_string(),
			RustTypeMarker::Struct(vec![
				StructField { name: "weight".to_string(), ty: RustTypeMarker::TypePointer("Weight".to_string()) },
				StructField { name: "class".to_string(), ty: RustTypeMarker::TypePointer("DispatchClass".to_string()) },
				StructField { name: "paysFee".to_string(), ty: RustTypeMarker::Bool },
			]),
		);
		types.insert(
			"MultiSignature".to_string(),
			RustTypeMarker::Enum(vec![
				EnumField {
					name: "Ed25519".to_string(),
					value: Some(RustTypeMarker::TypePointer("Ed25519Signature".to_string())),
				},
				EnumField {
					name: "Sr25519".to_string(),
					value: Some(RustTypeMarker::TypePointer("Sr25519Signature".to_string())),
				},
				EnumField {
					name: "Ecdsa".to_string(),
					value: Some(RustTypeMarker::TypePointer("EcdsaSignature".to_string())),
				},
			]),
		);
		types.insert(
			"Reasons".to_string(),
			RustTypeMarker::Enum(vec![
				EnumField { name: "Fee".into(), value: None },
				EnumField { name: "Misc".into(), value: None },
				EnumField { name: "All".into(), value: None },
			]),
		);
		types.insert(
			"WithdrawReasons".to_string(),
			RustTypeMarker::Set(vec![
				SetField { name: "TransactionPayment".to_string(), num: 1 },
				SetField { name: "Transfer".to_string(), num: 2 },
				SetField { name: "Reserve".to_string(), num: 4 },
				SetField { name: "Fee".to_string(), num: 8 },
				SetField { name: "Tip".to_string(), num: 16 },
			]),
		);

		for (key, val) in types.iter() {
			assert_eq!(val, &deser_dot_types.modules["runtime"].types[key]);
		}

		let mod_types = ModuleTypes { types, fallbacks: HashMap::new() };
		modules.insert("runtime".to_string(), mod_types);
		let dot_types = Modules { modules };
		assert_eq!(dot_types, deser_dot_types);
		Ok(())
	}
}
