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
// a 'stateful' Rust Type marker
pub enum SubstrateType {
    H512(primitives::H512),
    H256(primitives::H256),
    /// vectors, arrays, and tuples
    Composite(Vec<SubstrateType>),

    // Rust Data Primitive Types
    Set(SetField),
    UnitEnum(String),
    StructEnum(Vec<StructField>),
    Struct(Vec<StructField>),
    Array(Vec<SubstrateType>),
    Option(Box<Option<SubstrateType>>),
    Result(Box<Result<SubstrateType, SubstrateType>>),
    // need to create a 'Struct' Variant for Struct Enum and Struct Type

    // Std
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISize(isize),
    F32(f32),
    F64(f64),
    Bool(bool),
    // not sure what to do with this yet
    // may get rid of it
    Null,
}

/// Type with an associated name
#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub name: String,
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
