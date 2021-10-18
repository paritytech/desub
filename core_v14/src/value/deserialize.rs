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

use serde::{ Deserializer, Deserialize, de::{ self, IntoDeserializer, MapAccess, SeqAccess}, forward_to_deserialize_any };
use std::fmt::Display;
use super::{ Value, Composite, Primitive, Variant };

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum DeserializeError {
    #[error("{0}")]
    Custom(String),
    #[error("Seen a key, but where is the corresponding value?")]
    MapValueExpected
}

impl de::Error for DeserializeError {
    fn custom<T: Display>(msg:T) -> Self {
        DeserializeError::Custom(msg.to_string())
    }
}

impl <'de> Deserializer<'de> for Value {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
            match self {
                Self::BitSequence(seq) => {
                    visitor.visit_bytes(seq.as_raw_slice())
                },
                Value::Composite(Composite::Named(fields)) => {
                    visitor.visit_map(NamedFields {
                        iter: fields.into_iter(),
                        value: None
                    })
                },
                Value::Composite(Composite::Unnamed(fields)) => {
                    visitor.visit_seq(UnnamedFields {
                        iter: fields.into_iter()
                    })
                },
                Value::Variant(variant) => todo!(),
                Value::Primitive(prim) => {
                    prim.deserialize_any(visitor)
                },
            }
    }

    // Is this sane?
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl <'de> Deserializer<'de> for Variant {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        todo!()
    }

    // Is this sane?
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl <'de> Deserializer<'de> for Primitive {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
            match self {
                Primitive::Bool(v) => visitor.visit_bool(v),
                Primitive::Char(v) => visitor.visit_char(v),
                Primitive::Str(v) => visitor.visit_string(v),
                Primitive::U8(v) => visitor.visit_u8(v),
                Primitive::U16(v) => visitor.visit_u16(v),
                Primitive::U32(v) => visitor.visit_u32(v),
                Primitive::U64(v) => visitor.visit_u64(v),
                Primitive::U128(v) => visitor.visit_u128(v),
                Primitive::U256(v) => visitor.visit_bytes(&v),
                Primitive::I8(v) => visitor.visit_i8(v),
                Primitive::I16(v) => visitor.visit_i16(v),
                Primitive::I32(v) => visitor.visit_i32(v),
                Primitive::I64(v) => visitor.visit_i64(v),
                Primitive::I128(v) => visitor.visit_i128(v),
                Primitive::I256(v) => visitor.visit_bytes(&v),
            }
    }

    // Is this sane?
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct NamedFields {
    iter: std::vec::IntoIter<(String, Value)>,
    value: Option<Value>
}

impl <'de> MapAccess<'de> for NamedFields {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de> {
        let (k, v) = match self.iter.next() {
            Some(kv) => kv,
            None => return Ok(None)
        };
        self.value = Some(v);
        seed.deserialize(k.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de> {
        let v = self.value.take().ok_or(DeserializeError::MapValueExpected)?;
        seed.deserialize(v)
    }
}

struct UnnamedFields {
    iter: std::vec::IntoIter<Value>
}

impl <'de> SeqAccess<'de> for UnnamedFields {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
            match self.iter.next() {
                Some(v) => seed.deserialize(v).map(Some),
                None => Ok(None)
            }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn de_into_basic_named_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo {
            a: u8,
            b: bool,
        }

        let val = Value::Composite(Composite::Named(vec![
            ("a".into(), Value::Primitive(Primitive::U8(123))),
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
        ]));

        assert_eq!(
            Foo::deserialize(val),
            Ok(Foo { a: 123, b: true })
        )
    }

    #[test]
    fn test_into_basic_unnamed_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo(u8, bool, String);

        let val = Value::Composite(Composite::Unnamed(vec![
            Value::Primitive(Primitive::U8(123)),
            Value::Primitive(Primitive::Bool(true)),
            Value::Primitive(Primitive::Str("hello".into())),
        ]));

        assert_eq!(
            Foo::deserialize(val),
            Ok(Foo(123, true, "hello".into()))
        )
    }

}