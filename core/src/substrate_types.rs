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
//! Generic SubstrateType enum
//! Serialization and Deserialization Implementations (to serialize as if it were a native type)
//! Display Implementation

mod remote;

use self::remote::*;
use crate::{Error, SetField};
use primitives::crypto::AccountId32;
use primitives::crypto::{Ss58AddressFormat, Ss58Codec};
use serde::Serialize;
use std::{convert::TryFrom, fmt};

pub type Address = pallet_indices::address::Address<AccountId32, u32>;
pub type Vote = pallet_democracy::Vote;
pub type Conviction = pallet_democracy::Conviction;
pub type Data = pallet_identity::Data;

/// A 'stateful' version of [RustTypeMarker](enum.RustTypeMarker.html).
/// 'Std' variant is not here like in RustTypeMarker.
/// Instead common types are just apart of the enum
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum SubstrateType {
    /// 512-bit hash type
    H512(primitives::H512),
    /// 256-bit hash type
    H256(primitives::H256),

    /// Recursive Call Type
    Call(Vec<(String, SubstrateType)>),
    /// Era
    Era(runtime_primitives::generic::Era),

    /// Vote
    #[serde(with = "RemoteVote")]
    GenericVote(pallet_democracy::Vote),

    /// Substrate Indices Address Type
    // TODO: this is not generic for any chain that doesn't use a
    // u32 and [u8; 32] for its index/id
    #[serde(with = "RemoteAddress")]
    Address(Address),

    #[serde(with = "RemoteData")]
    Data(Data),

    /// SignedExtension Type
    SignedExtra(String),

    /// vectors, arrays, and tuples
    #[serde(serialize_with = "crate::util::as_hex")]
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
            SubstrateType::Call(c) => {
                write!(f, "CALL")?;
                for arg in c.iter() {
                    write!(f, "{}: {}", arg.0, arg.1)?;
                }
                Ok(())
            }
            SubstrateType::Era(v) => match v {
                runtime_primitives::generic::Era::Mortal(s, e) => write!(f, " Era {}..{}", s, e),
                runtime_primitives::generic::Era::Immortal => write!(f, " Immortal Era"),
            },
            SubstrateType::GenericVote(v) => write!(
                f,
                "Aye={}, Conviction={}",
                v.aye,
                v.conviction.lock_periods()
            ),
            SubstrateType::Address(v) => match v {
                pallet_indices::address::Address::Id(ref i) => write!(
                    f,
                    "Account::Id({})",
                    i.to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount)
                ),
                pallet_indices::address::Address::Index(i) => write!(f, "Index: {:?}", i),
            },
            SubstrateType::Data(d) => write!(f, "{:?}", d),
            SubstrateType::SignedExtra(v) => write!(f, "{}", v),
            SubstrateType::Composite(v) => {
                let mut s = String::from("");
                for v in v.iter() {
                    s.push_str(&format!("{}", v))
                }
                write!(f, "{}", s)
            }
            SubstrateType::Set(v) => write!(f, "{}", v),
            SubstrateType::Enum(v) => write!(f, "{}", v),
            SubstrateType::Struct(v) => {
                let mut s = String::from("");
                for val in v.iter() {
                    s.push_str(&format!("{}", val))
                }
                write!(f, "{}", s)
            }
            SubstrateType::Option(v) => write!(f, "{:?}", v),
            SubstrateType::Result(v) => write!(f, "{:?}", v),
            SubstrateType::U8(v) => {
                write!(f, "{:X}", v) // u8's print in hex format
            }
            SubstrateType::U16(v) => write!(f, "{}", v),
            SubstrateType::U32(v) => write!(f, "{}", v),
            SubstrateType::U64(v) => write!(f, "{}", v),
            SubstrateType::U128(v) => write!(f, "{}", v),
            SubstrateType::USize(v) => write!(f, "{}", v),
            SubstrateType::I8(v) => write!(f, "{}", v),
            SubstrateType::I16(v) => write!(f, "{}", v),
            SubstrateType::I32(v) => write!(f, "{}", v),
            SubstrateType::I64(v) => write!(f, "{}", v),
            SubstrateType::I128(v) => write!(f, "{}", v),
            SubstrateType::ISize(v) => write!(f, "{}", v),
            SubstrateType::F32(v) => write!(f, "{}", v),
            SubstrateType::F64(v) => write!(f, "{}", v),
            SubstrateType::Bool(v) => write!(f, "{}", v),
            SubstrateType::Null => write!(f, "Null"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum StructUnitOrTuple {
    Struct(Vec<StructField>),
    Unit(String),
    /// vector of variant name -> type
    Tuple {
        name: String,
        ty: Box<SubstrateType>,
    },
}

impl fmt::Display for StructUnitOrTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut _enum = String::from(" tuple[ ");
        match self {
            Self::Struct(v) => {
                for val in v.iter() {
                    _enum.push_str(&format!("{}, ", val))
                }
            }
            Self::Unit(v) => _enum.push_str(&format!("{}, ", v)),
            Self::Tuple { name, ty } => _enum.push_str(&format!(" {}:{} ", name, ty.to_string())),
        }
        _enum.push_str(" ]");
        write!(f, "{}", _enum)
    }
}

/// Type with an associated name
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct StructField {
    /// name of a field, if any
    /// this is an option, because IE a Tuple-enum Variant
    /// will not have named fields
    pub name: Option<String>,
    /// Type of field
    pub ty: SubstrateType,
}

impl StructField {
    pub fn new<S: Into<String>>(name: Option<S>, ty: SubstrateType) -> Self {
        let name: Option<String> = name.map(|s| s.into());
        Self { name, ty }
    }
}

impl fmt::Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct_field( {:?}: {} )", self.name, self.ty)
    }
}

// ============================================
// /\/\/\         CONVERSIONS            /\/\/\
// ============================================

impl TryFrom<&SubstrateType> for Vec<u8> {
    type Error = Error;
    fn try_from(ty: &SubstrateType) -> Result<Vec<u8>, Error> {
        match ty {
            SubstrateType::Composite(elements) => {
                if elements
                    .iter()
                    .any(|ty| !matches!(ty, SubstrateType::U8(_)))
                {
                    return Err(Error::Conversion(format!("{:?}", ty), "u8".to_string()));
                } else {
                    Ok(elements
                        .into_iter()
                        .map(|v| match v {
                            SubstrateType::U8(byte) => *byte,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<u8>>())
                }
            }
            _ => Err(Error::Conversion(format!("{}", ty), "Vec<u8>".to_string())),
        }
    }
}

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
