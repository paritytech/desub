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

use serde::{Deserialize, Deserializer, de::{self, EnumAccess, VariantAccess, IntoDeserializer, MapAccess, SeqAccess}, forward_to_deserialize_any};
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

// Our Value trait needs to handle BitSeq itself, but otherwise delegates to
// the inner implementations of things to handle. This macro makes that less repetitive
// to write by only requiring a bitseq impl.
macro_rules! delegate_except_bitseq {
    (
        $name:ident ( $self:ident, $($arg:ident),* ),
            $seq:pat => $expr:expr
    ) => {
        match $self {
            Value::BitSequence($seq) => {
                $expr
            },
            Value::Composite(composite) => {
                composite.$name( $($arg),* )
            },
            Value::Variant(variant) => {
                variant.$name( $($arg),* )
            },
            Value::Primitive(prim) => {
                prim.$name( $($arg),* )
            },
        }
    }
}

// Here, we implement any specific methods that may be of interest to the subtypes, and
// delegate to their implementations. The exception if the BitSequence pattern, which we
// match and handle here, since it does not have a wrapper type to implement this on.
impl <'de> Deserializer<'de> for Value {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_any(self, visitor),
            _ => {
                // The deserialize visitor expects a sequence of:
                // `u8` (head-bit index), `u64` (number of bits), `[T]` (data contents).
                // where T will be u8 here. The problem is that we can't get the value
                // for the head-bit index. Perhaps we can work out what it should be, but
                // for now we just give up.
                return Err(DeserializeError::Str("Deserializing a BitSequence is current unsupported"))
            }
        }
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_newtype_struct(self, name, visitor),
            _ => {
                visitor.visit_seq(SingleValueSeq { val: Some(self) })
            }
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_tuple(self, len, visitor),
            _ => {
                return Err(DeserializeError::Str("Cannot deserialize BitSequence into a tuple"));
            }
        }
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_tuple_struct(self, name, len, visitor),
            _ => {
                return Err(DeserializeError::Str("Cannot deserialize BitSequence into a tuple struct"));
            }
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_unit(self, visitor),
            _ => {
                return Err(DeserializeError::Str("Cannot deserialize BitSequence into a ()"));
            }
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_unit_struct(self, name, visitor),
            _ => {
                return Err(DeserializeError::String(format!("Cannot deserialize BitSequence into the unit struct {}", name)));
            }
        }
    }

    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_enum(self, name, variants, visitor),
            _ => {
                return Err(DeserializeError::String(format!("Cannot deserialize BitSequence into the enum {}", name)));
            }
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_bytes(self, visitor),
            seq => {
                visitor.visit_bytes(seq.as_raw_slice())
            }
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        delegate_except_bitseq!{ deserialize_byte_buf(self, visitor),
            seq => {
                visitor.visit_byte_buf(seq.into_vec())
            }
        }
    }

    // None of the sub types particularly care about these, so we just allow them to forward to
    // deserialize_any and go from there.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        option seq map struct identifier ignored_any
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

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        // 0 length composite types can be treated as the unit type:
        if self.len() == 0 {
            visitor.visit_unit()
        } else {
            Err(DeserializeError::Str("Cannot deserialize non-empty Composite into a unit value"))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        visitor.visit_seq(SingleValueSeq { val: Some(self) })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option struct map seq
        enum identifier ignored_any
    }
}

impl <'de> VariantAccess<'de> for Composite {
    type Error = DeserializeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de> {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        self.deserialize_any(visitor)
    }
}

impl <'de> Deserializer<'de> for Variant {
    type Error = DeserializeError;

    // This is an enum, so treat it as such if no hints given:
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_enum(self)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        visitor.visit_enum(self)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        visitor.visit_seq(SingleValueSeq { val: Some(self) })
    }

    // Delegate to the Composite deserializing with the enum values if anything else specific is asked for:

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_unit_struct(name, visitor)

    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_unit(visitor)
    }

    fn deserialize_struct<V>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        self.values.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option identifier ignored_any
    }
}

impl <'de> EnumAccess<'de> for Variant {
    type Error = DeserializeError;

    type Variant = Composite;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de> {
        let name = self.name.into_deserializer();
        let values = self.values;
        seed
            .deserialize(name)
            .map(|name| (name, values))
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

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: de::Visitor<'de> {
        visitor.visit_seq(SingleValueSeq { val: Some(self) })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
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

struct SingleValueSeq<V> {
    val: Option<V>
}

impl <'de, V> SeqAccess<'de> for SingleValueSeq<V>
where
    V: Deserializer<'de, Error = DeserializeError>,
{
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
            // Order shouldn't matter; match on names:
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
            ("a".into(), Value::Primitive(Primitive::U8(123))),
        ]));

        assert_eq!(
            Foo::deserialize(val),
            Ok(Foo { a: 123, b: true })
        )
    }

    #[test]
    fn de_unwrapped_into_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo {
            a: u8,
            b: bool,
        }

        let val = Composite::Named(vec![
            // Order shouldn't matter; match on names:
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
            ("a".into(), Value::Primitive(Primitive::U8(123))),
        ]);

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
    fn de_unwrapped_into_tuple_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Foo(u8, bool, String);

        let val = Composite::Unnamed(vec![
            Value::Primitive(Primitive::U8(123)),
            Value::Primitive(Primitive::Bool(true)),
            Value::Primitive(Primitive::Str("hello".into())),
        ]);

        assert_eq!(
            Foo::deserialize(val),
            Ok(Foo(123, true, "hello".into()))
        )
    }

    #[test]
    fn de_into_newtype_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct FooStr(String);
        let val = Value::Primitive(Primitive::Str("hello".into()));
        assert_eq!(
            FooStr::deserialize(val),
            Ok(FooStr("hello".into()))
        );

        #[derive(Deserialize, Debug, PartialEq)]
        struct FooVecU8(Vec<u8>);
        let val = Value::Composite(Composite::Unnamed(vec![
            Value::Primitive(Primitive::U8(1)),
            Value::Primitive(Primitive::U8(2)),
            Value::Primitive(Primitive::U8(3)),
        ]));
        assert_eq!(
            FooVecU8::deserialize(val),
            Ok(FooVecU8(vec![1,2,3]))
        );

        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum { Foo(u8, u8, u8) }
        #[derive(Deserialize, Debug, PartialEq)]
        struct FooVar(MyEnum);
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::U8(1)),
                Value::Primitive(Primitive::U8(2)),
                Value::Primitive(Primitive::U8(3)),
            ])
        });
        assert_eq!(
            FooVar::deserialize(val),
            Ok(FooVar(MyEnum::Foo(1,2,3)))
        );
    }

    #[test]
    fn de_unwrapped_into_newtype_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct FooStr(String);
        let val = Primitive::Str("hello".into());
        assert_eq!(
            FooStr::deserialize(val),
            Ok(FooStr("hello".into()))
        );

        #[derive(Deserialize, Debug, PartialEq)]
        struct FooVecU8(Vec<u8>);
        let val = Composite::Unnamed(vec![
            Value::Primitive(Primitive::U8(1)),
            Value::Primitive(Primitive::U8(2)),
            Value::Primitive(Primitive::U8(3)),
        ]);
        assert_eq!(
            FooVecU8::deserialize(val),
            Ok(FooVecU8(vec![1,2,3]))
        );

        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum { Foo(u8, u8, u8) }
        #[derive(Deserialize, Debug, PartialEq)]
        struct FooVar(MyEnum);
        let val = Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::U8(1)),
                Value::Primitive(Primitive::U8(2)),
                Value::Primitive(Primitive::U8(3)),
            ])
        };
        assert_eq!(
            FooVar::deserialize(val),
            Ok(FooVar(MyEnum::Foo(1,2,3)))
        );
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
        );

        // names will just be ignored:
        let val = Value::Composite(Composite::Named(vec![
            ("a".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
        ]));
        assert_eq!(
            <(String, bool)>::deserialize(val),
            Ok(("hello".into(), true))
        );

        // Wrong number of values should fail:
        let val = Value::Composite(Composite::Unnamed(vec![
            Value::Primitive(Primitive::Str("hello".into())),
            Value::Primitive(Primitive::Bool(true)),
            Value::Primitive(Primitive::U8(123)),
        ]));
        <(String, bool)>::deserialize(val)
            .expect_err("Wrong length, should err");
    }

    #[test]
    fn de_unwrapped_into_tuple() {
        let val = Composite::Unnamed(vec![
            Value::Primitive(Primitive::Str("hello".into())),
            Value::Primitive(Primitive::Bool(true)),
        ]);
        assert_eq!(
            <(String, bool)>::deserialize(val),
            Ok(("hello".into(), true))
        );

        // names will just be ignored:
        let val = Composite::Named(vec![
            ("a".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ("b".into(), Value::Primitive(Primitive::Bool(true))),
        ]);
        assert_eq!(
            <(String, bool)>::deserialize(val),
            Ok(("hello".into(), true))
        );

        // Wrong number of values should fail:
        let val = Composite::Unnamed(vec![
            Value::Primitive(Primitive::Str("hello".into())),
            Value::Primitive(Primitive::Bool(true)),
            Value::Primitive(Primitive::U8(123)),
        ]);
        <(String, bool)>::deserialize(val)
            .expect_err("Wrong length, should err");

    }


    #[test]
    fn de_bitvec() {
        use bitvec::{ bitvec, order::Lsb0 };
        let val = Value::BitSequence(bitvec![Lsb0, u8; 0, 1, 1, 0, 1, 0, 1, 0]);

        BitSequence::deserialize(val).expect_err("We can't deserialize this yet");
    }

    #[test]
    fn de_into_tuple_variant() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum {
            Foo(String, bool, u8)
        }

        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::Str("hello".into())),
                Value::Primitive(Primitive::Bool(true)),
                Value::Primitive(Primitive::U8(123)),
            ]),
        });
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo("hello".into(), true, 123))
        );

        // it's fine to name the fields; we'll just ignore the names
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![
                ("a".into(), Value::Primitive(Primitive::Str("hello".into()))),
                ("b".into(), Value::Primitive(Primitive::Bool(true))),
                ("c".into(), Value::Primitive(Primitive::U8(123))),
            ]),
        });
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo("hello".into(), true, 123))
        );
    }

    #[test]
    fn de_unwrapped_into_tuple_variant() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum {
            Foo(String, bool, u8)
        }

        let val = Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::Str("hello".into())),
                Value::Primitive(Primitive::Bool(true)),
                Value::Primitive(Primitive::U8(123)),
            ]),
        };
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo("hello".into(), true, 123))
        );

        // it's fine to name the fields; we'll just ignore the names
        let val = Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![
                ("a".into(), Value::Primitive(Primitive::Str("hello".into()))),
                ("b".into(), Value::Primitive(Primitive::Bool(true))),
                ("c".into(), Value::Primitive(Primitive::U8(123))),
            ]),
        };
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo("hello".into(), true, 123))
        );
    }

    #[test]
    fn de_into_struct_variant() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum {
            Foo { hi: String, a: bool, b: u8 }
        }

        // If names given, order doesn't matter:
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![
                // Deliberately out of order: names should ensure alignment:
                ("b".into(), Value::Primitive(Primitive::U8(123))),
                ("a".into(), Value::Primitive(Primitive::Bool(true))),
                ("hi".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ]),
        });
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo { hi: "hello".into(), a: true, b: 123 })
        );

        // No names needed if order is OK:
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::Str("hello".into())),
                Value::Primitive(Primitive::Bool(true)),
                Value::Primitive(Primitive::U8(123)),
            ]),
        });
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo { hi: "hello".into(), a: true, b: 123 })
        );

        // Wrong order won't work if no names:
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Unnamed(vec![
                Value::Primitive(Primitive::Bool(true)),
                Value::Primitive(Primitive::U8(123)),
                Value::Primitive(Primitive::Str("hello".into())),
            ]),
        });
        MyEnum::deserialize(val).expect_err("Wrong order shouldn't work");

        // Wrong names won't work:
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![
                ("b".into(), Value::Primitive(Primitive::U8(123))),
                // Whoops; wrong name:
                ("c".into(), Value::Primitive(Primitive::Bool(true))),
                ("hi".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ]),
        });
        MyEnum::deserialize(val).expect_err("Wrong names shouldn't work");

        // Too many names is OK; we can ignore fields we don't care about:
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![
                ("foo".into(), Value::Primitive(Primitive::U8(40))),
                ("b".into(), Value::Primitive(Primitive::U8(123))),
                ("a".into(), Value::Primitive(Primitive::Bool(true))),
                ("bar".into(), Value::Primitive(Primitive::Bool(false))),
                ("hi".into(), Value::Primitive(Primitive::Str("hello".into()))),
            ]),
        });
        assert_eq!(
            MyEnum::deserialize(val),
            Ok(MyEnum::Foo { hi: "hello".into(), a: true, b: 123 })
        );
    }

    #[test]
    fn de_into_unit_variants() {
        let val = Value::Variant(Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![]),
        });
        let unwrapped_val = Variant {
            name: "Foo".into(),
            values: Composite::Named(vec![]),
        };

        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum {
            Foo
        }
        assert_eq!(
            MyEnum::deserialize(val.clone()),
            Ok(MyEnum::Foo)
        );
        assert_eq!(
            MyEnum::deserialize(unwrapped_val.clone()),
            Ok(MyEnum::Foo)
        );

        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum2 {
            Foo()
        }
        assert_eq!(
            MyEnum2::deserialize(val.clone()),
            Ok(MyEnum2::Foo())
        );
        assert_eq!(
            MyEnum2::deserialize(unwrapped_val.clone()),
            Ok(MyEnum2::Foo())
        );

        #[derive(Deserialize, Debug, PartialEq)]
        enum MyEnum3 {
            Foo{}
        }
        assert_eq!(
            MyEnum3::deserialize(val),
            Ok(MyEnum3::Foo{})
        );
        assert_eq!(
            MyEnum3::deserialize(unwrapped_val),
            Ok(MyEnum3::Foo{})
        );
    }

}