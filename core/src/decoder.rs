// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version. //
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

//! A serializable/deserializable Decoder used to encode/decode substrate types
//! from compact SCALE encoded byte arrays
//! with special attention paid to generic types in runtime module trait
//! definitions if serialized, can be deserialized. This allows for portability
//! by not needing to import differently-versioned runtimes
//! as long as all the types of the runtime are registered within the decoder
//!
//! Theoretically, one could upload the deserialized decoder JSON to distribute
//! to different applications that need the type data

mod extrinsics;
mod metadata;

pub use self::extrinsics::{ExtrinsicArgument, GenericCall, GenericExtrinsic};
#[cfg(test)]
pub use self::metadata::test_suite;
pub use self::metadata::{Metadata, MetadataError, ModuleIndex};
use crate::{
    error::Error,
    substrate_types::{self, StructField, StructUnitOrTuple, SubstrateType},
    CommonTypes, RustTypeMarker, TypeDetective,
};
use codec::{Compact, CompactLen, Decode};
// use serde::Serialize;
use std::{collections::HashMap, convert::TryFrom};

type SpecVersion = u32;
/// Decoder for substrate types
///
/// hold information about the Runtime Metadata
/// and maps types inside the runtime metadata to self-describing types in
/// type-metadata
#[derive(Default, Debug)]
pub struct Decoder<T: TypeDetective> {
    // reference to an item in 'versions' vector
    // NOTE: possibly a concurrent HashMap
    versions: HashMap<SpecVersion, Metadata>,
    types: T,
    chain: String,
}

/// The type of Entry
///
/// # Note
///
/// not entirely sure if necessary as of yet
/// however, used for the purpose for narrowing down the context a type is being
/// used in
#[derive(Debug)]
pub enum Entry {
    Call,
    Storage,
    Event,
    Constant,
}

impl<T> Decoder<T>
where
    T: TypeDetective,
{
    /// Create new Decoder with specified types
    pub fn new(types: T, chain: &str) -> Self {
        Self {
            versions: HashMap::default(),
            types,
            chain: chain.to_string(),
        }
    }

    /// Insert a Metadata with Version attached
    /// If version exists, it's corresponding metadata will be updated
    pub fn register_version(&mut self, version: SpecVersion, metadata: Metadata) {
        self.versions.insert(version, metadata);
    }

    /// internal api to get runtime version
    /// panics if a version is not found
    ///
    /// get runtime version in less than linear time with binary search
    ///
    /// # Note
    /// Returns None if version is nonexistant
    pub fn get_version_metadata(&self, version: SpecVersion) -> Option<&Metadata> {
        self.versions.get(&version)
    }

    /// Decode an extrinsic
    pub fn decode_extrinsic(
        &self,
        spec: SpecVersion,
        data: &[u8],
    ) -> Result<GenericExtrinsic, Error> {
        let meta = self.versions.get(&spec).expect("Spec does not exist");

        // first byte -> vector length
        // second byte -> extrinsic version
        // third byte -> Outer enum index
        // fourth byte -> inner enum index (function index)
        // can check if signed via a simple & too
        let length = Self::scale_length(data)?;
        let mut cursor: usize = length.1;
        let version = data[cursor];
        let is_signed = version & 0b1000_0000 != 0;
        let version = version & 0b0111_1111;
        log::debug!("Extrinsic Version: {}", version);
        // the second byte will be the index of the call enum
        cursor += 1;

        // TODO: split into decode_signature
        let signature: Option<_> = if is_signed {
            // cursor += 1;
            log::debug!("SIGNED EXTRINSIC");
            let signature = self
                .types
                .get_extrinsic_ty(spec, self.chain.as_str(), "signature")
                .expect("Signature must not be empty")
                .as_type();
            log::debug!("TYPE: {:?}", signature);
            Some(self.decode_single(
                "runtime",
                spec,
                signature,
                data,
                &mut cursor,
                false,
            )?)
        } else {
            None
        };
        if let Some(s) = &signature {
            log::debug!("Signature: \n{}", s);
            log::debug!("End Signature");
        }
        log::debug!("data = {:?}", &data[cursor..]);
        log::debug!("cursor = {}", cursor);
        let module = meta.module_by_index(ModuleIndex::Call(data[cursor]))?;
        cursor += 1;
        log::debug!("cursor = {}", cursor);
        let call_meta = module.call(data[cursor])?;
        cursor += 1;
        log::debug!("cursor = {}", cursor);
        log::debug!("data = {:X?}", &data[cursor..]);

        // TODO: tuple of argument name -> value
        let mut types: Vec<(String, SubstrateType)> = Vec::new();
        for arg in call_meta.arguments() {
            log::trace!("arg = {:?}", arg);
            let val = self.decode_single(
                module.name(),
                spec,
                &arg.ty,
                data,
                &mut cursor,
                false,
            )?;
            types.push((arg.name.to_string(), val));
        }
        // log::debug!("{:?}", &data[cursor]);
        Ok(GenericExtrinsic::new(
            signature,
            types,
            call_meta.name(),
            module.name().into(),
        ))
    }

    /// Internal function to handle
    /// decoding of a single rust type marker
    /// from data and the curent position within the data
    ///
    /// # Panics
    /// panics if a type cannot be decoded
    fn decode_single(
        &self,
        module: &str,
        spec: SpecVersion,
        ty: &RustTypeMarker,
        data: &[u8],
        cursor: &mut usize,
        is_compact: bool,
    ) -> Result<SubstrateType, Error> {
        let ty = match ty {
            RustTypeMarker::TypePointer(v) => {
                if let Some(t) = self.decode_sub_type(spec, v, data, cursor, is_compact) {
                    t
                } else {
                    let new_type = self
                        .types
                        .get(module, v, spec, self.chain.as_str())
                        .ok_or_else(|| Error::from("Name Resolution Failure"))?
                        .as_type();
                    self.decode_single(module, spec, new_type, data, cursor, is_compact)?
                }
            }
            RustTypeMarker::Struct(v) => {
                let ty =
                    self.decode_structlike(v, module, spec, data, cursor, is_compact)?;
                SubstrateType::Struct(ty)
            }
            // TODO: test
            RustTypeMarker::Set(v) => {
                // a set item must be an u8
                // can decode this right away
                let index = data[*cursor];
                *cursor += 1;
                SubstrateType::Set(v[index as usize].clone())
            }
            RustTypeMarker::Tuple(v) => {
                let ty = v
                    .iter()
                    .map(|v| {
                        self.decode_single(module, spec, &v, data, cursor, is_compact)
                    })
                    .collect::<Result<Vec<SubstrateType>, Error>>();
                SubstrateType::Composite(ty?)
            }
            RustTypeMarker::Enum(v) => {
                let index = data[*cursor];
                *cursor += 1;
                log::debug!("HERE");
                log::debug!("data = {:?}", data[*cursor]);
                let variant = &v[index as usize];
                log::debug!("Don't get here");
                match &variant.ty {
                    crate::StructUnitOrTuple::Struct(ref v) => {
                        let ty = self.decode_structlike(
                            v, module, spec, data, cursor, is_compact,
                        )?;
                        SubstrateType::Enum(StructUnitOrTuple::Struct(ty))
                    }
                    crate::StructUnitOrTuple::Unit(v) => {
                        SubstrateType::Enum(StructUnitOrTuple::Unit(v.clone()))
                    }
                    crate::StructUnitOrTuple::Tuple(ref v) => {
                        let ty = self
                            .decode_single(module, spec, v, data, cursor, is_compact)?;
                        let name = variant
                            .variant_name
                            .as_ref()
                            .expect("Tuple Variant must have a name")
                            .clone();
                        SubstrateType::Enum(StructUnitOrTuple::Tuple(name, Box::new(ty)))
                    }
                }
            }
            RustTypeMarker::Array { size, ty } => {
                let mut decoded_arr = Vec::with_capacity(*size);
                if *size == 0 as usize {
                    log::trace!("Returning Empty Vector");
                    return Ok(SubstrateType::Composite(Vec::new()));
                } else {
                    for _ in 0..*size {
                        decoded_arr.push(
                            self.decode_single(
                                module, spec, ty, &data, cursor, is_compact,
                            )?,
                        )
                    }
                }
                // rely on cursor increments in sub-types (U32/substrate specific types)
                SubstrateType::Composite(decoded_arr)
            }
            RustTypeMarker::Std(v) => match v {
                CommonTypes::Vec(v) => {
                    let length = Self::scale_length(&data[*cursor..])?;
                    *cursor += length.1;
                    // we can just decode this as an "array" now
                    self.decode_single(
                        module,
                        spec,
                        &RustTypeMarker::Array {
                            size: length.0,
                            ty: v.clone(),
                        },
                        data,
                        cursor,
                        is_compact,
                    )?
                }
                CommonTypes::Option(v) => {
                    match data[*cursor] {
                        // None
                        0x00 => {
                            *cursor += 1;
                            SubstrateType::Option(Box::new(None))
                        }
                        // Some
                        0x01 => {
                            *cursor += 1;
                            let ty = self.decode_single(
                                module, spec, v, data, cursor, is_compact,
                            )?;
                            SubstrateType::Option(Box::new(Some(ty)))
                        }
                        _ => {
                            panic!("Cannot deduce correct Option<T> enum variant");
                        }
                    }
                }
                CommonTypes::Result(v, e) => {
                    match data[*cursor] {
                        // Ok
                        0x00 => {
                            *cursor += 1;
                            let ty = self.decode_single(
                                module, spec, v, data, cursor, is_compact,
                            )?;
                            SubstrateType::Result(Box::new(Ok(ty)))
                        }
                        // Err
                        0x01 => {
                            *cursor += 1;
                            let ty = self.decode_single(
                                module, spec, e, data, cursor, is_compact,
                            )?;
                            SubstrateType::Result(Box::new(Err(ty)))
                        }
                        _ => {
                            panic!("Cannot deduce correct Result<T> Enum Variant");
                        }
                    }
                }
                // TODO: test
                CommonTypes::Compact(v) => {
                    self.decode_single(module, spec, v, data, cursor, true)?
                }
            },
            RustTypeMarker::U8 => {
                let num: u8 = if is_compact {
                    let num: Compact<u8> = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += Compact::compact_len(&u8::from(num));
                    num.into()
                } else {
                    let num: u8 = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += 1;
                    num
                };
                num.into()
            }
            RustTypeMarker::U16 => {
                let num: u16 = if is_compact {
                    let num: Compact<u16> = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += Compact::compact_len(&u16::from(num));
                    num.into()
                } else {
                    let num: u16 = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += 2;
                    num
                };
                num.into()
            }
            RustTypeMarker::U32 => {
                let num: u32 = if is_compact {
                    let num: Compact<u32> = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += Compact::compact_len(&u32::from(num));
                    num.into()
                } else {
                    let num: u32 = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += 4;
                    num
                };
                num.into()
            }
            RustTypeMarker::U64 => {
                let num: u64 = if is_compact {
                    let num: Compact<u64> = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += Compact::compact_len(&u64::from(num));
                    num.into()
                } else {
                    let num: u64 = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += 8;
                    num
                };
                num.into()
            }
            RustTypeMarker::U128 => {
                let num: u128 = if is_compact {
                    let num: Compact<u128> = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += Compact::compact_len(&u128::from(num));
                    num.into()
                } else {
                    let num: u128 = Decode::decode(&mut &data[*cursor..])?;
                    *cursor += 16;
                    num
                };
                num.into()
            }
            RustTypeMarker::USize => {
                panic!("usize decoding not possible!")
                /* let size = std::mem::size_of::<usize>();
                let num: usize =
                    Decode::decode(&mut &data[*cursor..=*cursor+size])?;
                *cursor += std::mem::size_of::<usize>();
                num.into()
                 */
            }
            RustTypeMarker::I8 => {
                let num: i8 = if is_compact {
                    unimplemented!()
                } else {
                    Decode::decode(&mut &data[*cursor..])?
                };
                *cursor += 1;
                num.into()
            }
            RustTypeMarker::I16 => {
                let num: i16 = if is_compact {
                    unimplemented!()
                } else {
                    Decode::decode(&mut &data[*cursor..])?
                };
                *cursor += 2;
                num.into()
            }
            RustTypeMarker::I32 => {
                let num: i32 = if is_compact {
                    unimplemented!()
                } else {
                    Decode::decode(&mut &data[*cursor..])?
                };
                *cursor += 4;
                num.into()
            }
            RustTypeMarker::I64 => {
                let num: i64 = if is_compact {
                    // let num: Compact<i64> = Decode::decode(&mut &data[*cursor..*cursor+8])?;
                    // num.into()
                    unimplemented!()
                } else {
                    Decode::decode(&mut &data[*cursor..])?
                };
                *cursor += 8;
                num.into()
            }
            RustTypeMarker::I128 => {
                let num: i128 = if is_compact {
                    unimplemented!()
                } else {
                    Decode::decode(&mut &data[*cursor..])?
                };
                *cursor += 16;
                num.into()
            }
            RustTypeMarker::ISize => {
                panic!("isize decoding impossible!")
                /*
                let idx = std::mem::size_of::<isize>();
                let num: isize =
                    Decode::decode(&mut &data[*cursor..=*cursor + idx])?;
                *cursor += std::mem::size_of::<isize>();
                num.into()
                */
            }
            RustTypeMarker::F32 => {
                /*
                let num: f32 = Decode::decode(&mut &data[*cursor..=*cursor + 4])?;
                *cursor += 5;
                num.into()
                 */
                panic!("f32 decoding impossible!");
            }
            RustTypeMarker::F64 => {
                /*
                let num: f64 = Decode::decode(&mut &data[*cursor..=*cursor + 8])?;
                *cursor += 9;
                num.into()
                 */
                panic!("f64 decoding impossible!");
            }
            RustTypeMarker::String => unimplemented!(),
            RustTypeMarker::Bool => {
                let boo: bool = Decode::decode(&mut &data[*cursor..=*cursor])?;
                *cursor += 1;
                //   . - .
                //  ( o o )
                //  |  0  \
                //   \     \
                //    `~~~~~' boo!
                boo.into()
            }
            RustTypeMarker::Null => SubstrateType::Null,
        };
        Ok(ty)
    }

    /// internal API to decode substrate type
    /// Tries to decode a type that is native to substrate
    /// for example, H256. Returns none if type cannot be deduced
    /// Supported types:
    /// - H256
    /// - H512
    // TODO: test this with the substrate types used
    fn decode_sub_type(
        &self,
        _spec: SpecVersion,
        ty: &str,
        data: &[u8],
        cursor: &mut usize,
        _is_compact: bool,
    ) -> Option<SubstrateType> {
        match ty {
            "Lookup" => {
                let inc: usize;
                // TODO: requires more investigation
                // cursor increments for 0x00 .. 0xfe may be incorrect
                match data[*cursor] {
                    0x00..=0xef => {
                        inc = 1;
                    }
                    0xfc => {
                        inc = 2;
                    }
                    0xfd => {
                        inc = 4;
                    }
                    0xfe => {
                        inc = 4;
                    }
                    0xff => {
                        inc = 32;
                    }
                    _ => {
                        log::error!("Invalid Address");
                        return None;
                    }
                };
                let val: substrate_types::Address =
                    Decode::decode(&mut &data[*cursor..]).ok()?;

                *cursor += inc + 1; // +1 for byte 0x00-0xff
                Some(SubstrateType::Address(val))
            }
            "Era" => {
                let val: runtime_primitives::generic::Era =
                    Decode::decode(&mut &data[*cursor..]).ok()?;
                match val {
                    // although phase and period are both u64, era is Encoded
                    // in only two bytes
                    runtime_primitives::generic::Era::Immortal => *cursor += 1,
                    runtime_primitives::generic::Era::Mortal(_, _) => *cursor += 2,
                };
                Some(SubstrateType::Era(val))
            }
            "H256" => {
                let val: primitives::H256 = Decode::decode(&mut &data[*cursor..]).ok()?;
                *cursor += 32;
                Some(SubstrateType::H256(val))
            }
            "H512" => {
                let val: primitives::H512 = Decode::decode(&mut &data[*cursor..]).ok()?;
                *cursor += 64;
                Some(SubstrateType::H512(val))
            }
            _ => None,
        }
    }

    /// internal api to get the number of items in a encoded series
    /// returns a tuple of (number_of_items, length_of_prefix)
    /// length of prefix is the length in bytes that the prefix took up
    /// in the encoded data
    fn scale_length(mut data: &[u8]) -> Result<(usize, usize), Error> {
        // alternative to `DecodeLength` trait, to avoid casting from a trait
        let u32_length = u32::from(Compact::<u32>::decode(&mut data)?);
        let length_of_prefix: usize = Compact::compact_len(&u32_length);
        let usize_length = usize::try_from(u32_length)
            .map_err(|_| Error::from("Failed convert decoded size into usize."))?;
        Ok((usize_length, length_of_prefix))
    }

    /// internal api to decode a vector of struct IdentityFields
    /// avoids code duplications when dealing with structfields in structs/enums
    fn decode_structlike(
        &self,
        fields: &[crate::StructField],
        module: &str,
        spec: SpecVersion,
        data: &[u8],
        cursor: &mut usize,
        is_compact: bool,
    ) -> Result<Vec<StructField>, Error> {
        fields
            .iter()
            .map(|field| {
                let ty = self
                    .decode_single(module, spec, &field.ty, data, cursor, is_compact)?;
                Ok(StructField {
                    name: Some(field.name.clone()),
                    ty,
                })
            })
            .collect::<Result<Vec<StructField>, Error>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        decoder::metadata::test_suite as meta_test_suite, substrate_types::StructField,
        test_suite, Decodable, EnumField,
    };
    use codec::Encode;

    struct GenericTypes;
    impl TypeDetective for GenericTypes {
        fn get(
            &self,
            _module: &str,
            _ty: &str,
            _spec: u32,
            _chain: &str,
        ) -> Option<&dyn Decodable> {
            Some(&RustTypeMarker::I128)
        }

        fn get_extrinsic_ty(
            &self,
            spec: u32,
            chain: &str,
            ty: &str,
        ) -> Option<&dyn Decodable> {
            None
        }

        fn resolve(
            &self,
            _module: &str,
            _ty: &RustTypeMarker,
        ) -> Option<&RustTypeMarker> {
            Some(&RustTypeMarker::I128)
        }
    }

    #[test]
    fn should_insert_metadata() {
        // let types = PolkadotTypes::new().unwrap();
        // types.get("balances", "BalanceLock", 1042, "kusama");

        let mut decoder = Decoder::new(GenericTypes, "kusama");
        decoder.register_version(
            test_suite::mock_runtime(0).spec_version,
            meta_test_suite::test_metadata(),
        );
        decoder.register_version(
            test_suite::mock_runtime(1).spec_version,
            meta_test_suite::test_metadata(),
        );
        decoder.register_version(
            test_suite::mock_runtime(2).spec_version,
            meta_test_suite::test_metadata(),
        );
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(0).spec_version));
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(1).spec_version));
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(2).spec_version))
    }

    #[test]
    fn should_get_version_metadata() {
        // let types = PolkadotTypes::new().unwrap();
        let mut decoder = Decoder::new(GenericTypes, "kusama");
        let rt_version = test_suite::mock_runtime(0);
        let meta = meta_test_suite::test_metadata();
        decoder.register_version(rt_version.spec_version.clone(), meta.clone());
        let _other_meta = decoder.get_version_metadata(rt_version.spec_version);
        assert_eq!(Some(&meta), _other_meta.clone())
    }

    #[test]
    fn should_get_scale_length() {
        let encoded = vec![32, 4].encode();
        for v in encoded.iter() {
            print!("{:08b} ", v);
        }
        let len = Decoder::<GenericTypes>::scale_length(encoded.as_slice()).unwrap();
        assert_eq!(len.0, 2);
    }

    macro_rules! decode_test {
        ( $v: expr, $x:expr, $r: expr) => {{
            let val = $v.encode();
            let decoder = Decoder::new(GenericTypes, "kusama");
            let res = decoder
                .decode_single("", 1031, &$x, val.as_slice(), &mut 0, false)
                .unwrap();

            assert_eq!($r, res)
        }};
    }

    #[test]
    fn should_decode_option() {
        let val: Option<u32> = Some(0x1337);
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Option(Box::new(RustTypeMarker::U32))),
            SubstrateType::Option(Box::new(Some(SubstrateType::U32(0x1337))))
        );
        let val: Option<u32> = None;
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Option(Box::new(RustTypeMarker::U32))),
            SubstrateType::Option(Box::new(None))
        );
    }

    #[test]
    fn should_decode_result() {
        let val: Result<u32, u32> = Ok(0x1337);
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Result(
                Box::new(RustTypeMarker::U32),
                Box::new(RustTypeMarker::U32)
            )),
            SubstrateType::Result(Box::new(Ok(SubstrateType::U32(0x1337))))
        );

        let val: Result<u32, u32> = Err(0x1337);
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Result(
                Box::new(RustTypeMarker::U32),
                Box::new(RustTypeMarker::U32),
            )),
            SubstrateType::Result(Box::new(Err(SubstrateType::U32(0x1337))))
        );
    }

    #[test]
    fn should_decode_vector() {
        let val: Vec<u32> = vec![12, 32, 0x1337, 62];
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U32))),
            SubstrateType::Composite(vec![
                SubstrateType::U32(12),
                SubstrateType::U32(32),
                SubstrateType::U32(0x1337),
                SubstrateType::U32(62)
            ])
        );

        let val: Vec<u128> = vec![12, 32, 0x1337, 62];
        decode_test!(
            val,
            RustTypeMarker::Std(CommonTypes::Vec(Box::new(RustTypeMarker::U128))),
            SubstrateType::Composite(vec![
                SubstrateType::U128(12),
                SubstrateType::U128(32),
                SubstrateType::U128(0x1337),
                SubstrateType::U128(62)
            ])
        );
    }

    #[test]
    fn should_decode_array() {
        let val: [u32; 4] = [12, 32, 0x1337, 62];
        decode_test!(
            val,
            RustTypeMarker::Array {
                size: 4,
                ty: Box::new(RustTypeMarker::U32),
            },
            SubstrateType::Composite(vec![
                SubstrateType::U32(12),
                SubstrateType::U32(32),
                SubstrateType::U32(0x1337),
                SubstrateType::U32(62)
            ])
        )
    }

    #[test]
    fn should_decode_struct() {
        #[derive(Encode, Decode)]
        struct ToDecode {
            foo: u32,
            name: Vec<u8>,
        }
        let val = ToDecode {
            foo: 0x1337,
            name: vec![8, 16, 30, 40],
        };
        decode_test!(
            val,
            RustTypeMarker::Struct(vec![
                crate::StructField {
                    name: "foo".to_string(),
                    ty: RustTypeMarker::U32,
                },
                crate::StructField {
                    name: "name".to_string(),
                    ty: RustTypeMarker::Std(CommonTypes::Vec(Box::new(
                        RustTypeMarker::U8,
                    ))),
                },
            ]),
            SubstrateType::Struct(vec![
                StructField {
                    name: Some("foo".to_string()),
                    ty: SubstrateType::U32(0x1337)
                },
                StructField {
                    name: Some("name".to_string()),
                    ty: SubstrateType::Composite(vec![
                        SubstrateType::U8(8),
                        SubstrateType::U8(16),
                        SubstrateType::U8(30),
                        SubstrateType::U8(40)
                    ])
                }
            ])
        );
    }

    #[test]
    fn should_decode_tuple() {
        let val: (u32, u32, u32, u32) = (18, 32, 42, 0x1337);
        decode_test!(
            val,
            RustTypeMarker::Tuple(vec![
                RustTypeMarker::U32,
                RustTypeMarker::U32,
                RustTypeMarker::U32,
                RustTypeMarker::U32,
            ]),
            SubstrateType::Composite(vec![
                SubstrateType::U32(18),
                SubstrateType::U32(32),
                SubstrateType::U32(42),
                SubstrateType::U32(0x1337)
            ])
        )
    }

    #[test]
    fn should_decode_unit_enum() {
        #[derive(Encode, Decode)]
        enum Foo {
            Zoo,
            Wraith,
            Spree,
        }
        let val = Foo::Wraith;
        decode_test!(
            val,
            RustTypeMarker::Enum(vec![
                EnumField::new(None, crate::StructUnitOrTuple::Unit("Zoo".into())),
                EnumField::new(None, crate::StructUnitOrTuple::Unit("Wraith".into())),
                EnumField::new(None, crate::StructUnitOrTuple::Unit("Spree".into())),
            ]),
            SubstrateType::Enum(StructUnitOrTuple::Unit("Wraith".into()))
        );
    }

    #[test]
    fn should_decode_tuple_enum() {
        #[derive(Encode, Decode)]
        struct TestStruct(i128);

        #[derive(Encode, Decode)]
        enum Foo {
            Zoo(TestStruct),
            Wraith(TestStruct),
        }
        let val = Foo::Wraith(TestStruct(0x1337));
        decode_test!(
            val,
            RustTypeMarker::Enum(vec![
                EnumField::new(
                    Some("Zoo".into()),
                    crate::StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                        "TestStruct".into(),
                    )),
                ),
                EnumField::new(
                    Some("Wraith".into()),
                    crate::StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                        "TestStruct".into(),
                    )),
                ),
            ]),
            SubstrateType::Enum(StructUnitOrTuple::Tuple(
                "Wraith".into(),
                Box::new(SubstrateType::I128(0x1337))
            ))
        );
    }

    #[test]
    fn should_decode_structlike_enum() {
        #[derive(Encode, Decode)]
        enum Foo {
            Zoo { name: Vec<u8>, id: u32 },
            Wraith { name: Vec<u16>, id: u64 },
        }
        let val = Foo::Wraith {
            name: vec![0x13, 0x37],
            id: 15,
        };

        decode_test!(
            val,
            RustTypeMarker::Enum(vec![
                EnumField::new(
                    Some("Zoo".into()),
                    crate::StructUnitOrTuple::Struct(vec![
                        crate::StructField::new(
                            "name",
                            RustTypeMarker::Std(CommonTypes::Vec(Box::new(
                                RustTypeMarker::U8,
                            ))),
                        ),
                        crate::StructField::new("id", RustTypeMarker::U32),
                    ]),
                ),
                EnumField::new(
                    Some("Wraith".into()),
                    crate::StructUnitOrTuple::Struct(vec![
                        crate::StructField::new(
                            "name",
                            RustTypeMarker::Std(CommonTypes::Vec(Box::new(
                                RustTypeMarker::U16,
                            ))),
                        ),
                        crate::StructField::new("id", RustTypeMarker::U64),
                    ]),
                ),
            ]),
            SubstrateType::Enum(StructUnitOrTuple::Struct(vec![
                StructField {
                    name: Some("name".into()),
                    ty: SubstrateType::Composite(vec![
                        SubstrateType::U16(0x13),
                        SubstrateType::U16(0x37)
                    ])
                },
                StructField {
                    name: Some("id".into()),
                    ty: SubstrateType::U64(15)
                }
            ]))
        );
    }
}
