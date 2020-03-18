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

//! Stucture for registering substrate types

use crate::SetField;
use std::fmt;

pub type Address = pallet_indices::address::Address<[u8; 32], u32>;

#[derive(Debug, PartialEq, Clone)]
/// a 'stateful' Rust Type marker
/// 'Std' variant is not here like in RustTypeMarker
/// Instead common types are just apart fo the original enum
pub enum SubstrateType {

    /// 512-bit hash type
    H512(primitives::H512),
    /// 256-bit hash type
    H256(primitives::H256),

    /// Era
    Era(runtime_primitives::generic::Era),

    /// Substrate Indices Address Type
    // TODO: this is not generic for any chain that doesn't use a
    // u32 and [u8; 32] for its index/id
    Address(Address),

    /// SignedExtension Type
    SignedExtra(String),

    /// vectors, arrays, and tuples
    Composite(Vec<SubstrateType>),

    /// C-Like Enum Type
    Set(SetField),
    /// Enum
    Enum(StructUnitOrTuple),
    /// Struct Type
    Struct(Vec<StructField>),
    /// Option Type
    Option(Box<Option<SubstrateType>>),
    /// Result Type
    Result(Box<Result<SubstrateType, SubstrateType>>),

    // Std
    /// The unsigned 8-bit type
    U8(u8),
    /// unsigned 16-bit type
    U16(u16),
    /// unsigned 32-bit type
    U32(u32),
    /// unsigned 64-bit type
    U64(u64),
    /// unsigned 128-bit type
    U128(u128),
    /// unsigned cpu word-size type
    USize(usize),
    /// signed 8-bit type
    I8(i8),
    /// signed 16-bit type
    I16(i16),
    /// signed 32-bit type
    I32(i32),
    /// signed 64-bit type
    I64(i64),
    /// signed 128-bit type
    I128(i128),
    /// signed word-sized type
    ISize(isize),
    /// floating-point 32-bit type (not supported by SCALE)
    F32(f32),
    /// floating-point 64-bit type (not supported by SCALE)
    F64(f64),

    /// boolean type
    Bool(bool),
    // not sure what to do with this yet
    // may get rid of it
    Null,
}

impl fmt::Display for SubstrateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubstrateType::H512(v) => write!(f, "{}", v),
            SubstrateType::H256(v) => write!(f, "{}", v),
            SubstrateType::Era(v) => {
                match v {
                    runtime_primitives::generic::Era::Mortal(s, e) => {
                        write!(f, "Era {}..{}", s, e)
                    },
                    runtime_primitives::generic::Era::Immortal => {
                        write!(f, "Immortal Era")
                    }
                }
            },
            SubstrateType::Address(v) => {
                match v {
                    pallet_indices::address::Address::Id(ref i) => {
                        let mut s = String::from("");
                        for v in i.iter() {
                            s.push_str(&format!("{:x?}", v));
                        }
                        write!(f, "Account::Id({})", s.as_str())
                    },
                    pallet_indices::address::Address::Index(i) => {
                        write!(f, "Index: {:?}", i)
                    }
                }
            },
            SubstrateType::SignedExtra(v) => write!(f, "{}", v),
            SubstrateType::Composite(v) => {
                let mut s = String::from("");
                for v in v.iter() {
                    s.push_str(&format!("{}", v))
                }
                write!(f, "{}", s)
            }
            SubstrateType::Set(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::Enum(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::Struct(v) => {
                let mut s = String::from("");
                for val in v.iter() {
                    s.push_str(&format!("{}", val))
                }
                write!(f, "{}", s)
            },
            SubstrateType::Option(v) => {
                write!(f, "{:?}", v)
            },
            SubstrateType::Result(v) => {
                write!(f, "{:?}", v)
            },
            SubstrateType::U8(v) => {
                write!(f, "{:X}", v) // u8's print in hex format
            },
            SubstrateType::U16(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::U32(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::U64(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::U128(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::USize(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::I8(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::I16(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::I32(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::I64(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::I128(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::ISize(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::F32(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::F64(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::Bool(v) => {
                write!(f, "{}", v)
            },
            SubstrateType::Null => {
                write!(f, "Null")
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructUnitOrTuple {
    Struct(Vec<StructField>),
    Unit(String),
    /// vector of variant name -> type
    Tuple(String, Box<SubstrateType>)
}

impl fmt::Display for StructUnitOrTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut _enum = String::from("enum[ ");
        match self {
            Self::Struct(v) => {
                for val in v.iter() {
                    _enum.push_str(&format!("{}, ", val))
                }
            },
            Self::Unit(v) => {
                _enum.push_str(&format!("{}, ", v))
            },
            Self::Tuple(name, v) => _enum.push_str(&format!("{}:{}", name, v.to_string())),
        }
        _enum.push_str(" ]");
        write!(f, "{}", _enum)
    }
}

/// Type with an associated name
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    /// name of a field, if any
    /// this is an option, because IE a Tuple-enum Variant
    /// will not have named fields
    pub name: Option<String>,
    /// Type of field
    pub ty: SubstrateType,
}

impl fmt::Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct_field({:?}: {})", self.name, self.ty)
    }
}

// ============================================
// /\/\/\         CONVERSIONS            /\/\/\
// ============================================
impl From<u8> for SubstrateType {
    fn from(num: u8) -> SubstrateType {
        SubstrateType::U8(num)
    }
}

impl From<u16> for SubstrateType {
    fn from(num: u16) -> SubstrateType {
        SubstrateType::U16(num)
    }
}

impl From<u32> for SubstrateType {
    fn from(num: u32) -> SubstrateType {
        SubstrateType::U32(num)
    }
}

impl From<u64> for SubstrateType {
    fn from(num: u64) -> SubstrateType {
        SubstrateType::U64(num)
    }
}

impl From<u128> for SubstrateType {
    fn from(num: u128) -> SubstrateType {
        SubstrateType::U128(num)
    }
}

impl From<usize> for SubstrateType {
    fn from(num: usize) -> SubstrateType {
        SubstrateType::USize(num)
    }
}

impl From<i8> for SubstrateType {
    fn from(num: i8) -> SubstrateType {
        SubstrateType::I8(num)
    }
}

impl From<i16> for SubstrateType {
    fn from(num: i16) -> SubstrateType {
        SubstrateType::I16(num)
    }
}

impl From<i32> for SubstrateType {
    fn from(num: i32) -> SubstrateType {
        SubstrateType::I32(num)
    }
}

impl From<i64> for SubstrateType {
    fn from(num: i64) -> SubstrateType {
        SubstrateType::I64(num)
    }
}

impl From<i128> for SubstrateType {
    fn from(num: i128) -> SubstrateType {
        SubstrateType::I128(num)
    }
}

impl From<isize> for SubstrateType {
    fn from(num: isize) -> SubstrateType {
        SubstrateType::ISize(num)
    }
}

impl From<f32> for SubstrateType {
    fn from(num: f32) -> SubstrateType {
        SubstrateType::F32(num)
    }
}

impl From<f64> for SubstrateType {
    fn from(num: f64) -> SubstrateType {
        SubstrateType::F64(num)
    }
}

impl From<bool> for SubstrateType {
    fn from(val: bool) -> SubstrateType {
        SubstrateType::Bool(val)
    }
}
