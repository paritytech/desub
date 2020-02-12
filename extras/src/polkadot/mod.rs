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

use core::{decoder::Decoder, RustEnum, RustTypeMarker, StructField, SetField};
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::{collections::HashMap, fmt, marker::PhantomData};
use super::error::Error;

// TODO: open this file or pass it via CLI to reduce binary size
const DEFS: &'static str = include_str!("./definitions/definitions.json");

pub fn register() -> Result<(), Error> {
    let decoder = Decoder::new();
    Ok(())
}

pub fn definitions(raw_json: &str) -> Result<PolkadotTypes, Error> {
    let types: PolkadotTypes = serde_json::from_str(raw_json)?;
    Ok(types)
}

#[derive(Default, Debug)]
pub struct PolkadotTypes {
    // module name -> Type Map of module
    pub modules: HashMap<String, ModuleTypes>,
}

#[derive(Debug, Default)]
pub struct ModuleTypes {
    // Type Name -> Type
    pub types: HashMap<String, RustTypeMarker>,
}

impl<'de> Deserialize<'de> for PolkadotTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PolkadotTypesVisitor;

        impl<'de> Visitor<'de> for PolkadotTypesVisitor {
            type Value = PolkadotTypes;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("map types")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PolkadotTypes, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut modules: HashMap<String, ModuleTypes> = HashMap::new();
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        _ => {
                            let val: ModuleTypes = map.next_value()?;
                            modules.insert(key.to_string(), val);
                        }
                    }
                }
                Ok(PolkadotTypes { modules })
            }
        }
        deserializer.deserialize_map(PolkadotTypesVisitor)
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
        let mut module_types: HashMap<String, RustTypeMarker> = HashMap::new();

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                // skip over "types" key, this encapsulates the types we actually care
                // about
                "types" => {
                    let val: serde_json::Value = map.next_value()?;
                    let val = val.as_object().expect("Types must refer to an object");
                    for (key, val) in val.iter() {
                        parse_mod_types(&mut module_types, key, val);
                    }
                }
                _ => panic!("Received key that should not exist")
            }
        }
        Ok(ModuleTypes {
            types: module_types,
        })
    }
}

fn parse_mod_types(
    module_types: &mut HashMap<String, RustTypeMarker>,
    key: &str,
    val: &serde_json::Value,
) {
    let mut t: Option<RustTypeMarker> = None;
    if val.is_string() {
        module_types.insert(
            key.to_string(),
            parse_type(val.as_str().expect("Checked; qed"))
        );
    } else if val.is_object() {
        let obj = val
            .as_object()
            .expect("checked for object before unwrap; qed");

        let mut fields: Option<Vec<RustTypeMarker>> = None;
        if obj.contains_key("_enum") {
            module_types.insert(key.to_string(), parse_enum(&obj["_enum"]));
        } else if obj.contains_key("_set") {
            let obj = obj["_set"].as_object().expect("_set is a map");
            module_types.insert(key.to_string(), parse_set(obj));
        } else {
            let mut fields = Vec::new();
            for (key, val) in obj.iter() {
                let field = StructField::new(key, parse_type(&val_to_str(val)));
                fields.push(field);
            }
            let t = RustTypeMarker::Struct(fields);
            module_types.insert(key.to_string(), t);
        }
    }
}

/// internal api to convert a serde value to str
///
/// # Panics
/// panics if the value is not a string
fn val_to_str(v: &serde_json::Value) -> String {
    v.as_str().expect("will be string").to_string()
}

fn parse_set(obj: &serde_json::map::Map<String, serde_json::Value>) -> RustTypeMarker {
    let mut set_vec = Vec::new();
    for (key, value) in obj.iter() {
        let set_field = SetField::new(key, value.as_u64().expect("will not be negative"));
        set_vec.push(set_field)
    }
    RustTypeMarker::Set(set_vec)
}

fn parse_enum(obj: &serde_json::Value) -> RustTypeMarker {
    if obj.is_array() {
        let arr = obj.as_array().expect("checked before cast; qed");
        let mut rust_enum = Vec::new();
        for unit in arr.iter() {
            // if an enum is an array, it's a unit enum (stateless)
            rust_enum.push(
                unit.as_str()
                    .expect("Will be string according to polkadot-js defs")
                    .to_string(),
            )
        }
        RustTypeMarker::Enum(RustEnum::Unit(rust_enum))
    } else if obj.is_object() {
        let obj = obj.as_object().expect("Checked before casting; qed");
        let mut rust_enum = Vec::new();
        for (key, value) in obj.iter() {
            let value = if value.is_null() {
                "null"
            } else {
                value.as_str().expect("will be str; qed")
            };
            let field = StructField::new(key, parse_type(value));
            rust_enum.push(field);
        }
        RustTypeMarker::Enum(RustEnum::Struct(rust_enum))
    // if enum is an object, it's an enum with tuples defined as structs
    } else {
        panic!("Unnaccounted type")
    }
}

/// Returns a primitive type or rust TypePointer
fn parse_type(t: &str) -> RustTypeMarker {
    match t {
        "u8" => RustTypeMarker::U8,
        "u16" => RustTypeMarker::U16,
        "u32" => RustTypeMarker::U32,
        "u64" => RustTypeMarker::U64,
        "u128" => RustTypeMarker::U128,
        "usize" => RustTypeMarker::USize,

        "i8" => RustTypeMarker::I8,
        "i16" => RustTypeMarker::I16,
        "i32" => RustTypeMarker::I32,
        "i64" => RustTypeMarker::I64,
        "i128" => RustTypeMarker::I128,
        "isize" => RustTypeMarker::ISize,

        "f32" => RustTypeMarker::F32,
        "f64" => RustTypeMarker::F64,
        _ => RustTypeMarker::TypePointer(t.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
const RAW_JSON: &'static str = r#"
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
    fn should_deserialize() -> Result<(), Error> {
        let types = definitions(DEFS)?;
        dbg!(&types);
        Ok(())
    }

    #[test]
    fn should_deserialize_correctly() -> Result<() ,Error> {
        let types = definitions(RAW_JSON)?;
        dbg!(&types);
        Ok(())
    }
}
