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

#[derive(Debug, PartialEq, Clone)]
/// a 'stateful' Rust Type marker
/// 'Std' variant is not here like in RustTypeMarker
/// Instead common types are just apart fo the original enum
pub enum SubstrateType {

    /// 512-bit hash type
    H512(primitives::H512),
    /// 256-bit hash type
    H256(primitives::H256),

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

#[derive(Debug, PartialEq, Clone)]
pub enum StructUnitOrTuple {
    Struct(Vec<StructField>),
    Unit(String),
    /// vector of variant name -> type
    Tuple(String, Box<SubstrateType>)
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
