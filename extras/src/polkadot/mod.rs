Warning: can't set `wrap_comments = true`, unstable features are only available in nightly channel.
Warning: can't set `spaces_around_ranges = true`, unstable features are only available in nightly channel.
// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
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

use core::{decoder::Decoder, RustTypeMarker};
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::{collections::HashMap, fmt, marker::PhantomData};

// TODO: open this file or pass it via CLI to reduce binary size
const DEFS: &'static str = include_str!("./definitions/definitions.json");

pub fn register() {
    let decoder = Decoder::new();
    let decoded: PolkadotTypes =
        serde_json::from_str(DEFS).expect("Deserialization is infallible");
    //    let decoded: serde_json::Value = serde_json::from_str(DEFS)
    //        .expect("Deserialization is infallible");
    //    dbg!(decoded);
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
                formatter.write_str("Map types")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PolkadotTypes, V::Error>
            where
                V: MapAccess<'de>,
            {
                let modules: HashMap<String, ModuleTypes> = HashMap::new();
                while let Some(key) = map.next_key::<&str>()? {
                    // this key are all modules, IE
                    // "runtime", "metadata", "rpc", etc
                    // and then it goes "types": { string: string / string : object /  }
                    match key {
                        _ => {
                            let val: ModuleTypes = map.next_value()?;
                            // dbg!(&key);
                        }
                    }
                }
                Ok(PolkadotTypes::default())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                dbg!(v);
                Ok(PolkadotTypes::default())
            }
        }
        deserializer.deserialize_map(PolkadotTypesVisitor);
        Ok(PolkadotTypes::default())
    }
}

impl<'de> Deserialize<'de> for ModuleTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
                let module_types: HashMap<String, RustTypeMarker> = HashMap::new();
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        // skip over "types" key, this encapsulates the types we actually care about
                        "types" => {
                            let val: ModuleTypes = map.next_value()?;
                        }
                        _ => {
                            let mut t: Option<RustTypeMarker> = None;
                            let val: serde_json::Value = map.next_value()?;
                            if val.is_string() {
                                t = Some(RustTypeMarker::Pointer(
                                    val.as_str()
                                        .expect("checked for str; qed")
                                        .to_string(),
                                ));
                            } else if val.is_object() {
                                let obj = val.as_object()
                                             .expect("checked for object before unwrap; qed");
                                let gen_struct: RustTypeMarker;

                            }
                            // let val: serde_json::Value = map.next_value()?;
                            dbg!(&key);
                            dbg!(&val);
                        }
                    }
                }
                Ok(ModuleTypes::default())
            }
        }

        deserializer.deserialize_map(ModuleTypeVisitor);
        Ok(ModuleTypes::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize() {
        register()
    }
}
