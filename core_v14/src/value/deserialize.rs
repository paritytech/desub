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

use serde::{ Deserializer, de::{ self, IntoDeserializer, MapAccess, SeqAccess}, forward_to_deserialize_any };
use std::fmt::Display;
use super::{ Value, Composite, Primitive, Variant };

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum DeserializeError {
    #[error("{0}")]
    String(String),
    #[error("{0}")]
    Str(&'static str),
}

impl de::Error for DeserializeError {
    fn custom<T: Display>(msg:T) -> Self {
        DeserializeError::String(msg.to_string())
    }
}

impl <'de> Deserializer<'de> for Value {
    type Error = DeserializeError;

    // Most methods will delegate to this. We just call the visitor with whatever
    // we happen to have. We'll specialise a few cases in subsequent methods.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
            match self {
                Self::BitSequence(_seq) => {
                    // The deserialize visitor expects a sequence of:
                    // `u8` (head-bit index), `u64` (number of bits), `[T]` (data contents).
                    // where T will be u8 here. The problem is that we can't get the value
                    // for the head-bit index. Perhaps we can work out what it should be, but
                    // for now we just give up.
                    return Err(DeserializeError::Str("Deserializing a BitSequence is current unsupported"))
                },
                Value::Composite(composite) => {
                    composite.deserialize_any(visitor)
                },
                Value::Variant(variant) => {
                    variant.deserialize_any(visitor)
                },
                Value::Primitive(prim) => {
                    prim.deserialize_any(visitor)
                },
            }
    }

    // A newtype struct like Foo(u8) will look for an input Primitive::U8;
    // by default it would expect a sequence of values (of length 1).
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        visitor.visit_seq(SingleValueSeq { val: Some(self) })
    }

    // A tuple like (u8, bool) will look for an input like Composite(vec![Primitive::U8, Primitive::Bool]).
    // Complain if lengths don't match rather than allowing deserialization (the default). Don't mind
    // whether values are named or not; we ignore names
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        match self {
            // Let the Composite impl handle this:
            Value::Composite(composite) => {
                composite.deserialize_tuple(len, visitor)
            },
            // An enum variant of either of the above is fine, too (we just ignore the enum bit):
            Value::Variant(variant) => {
                variant.values.deserialize_tuple(len, visitor)
            },
            // These two aren't valid:
            Value::Primitive(_) => {
                return Err(DeserializeError::Str("Cannot deserialize primitive type into tuple struct"));
            },
            Value::BitSequence(_) => {
                return Err(DeserializeError::Str("Cannot deserialize BitSequence into tuple struct"));
            }
        }
    }

    // Handle the same as above
    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.deserialize_tuple(len, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq
        map struct enum identifier ignored_any
    }
}

impl <'de> Deserializer<'de> for Composite {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        match self {
            Composite::Named(fields) => {
                visitor.visit_map(NamedFields {
                    iter: fields.into_iter(),
                    value: None
                })
            },
            Composite::Unnamed(fields) => {
                visitor.visit_seq(UnnamedFields {
                    iter: fields.into_iter()
                })
            },
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        match self {
            // A sequence of named values just ignores the names:
            Composite::Named(values) => {
                if values.len() != len {
                    return Err(DeserializeError::String(format!("Cannot deserialize composite of length {} into tuple of length {}", values.len(), len)));
                }
                visitor.visit_seq(NamedFields { iter: values.into_iter(), value: None })
            },
            // A sequence of unnamed values is ideal:
            Composite::Unnamed(values) => {
                if values.len() != len {
                    return Err(DeserializeError::String(format!("Cannot deserialize composite of length {} into tuple of length {}", values.len(), len)));
                }
                visitor.visit_seq(UnnamedFields { iter: values.into_iter() })
            },
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.deserialize_tuple(len, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        map struct enum identifier ignored_any
    }
}


impl <'de> Deserializer<'de> for Variant {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        todo!()
    }

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
        let v = self.value.take().expect("next_key_seed should have populated the value");
        seed.deserialize(v)
    }
}
impl <'de> SeqAccess<'de> for NamedFields {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
            match self.iter.next() {
                Some((_k,v)) => seed.deserialize(v).map(Some),
                None => Ok(None)
            }
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

struct SingleValueSeq {
    val: Option<Value>
}

impl <'de> SeqAccess<'de> for SingleValueSeq {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
            let val = match self.val.take() {
                Some(val) => val,
                None => return Ok(None)
            };
            Ok(Some(seed.deserialize(val)?))
    }
}


#[cfg(test)]
mod test {

    use crate::value::BitSequence;
    use serde::Deserialize;

    use super::*;

    #[test]
    fn de_into_struct() {
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
    fn de_into_tuple_struct() {
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

    #[test]
    fn de_into_newtype_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo(String);

        let val = Value::Primitive(Primitive::Str("hello".into()));

        assert_eq!(
            Foo::deserialize(val),
            Ok(Foo("hello".into()))
        )
    }

    #[test]
    fn de_into_tuple() {
        let val = Value::Composite(Composite::Unnamed(vec![
            Value::Primitive(Primitive::Str("hello".into())),
            Value::Primitive(Primitive::Bool(true)),
        ]));

        assert_eq!(
            <(String, bool)>::deserialize(val),
            Ok(("hello".into(), true))
        )
    }

    #[test]
    fn de_named_into_tuple() {
        let val = Value::Composite(Composite::Named(vec![
            ("a".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
        ]));

        assert_eq!(
            <(String, bool)>::deserialize(val),
            Ok(("hello".into(), true))
        )
    }

    #[test]
    fn de_into_tuple_wrong_length_should_fail() {
        let val = Value::Composite(Composite::Unnamed(vec![
            Value::Primitive(Primitive::Str("hello".into())),
            Value::Primitive(Primitive::Bool(true)),
            Value::Primitive(Primitive::U8(123)),
        ]));

        <(String, bool)>::deserialize(val)
            .expect_err("Wrong length, should err");
    }

    #[test]
    fn de_bitvec() {
        use bitvec::{ bitvec, order::Lsb0 };
        let val = Value::BitSequence(bitvec![Lsb0, u8; 0, 1, 1, 0, 1, 0, 1, 0]);

        BitSequence::deserialize(val).expect_err("We can't deserialize this yet");
    }

}