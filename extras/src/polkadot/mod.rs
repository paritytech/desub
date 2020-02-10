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

use core::{RustTypeMarker, decoder::Decoder};
use std::{marker::PhantomData, collections::HashMap, fmt};
use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};


// TODO: open this file or pass it via CLI to reduce binary size
const DEFS: &'static str = include_str!("./definitions/definitions.json");

pub fn register() {
    let decoder = Decoder::new();
    let decoded: PolkadotTypes = serde_json::from_str(DEFS)
        .expect("Deserialization is infallible");
//    let decoded: serde_json::Value = serde_json::from_str(DEFS)
//        .expect("Deserialization is infallible");
//    dbg!(decoded);
}

#[derive(Default, Debug)]
pub struct PolkadotTypes {
    // module name -> Type Map of module
    pub modules: HashMap<String, ModuleTypes>
}

impl<'de> Deserialize<'de> for PolkadotTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PolkadotTypesVisitor;

        impl<'de> Visitor <'de> for PolkadotTypesVisitor {
            type Value = PolkadotTypes;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Map types")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PolkadotTypes, V::Error>
            where
                V: MapAccess<'de>
            {
                while let Some(key) = map.next_key::<serde_json::Value>()? {
                    // this key are all modules, IE
                    // "runtime", "metadata", "rpc", etc
                    // and then it goes "types": { string: string / string : object /  }
                    match key {
                        _ => {
                            let val: serde_json::Value = map.next_value()?;
                            dbg!(&key);
                        }
                    }
                }
                Ok(PolkadotTypes::default())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error
            {
                dbg!(v);
                Ok(PolkadotTypes::default())
            }
        }
        deserializer.deserialize_map(PolkadotTypesVisitor);
        Ok(PolkadotTypes::default())
    }
}

#[derive(Debug, Default)]
pub struct ModuleTypes {
    // Type Name -> Type
    pub types: HashMap<String, RustTypeMarker>
}
/*
impl<'de> Deserialize<'de> for ModuleTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

    }
}
*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize() {
        register()
    }
}
