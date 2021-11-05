// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use super::{Composite, Primitive, Value, Variant};
use serde::{ Serialize, ser::{ SerializeMap, SerializeSeq } };

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            Value::Composite(val) => val.serialize(serializer),
            Value::Variant(val) => val.serialize(serializer),
            Value::BitSequence(val) => val.serialize(serializer),
            Value::Primitive(val) => val.serialize(serializer),
        }
    }
}

impl Serialize for Composite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            Composite::Named(vals) => {
                let mut map = serializer.serialize_map(Some(vals.len()))?;
                for (key, val) in vals {
                    map.serialize_entry(key, val)?;
                }
                map.end()
            },
            Composite::Unnamed(vals) => {
                let mut seq = serializer.serialize_seq(Some(vals.len()))?;
                for val in vals {
                    seq.serialize_element(val)?;
                }
                seq.end()
            }
        }
    }
}

impl Serialize for Primitive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        // Delegate to the serialization strategy used by the primitive types.
        match self {
            Primitive::Bool(v) => v.serialize(serializer),
            Primitive::Char(v) => v.serialize(serializer),
            Primitive::Str(v) => v.serialize(serializer),
            Primitive::U8(v) => v.serialize(serializer),
            Primitive::U16(v) => v.serialize(serializer),
            Primitive::U32(v) => v.serialize(serializer),
            Primitive::U64(v) => v.serialize(serializer),
            Primitive::U128(v) => v.serialize(serializer),
            Primitive::U256(v) => v.serialize(serializer),
            Primitive::I8(v) => v.serialize(serializer),
            Primitive::I16(v) => v.serialize(serializer),
            Primitive::I32(v) => v.serialize(serializer),
            Primitive::I64(v) => v.serialize(serializer),
            Primitive::I128(v) => v.serialize(serializer),
            Primitive::I256(v) => v.serialize(serializer),
        }
    }
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        // W can't use the enum serializing in the serde data model because that requires static
        // strs and enum indexes, which we don't have (since this is a runtime value), so we serialize
        // as a map with a type and a value, and make sure that we allow this format when attempting to
        // deserialize into a `Variant` type for a bit of symmetry (although note that if you try to deserialize
        // this into a `Value` type it'll have no choice but to deserialize straight into a `Composite::Named` map).
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("values", &self.values)?;
        map.end()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::json;

    fn assert_value(value: Value, expected: serde_json::Value) {
        let val = serde_json::to_value(&value).expect("can serialize to serde_json::Value");
        assert_eq!(val, expected);
    }

    #[test]
    fn serialize_primitives() {
        // a subset of the primitives to sanity check that they are unwrapped:
        assert_value(Value::Primitive(Primitive::U8(1)), json!(1));
        assert_value(Value::Primitive(Primitive::U16(1)), json!(1));
        assert_value(Value::Primitive(Primitive::U32(1)), json!(1));
        assert_value(Value::Primitive(Primitive::U64(1)), json!(1));
        assert_value(Value::Primitive(Primitive::Bool(true)), json!(true));
        assert_value(Value::Primitive(Primitive::Bool(false)), json!(false));
    }

    #[test]
    fn serialize_composites() {
        assert_value(
            Value::Composite(Composite::Named(vec![
                ("a".into(), Value::Primitive(Primitive::Bool(true))),
                ("b".into(), Value::Primitive(Primitive::Str("hello".into()))),
                ("c".into(), Value::Primitive(Primitive::Char('c'))),
            ])),
            json!({
                "a": true,
                "b": "hello",
                "c": 'c'
            })
        );
        assert_value(
            Value::Composite(Composite::Unnamed(vec![
                Value::Primitive(Primitive::Bool(true)),
                Value::Primitive(Primitive::Str("hello".into())),
                Value::Primitive(Primitive::Char('c')),
            ])),
            json!([
                true,
                "hello",
                'c'
            ])
        )
    }

    #[test]
    fn serialize_variants() {
        assert_value(
            Value::Variant(Variant {
                name: "Foo".into(),
                values: Composite::Named(vec![
                    ("a".into(), Value::Primitive(Primitive::Bool(true))),
                    ("b".into(), Value::Primitive(Primitive::Str("hello".into()))),
                    ("c".into(), Value::Primitive(Primitive::Char('c'))),
                ])
            }),
            json!({
                "name": "Foo",
                "values": {
                    "a": true,
                    "b": "hello",
                    "c": 'c'
                }
            })
        );
        assert_value(
            Value::Variant(Variant {
                name: "Bar".into(),
                values: Composite::Unnamed(vec![
                    Value::Primitive(Primitive::Bool(true)),
                    Value::Primitive(Primitive::Str("hello".into())),
                    Value::Primitive(Primitive::Char('c')),
                ])
            }),
            json!({
                "name": "Bar",
                "values": [
                    true,
                    "hello",
                    'c'
                ]
            })
        )
    }

}