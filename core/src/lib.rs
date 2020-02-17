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

pub mod decoder;
mod error;
#[allow(unused, dead_code)] // TODO: refactor to not need this attribute
pub mod metadata;

#[cfg(test)]
mod test_suite;

use serde::{Deserialize, Serialize};

pub trait TypeDetective {
    type Error;
    /// Get a 'Decodable' type
    fn get(
        &self,
        module: &str,
        ty: &str,
        spec: usize,
        chain: &str,
    ) -> Result<&dyn Decodable, Self::Error>;

    /// Resolve a type pointer into the type it points to
    fn resolve(
        &self,
        module: &str,
        ty: &RustTypeMarker,
    ) -> Option<&RustTypeMarker>;

    /// Resolve all type pointers until no type pointers are left within the struct
    /// preserves names of the types
    fn recursive_resolve(&self, module: &str, ty: &RustTypeMarker) -> Option<&RustTypeMarker>;
}

type TypePointer = String;
pub trait Decodable {
    /// Cast type to a referenced type pointer type
    fn as_type_pointer(&self) -> Option<&TypePointer>;
    /// Cast type to an owned type pointer type
    fn as_type_pointer_owned(&self) -> Option<TypePointer>;
    /// Cast type to a struct
    fn as_struct(&self) -> Option<&GenericStruct>;
    /// Cast type to an enum
    fn as_enum(&self) -> Option<&RustEnum>;
    /// Cast type to a set
    fn as_set(&self) -> Option<&Vec<SetField>>;
    /// Return type as reference
    fn as_type(&self) -> &RustTypeMarker;
    /// return the owned version of RustTypeMarker
    fn as_type_owned(&self) -> RustTypeMarker;
    /*
    /// Resolve a type pointer into the type it points to
    fn resolve(&self, ty: RustTypeMarker) -> RustTypeMarker;
    */

    fn is_str(&self) -> bool;
    fn is_struct(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_set(&self) -> bool;
    fn is_primitive(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: RustTypeMarker,
}

impl StructField {
    pub fn new<S: Into<String>>(name: S, ty: RustTypeMarker) -> Self {
        let name = name.into();
        Self { name, ty }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SetField {
    pub name: String,
    pub num: usize,
}

impl SetField {
    pub fn new<S: Into<String>, N: Into<u64>>(name: S, num: N) -> Self {
        let (name, num) = (name.into(), num.into());
        Self {
            name,
            num: num as usize,
        }
    }
}

type GenericStruct = Vec<StructField>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RustEnum {
    Unit(Vec<String>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
/// Definitions for common patterns seen in Substrate/Polkadot
/// type definitions
/// Definitions for Vec/Option/Compact
pub enum CommonTypes {
    /// Rust std Vec<T> type
    Vec(Box<RustTypeMarker>),
    /// Rust std Option<T> type
    Option(Box<RustTypeMarker>),
    /// Rust  Result<T, E> type
    Result(Box<RustTypeMarker>, Box<RustTypeMarker>),
    /// parity-scale-codec Compact<T> type
    Compact(Box<RustTypeMarker>),
}

impl CommonTypes {
    /// returns the inner types of Common Rust Constructs
    /// types with more than one generic (E.G Result<T, E>)
    /// are indexes in a Vector
    /// Types with only one generic (E.G Option<T>) have only
    /// one vector element
    pub fn get_inner_type(&self) -> Vec<&RustTypeMarker> {
        match self {
            CommonTypes::Vec(ref v_inner) => vec![v_inner],
            CommonTypes::Option(ref o_inner) => vec![o_inner],
            CommonTypes::Result(ref r_inner1, ref r_inner2) => vec![r_inner1, r_inner2],
            CommonTypes::Compact(ref c_inner) => vec![c_inner]
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RustTypeMarker {
    /// name of a type that exists elsewhere in type declarations
    TypePointer(String),

    /// Some Struct
    /// Field Name -> Field Type
    Struct(Vec<StructField>),

    /// A C-Like Enum
    Set(Vec<SetField>),

    /// A tuple type (max size 64)
    Tuple(Vec<RustTypeMarker>),

    /// Some Enum
    /// A Rust Enum that contains mixed "Struct" and Unit fields
    /// will have unit fields as struct but with the type as "Null"
    Enum(RustEnum),

    /// A sized array
    Array {
        /// size of the array
        size: usize,
        /// type of array
        ty: Box<RustTypeMarker>,
    },

    /// Definitions for common patterns seen in substrate/polkadot
    /// type definitions
    Std(CommonTypes),

    /// primitive unsigned 8 bit integer
    U8,
    /// primtiive unsigned 16 bit integer
    U16,
    /// primitive unsigned 32 bit integer
    U32,
    /// primitive unsigned 64 bit integer
    U64,
    /// primitive unsigned 128 bit integer
    U128,
    /// primitive unsigned word-sized integer
    USize,

    /// primitive signed 8 bit integer
    I8,
    /// primitive signed 16 bit integer
    I16,
    /// primitive signed 32 bit integer
    I32,
    /// primitive signed 64 bit integer
    I64,
    /// primitive signed 128 bit integer
    I128,
    /// primitive signed word-sized integer
    ISize,

    /// primitive IEEE-spec 32-bit floating-point number
    F32,
    /// primitive IEEE-spec 64-bit floating-point number
    F64,

    /// Boolean true/false type
    Bool,

    /// Used for fields that don't exist (ex Unit variant in an enum with both
    /// units/structs)
    Null,
}

impl Decodable for RustTypeMarker {
    fn as_type_pointer(&self) -> Option<&TypePointer> {
        match self {
            RustTypeMarker::TypePointer(s) => Some(s),
            _ => None,
        }
    }

    fn as_type_pointer_owned(&self) -> Option<TypePointer> {
        match self {
            RustTypeMarker::TypePointer(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_struct(&self) -> Option<&GenericStruct> {
        match self {
            RustTypeMarker::Struct(ref s) => Some(s),
            _ => None,
        }
    }

    fn as_enum(&self) -> Option<&RustEnum> {
        match self {
            RustTypeMarker::Enum(ref e) => Some(e),
            _ => None,
        }
    }

    fn as_set(&self) -> Option<&Vec<SetField>> {
        match self {
            RustTypeMarker::Set(ref s) => Some(s),
            _ => None,
        }
    }

    fn as_type(&self) -> &RustTypeMarker {
        self
    }

    fn as_type_owned(&self) -> RustTypeMarker {
        self.clone()
    }

    fn is_str(&self) -> bool {
        match self {
            RustTypeMarker::TypePointer(_) => true,
            _ => false,
        }
    }

    fn is_struct(&self) -> bool {
        match self {
            RustTypeMarker::Struct(_) => true,
            _ => false,
        }
    }

    fn is_enum(&self) -> bool {
        match self {
            RustTypeMarker::Enum(_) => true,
            _ => false,
        }
    }

    fn is_set(&self) -> bool {
        match self {
            RustTypeMarker::Set(_) => true,
            _ => false,
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            RustTypeMarker::U8 => true,
            RustTypeMarker::U16 => true,
            RustTypeMarker::U32 => true,
            RustTypeMarker::U64 => true,
            RustTypeMarker::U128 => true,
            RustTypeMarker::USize => true,

            RustTypeMarker::I8 => true,
            RustTypeMarker::I16 => true,
            RustTypeMarker::I32 => true,
            RustTypeMarker::I64 => true,
            RustTypeMarker::I128 => true,
            RustTypeMarker::ISize => true,

            RustTypeMarker::F32 => true,
            RustTypeMarker::F64 => true,

            RustTypeMarker::Bool => true,
            _ => false,
        }
    }
}
