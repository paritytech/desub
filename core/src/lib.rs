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

pub trait TypeDetective {
    fn get(ty: &str) -> &Decodable;
}

pub trait Decodable {
    fn as_string(&self) -> String;
    fn as_str(&self) -> &str;
    fn as_generic_struct(&self) -> GenericStruct;
    fn as_primitive(&self) -> PrimitiveField;
    fn as_bytes(&self) -> Vec<u8>;
    fn as_encoded_bytes(&self) -> Vec<u8>;


    fn is_str(&self) -> bool;
    fn is_bytes(&self) -> bool;
    fn is_generic_struct(&self) -> bool;
    fn is_primitive(&self) -> bool;
}

// tuples may be represented as anonymous structs

pub struct GenericStruct {
    name: String,
    fields: Vec<StructOrPrimitive>
}

pub struct PrimitiveField {
    name: Option<String>,
    field: RustType
}

enum StructOrPrimitive {
    Struct(GenericStruct),
    Primitive(PrimitiveField)
}

pub enum RustType {
    Enum,
    Array {
        size: usize,
        ty: String
    },

    U8,
    U16,
    U32,
    U64,
    U128,
    USize,

    I8,
    I16,
    I32,
    I64,
    I128,

    F32,
    F64,
}

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
