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

use crate::{error::Error, regex};
use super::{Modules, ModuleTypes};
use core::{decoder::Decoder, RustEnum, RustTypeMarker, SetField, StructField};
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::{collections::HashMap, fmt, marker::PhantomData};

// TODO: open this file or pass it via CLI to reduce binary size
pub const DEFS: &'static str = include_str!("./dot_definitions/definitions.json");

pub fn register() -> Result<(), Error> {
    let decoder = Decoder::new();
    Ok(())
}

pub fn definitions(raw_json: &str) -> Result<Modules, Error> {
    let types: Modules = serde_json::from_str(raw_json)?;
    Ok(types)
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
                    match key {
                        _ => {
                            let val: ModuleTypes = map.next_value()?;
                            modules.insert(key.to_string(), val);
                        }
                    }
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
                m @ _ => {
                    let val: serde_json::Value = map.next_value()?;
                    //let val = val.as_object().expect("Types must refer to an object");
                    parse_mod_types(&mut module_types, m, &val);
                }
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
            parse_type(val.as_str().expect("Checked; qed")),
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
        "bool" => RustTypeMarker::Bool,

        s @ _ => {
            let re = regex::rust_array_decl();
            if re.is_match(s) {
                let caps = re.captures(s).expect("checked for array declaration; ");

                let t = caps
                    .name("type")
                    .expect("type match should always exist")
                    .as_str();
                let size = caps.name("size").expect("name match should always exist");
                let caps = caps
                    .iter()
                    .map(|c| c.map(|c| c.as_str()))
                    .collect::<Vec<Option<&str>>>();

                let ty = if caps[2].is_some() {
                    match t {
                        "u" => RustTypeMarker::U8,
                        "i" => RustTypeMarker::I8,
                        "f" => panic!("type does not exist 'f8'"),
                        _ => panic!("impossible match encountered"),
                    }
                } else if caps[3].is_some() {
                    match t {
                        "u" => RustTypeMarker::U16,
                        "i" => RustTypeMarker::I16,
                        "f" => panic!("type does not exist 'f16'"),
                        _ => panic!("impossible match encountered"),
                    }
                } else if caps[4].is_some() {
                    match t {
                        "u" => RustTypeMarker::U32,
                        "i" => RustTypeMarker::I32,
                        "f" => RustTypeMarker::F32,
                        _ => panic!("impossible match encountered"),
                    }
                } else if caps[5].is_some() {
                    match t {
                        "u" => RustTypeMarker::U64,
                        "i" => RustTypeMarker::I64,
                        "f" => RustTypeMarker::F64,
                        _ => panic!("impossible match encountered"),
                    }
                } else if caps[6].is_some() {
                    match t {
                        "u" => RustTypeMarker::U128,
                        "i" => RustTypeMarker::I128,
                        "f" => panic!("type does not exist: 'f128'"),
                        _ => panic!("impossible match encountered"),
                    }
                } else {
                    panic!("Couldn't determine size of array");
                };
                let ty = Box::new(ty);
                let size = size
                    .as_str()
                    .parse::<usize>()
                    .expect("Should always be number");
                RustTypeMarker::Array { size, ty }
            } else {
                RustTypeMarker::TypePointer(t.to_string())
            }
        }
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
    fn should_deserialize_correctly() -> Result<(), Error> {
        let deser_dot_types = definitions(RAW_JSON)?;
        let mut modules = HashMap::new();
        let mut types = HashMap::new();
        types.insert(
            "Extrinsic".to_string(),
            RustTypeMarker::TypePointer("GenericExtrinsic".to_string()),
        );
        types.insert(
            "hash".to_string(),
            RustTypeMarker::TypePointer("H512".to_string()),
        );
        types.insert("BlockNumber".to_string(), RustTypeMarker::U64);
        types.insert(
            "ChangesTrieConfiguration".to_string(),
            RustTypeMarker::Struct(vec![
                StructField {
                    name: "digestInterval".to_string(),
                    ty: RustTypeMarker::U32,
                },
                StructField {
                    name: "digestLevels".to_string(),
                    ty: RustTypeMarker::U32,
                },
            ]),
        );
        types.insert(
            "DispatchInfo".to_string(),
            RustTypeMarker::Struct(vec![
                StructField {
                    name: "weight".to_string(),
                    ty: RustTypeMarker::TypePointer("Weight".to_string()),
                },
                StructField {
                    name: "class".to_string(),
                    ty: RustTypeMarker::TypePointer("DispatchClass".to_string()),
                },
                StructField {
                    name: "paysFee".to_string(),
                    ty: RustTypeMarker::Bool,
                },
            ]),
        );
        types.insert(
            "MultiSignature".to_string(),
            RustTypeMarker::Enum(RustEnum::Struct(vec![
                StructField {
                    name: "Ed25519".to_string(),
                    ty: RustTypeMarker::TypePointer("Ed25519Signature".to_string()),
                },
                StructField {
                    name: "Sr25519".to_string(),
                    ty: RustTypeMarker::TypePointer("Sr25519Signature".to_string()),
                },
                StructField {
                    name: "Ecdsa".to_string(),
                    ty: RustTypeMarker::TypePointer("EcdsaSignature".to_string()),
                },
            ])),
        );
        types.insert(
            "Reasons".to_string(),
            RustTypeMarker::Enum(RustEnum::Unit(vec![
                "Fee".to_string(),
                "Misc".to_string(),
                "All".to_string(),
            ])),
        );
        types.insert(
            "WithdrawReasons".to_string(),
            RustTypeMarker::Set(vec![
                SetField {
                    name: "TransactionPayment".to_string(),
                    num: 1,
                },
                SetField {
                    name: "Transfer".to_string(),
                    num: 2,
                },
                SetField {
                    name: "Reserve".to_string(),
                    num: 4,
                },
                SetField {
                    name: "Fee".to_string(),
                    num: 8,
                },
                SetField {
                    name: "Tip".to_string(),
                    num: 16,
                },
            ]),
        );

        for (key, val) in types.iter() {
            assert_eq!(val, &deser_dot_types.modules["runtime"].types[key]);
        }

        let mod_types = ModuleTypes { types };
        modules.insert("runtime".to_string(), mod_types);
        let dot_types = Modules { modules };
        assert_eq!(dot_types, deser_dot_types);
        Ok(())
    }
}
