#[cfg(feature = "polkadot")]
pub mod polkadot;

mod definitions;
mod overrides;
mod extrinsics;
pub mod error;

pub use self::definitions::*;
pub use self::overrides::*;
pub use self::extrinsics::*;

use serde::{Serialize, Deserialize, de::{Deserializer, MapAccess, Visitor}};
use core::{regex, EnumField, RustTypeMarker, SetField, StructField, StructUnitOrTuple};
use std::{collections::HashMap, fmt};

#[derive(Serialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct Modules {
    // module name -> Type Map of module
    pub modules: HashMap<String, ModuleTypes>,
}

#[derive(Serialize, Debug, Default, PartialEq, Eq, Clone)]
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
                m => {
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
    if val.is_string() {
        module_types.insert(
            key.to_string(),
            regex::parse(val.as_str().expect("Checked; qed")).expect("not a type"),
        );
    } else if val.is_object() {
        let obj = val
            .as_object()
            .expect("checked for object before unwrap; qed");
        if obj.contains_key("_enum") {
            module_types.insert(key.to_string(), parse_enum(&obj["_enum"]));
        } else if obj.contains_key("_set") {
            let obj = obj["_set"].as_object().expect("_set is a map");
            module_types.insert(key.to_string(), parse_set(obj));
        } else if obj.contains_key("_alias") {
            let mut fields = Vec::new();
            for (key, val) in obj.iter() {
                if key == "_alias" {
                    continue;
                } else {
                    let field = StructField::new(
                        key,
                        regex::parse(&val_to_str(val)).expect("Not a type"),
                    );
                    fields.push(field);
                }
            }
            module_types.insert(key.to_string(), RustTypeMarker::Struct(fields));
        } else {
            let mut fields = Vec::new();
            for (key, val) in obj.iter() {
                let field = StructField::new(
                    key,
                    regex::parse(&val_to_str(val)).expect("Not a type"),
                );
                fields.push(field);
            }
            module_types.insert(key.to_string(), RustTypeMarker::Struct(fields));
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
        let num: u8 = serde_json::from_value(value.clone()).expect("Must be u8");
        let set_field = SetField::new(key, num);
        set_vec.push(set_field)
    }
    RustTypeMarker::Set(set_vec)
}

/// internal api to convert a serde value to str
///
/// # Panics
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
        let rust_enum = rust_enum
            .into_iter()
            .map(|f| f.into())
            .collect::<Vec<EnumField>>();
        RustTypeMarker::Enum(rust_enum)
    // all enum 'objects' in polkadot.js definitions are tuple-enums
    } else if obj.is_object() {
        let obj = obj.as_object().expect("Checked before casting; qed");
        let mut rust_enum = Vec::new();
        for (key, value) in obj.iter() {
            let value = if value.is_null() {
                "null"
            } else {
                value.as_str().expect("will be str; qed")
            };
            let field = EnumField::new(
                Some(key.into()),
                StructUnitOrTuple::Tuple(regex::parse(value).expect("Not a type")),
            );
            rust_enum.push(field);
        }
        RustTypeMarker::Enum(rust_enum)
    // so far, polkadot.js does not define any struct-like enums
    } else {
        panic!("Unnaccounted type")
    }
}

