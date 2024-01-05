// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::visitor::{ Visitor, DecodeAsTypeResult };
use crate::error::Error as DecodeError;
use crate::visitor_types::{
    Array, BitSequence, Composite, Sequence, Str, Tuple,
    Variant,
};
use crate::Field;
use codec::{self, Decode};
use scale_info::Type;
use scale_info::{
    form::PortableForm, Path, PortableRegistry, TypeDef, TypeDefArray, TypeDefBitSequence,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

/// Decode data according to the type ID and [`PortableRegistry`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode_with_visitor<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: u32,
    types: &'info PortableRegistry,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    decode_with_visitor_maybe_compact(data, ty_id, types, visitor, false)
}

macro_rules! err_if_compact {
    ($is_compact:expr, $ty:expr) => {
        if $is_compact {
            return Err(DecodeError::CannotDecodeCompactIntoType($ty.clone().into()).into());
        }
    };
}

/// This is like [`decode_with_visitor`], except that if you are currently trying to decode
/// a type that we know to be compact encoded, this can be indicated by passing true to the
/// final boolean. Use [`decode_with_visitor`] by default.
pub fn decode_with_visitor_maybe_compact<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: u32,
    types: &'info PortableRegistry,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    // Provide option to "bail out" and do something custom first.
    let visitor = match visitor.unchecked_decode_as_type(data, TypeId(ty_id), types) {
        DecodeAsTypeResult::Decoded(r) => return r,
        DecodeAsTypeResult::Skipped(v) => v,
    };

    let ty = types.resolve(ty_id).ok_or(DecodeError::TypeIdNotFound(ty_id))?;
    let ty_id = TypeId(ty_id);
    let path = &ty.path;

    match &ty.type_def {
        TypeDef::Composite(inner) => {
            decode_composite_value(data, ty_id, path, inner, ty, types, visitor, is_compact)
        }
        TypeDef::Variant(inner) => {
            err_if_compact!(is_compact, ty);
            decode_variant_value(data, ty_id, path, inner, types, visitor)
        }
        TypeDef::Sequence(inner) => {
            err_if_compact!(is_compact, ty);
            decode_sequence_value(data, ty_id, inner, types, visitor)
        }
        TypeDef::Array(inner) => {
            err_if_compact!(is_compact, ty);
            decode_array_value(data, ty_id, inner, types, visitor)
        }
        TypeDef::Tuple(inner) => decode_tuple_value(data, ty_id, inner, types, visitor, is_compact),
        TypeDef::Primitive(inner) => {
            decode_primitive_value(data, ty_id, inner, visitor, is_compact)
        }
        TypeDef::Compact(inner) => {
            decode_with_visitor_maybe_compact(data, inner.type_param.id, types, visitor, true)
        }
        TypeDef::BitSequence(inner) => {
            err_if_compact!(is_compact, ty);
            decode_bit_sequence_value(data, ty_id, inner, types, visitor)
        }
    }
}

/// Note: Only `U8`, `U16`, `U32`, `U64`, `U128` allow compact encoding and should be provided for the `compact_type` argument.
#[allow(clippy::too_many_arguments)]
fn decode_composite_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    path: &'info Path<PortableForm>,
    ty: &'info TypeDefComposite<PortableForm>,
    ty_super: &'info Type<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    // guard against invalid compact types: only composites with 1 field can be compact encoded
    if is_compact && ty.fields.len() != 1 {
        return Err(DecodeError::CannotDecodeCompactIntoType(ty_super.clone()).into());
    }

    let mut fields = ty.fields.iter().map(|f| Field::new(f.ty.id, f.name.as_deref()));
    let mut items = Composite::new(data, path, &mut fields, types, is_compact);
    let res = visitor.visit_composite(&mut items, ty_id);

    // Skip over any bytes that the visitor chose not to decode:
    items.skip_decoding()?;
    *data = items.bytes_from_undecoded();

    res
}

fn decode_variant_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    path: &'info Path<PortableForm>,
    ty: &'info TypeDefVariant<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    let mut variant = Variant::new(data, path, ty, types)?;
    let res = visitor.visit_variant(&mut variant, ty_id);

    // Skip over any bytes that the visitor chose not to decode:
    variant.skip_decoding()?;
    *data = variant.bytes_from_undecoded();

    res
}

fn decode_sequence_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    ty: &'info TypeDefSequence<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    let mut items = Sequence::new(data, ty.type_param.id, types)?;
    let res = visitor.visit_sequence(&mut items, ty_id);

    // Skip over any bytes that the visitor chose not to decode:
    items.skip_decoding()?;
    *data = items.bytes_from_undecoded();

    res
}

fn decode_array_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    ty: &'info TypeDefArray<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    let len = ty.len as usize;
    let mut arr = Array::new(data, ty.type_param.id, len, types);
    let res = visitor.visit_array(&mut arr, ty_id);

    // Skip over any bytes that the visitor chose not to decode:
    arr.skip_decoding()?;
    *data = arr.bytes_from_undecoded();

    res
}

fn decode_tuple_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    ty: &'info TypeDefTuple<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    // guard against invalid compact types: only composites with 1 field can be compact encoded
    if is_compact && ty.fields.len() != 1 {
        return Err(DecodeError::CannotDecodeCompactIntoType(ty.clone().into()).into());
    }

    let mut fields = ty.fields.iter().map(|f| Field::unnamed(f.id));
    let mut items = Tuple::new(data, &mut fields, types, is_compact);
    let res = visitor.visit_tuple(&mut items, ty_id);

    // Skip over any bytes that the visitor chose not to decode:
    items.skip_decoding()?;
    *data = items.bytes_from_undecoded();

    res
}

fn decode_primitive_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    ty: &'info TypeDefPrimitive,
    visitor: V,
    is_compact: bool,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    fn decode_32_bytes<'scale>(data: &mut &'scale [u8]) -> Result<&'scale [u8; 32], DecodeError> {
        // Pull an array from the data if we can, preserving the lifetime.
        let arr: &'scale [u8; 32] = match (*data).try_into() {
            Ok(arr) => arr,
            Err(_) => return Err(DecodeError::NotEnoughInput),
        };
        // If this works out, remember to shift data 32 bytes forward.
        *data = &data[32..];
        Ok(arr)
    }
    match ty {
        TypeDefPrimitive::Bool => {
            err_if_compact!(is_compact, ty);
            let b = bool::decode(data).map_err(|e| e.into())?;
            visitor.visit_bool(b, ty_id)
        }
        TypeDefPrimitive::Char => {
            err_if_compact!(is_compact, ty);
            // Treat chars as u32's
            let val = u32::decode(data).map_err(|e| e.into())?;
            let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
            visitor.visit_char(c, ty_id)
        }
        TypeDefPrimitive::Str => {
            err_if_compact!(is_compact, ty);
            // Avoid allocating; don't decode into a String. instead, pull the bytes
            // and let the visitor decide whether to use them or not.
            let mut s = Str::new(data)?;
            // Since we aren't decoding here, shift our bytes along to after the str:
            *data = s.bytes_after();
            visitor.visit_str(&mut s, ty_id)
        }
        TypeDefPrimitive::U8 => {
            let n = if is_compact {
                codec::Compact::<u8>::decode(data).map(|c| c.0)
            } else {
                u8::decode(data)
            }
            .map_err(Into::into)?;
            visitor.visit_u8(n, ty_id)
        }
        TypeDefPrimitive::U16 => {
            let n = if is_compact {
                codec::Compact::<u16>::decode(data).map(|c| c.0)
            } else {
                u16::decode(data)
            }
            .map_err(Into::into)?;
            visitor.visit_u16(n, ty_id)
        }
        TypeDefPrimitive::U32 => {
            let n = if is_compact {
                codec::Compact::<u32>::decode(data).map(|c| c.0)
            } else {
                u32::decode(data)
            }
            .map_err(Into::into)?;
            visitor.visit_u32(n, ty_id)
        }
        TypeDefPrimitive::U64 => {
            let n = if is_compact {
                codec::Compact::<u64>::decode(data).map(|c| c.0)
            } else {
                u64::decode(data)
            }
            .map_err(Into::into)?;
            visitor.visit_u64(n, ty_id)
        }
        TypeDefPrimitive::U128 => {
            let n = if is_compact {
                codec::Compact::<u128>::decode(data).map(|c| c.0)
            } else {
                u128::decode(data)
            }
            .map_err(Into::into)?;
            visitor.visit_u128(n, ty_id)
        }
        TypeDefPrimitive::U256 => {
            err_if_compact!(is_compact, *ty);
            let arr = decode_32_bytes(data)?;
            visitor.visit_u256(arr, ty_id)
        }
        TypeDefPrimitive::I8 => {
            err_if_compact!(is_compact, ty);
            let n = i8::decode(data).map_err(|e| e.into())?;
            visitor.visit_i8(n, ty_id)
        }
        TypeDefPrimitive::I16 => {
            err_if_compact!(is_compact, ty);
            let n = i16::decode(data).map_err(|e| e.into())?;
            visitor.visit_i16(n, ty_id)
        }
        TypeDefPrimitive::I32 => {
            err_if_compact!(is_compact, ty);
            let n = i32::decode(data).map_err(|e| e.into())?;
            visitor.visit_i32(n, ty_id)
        }
        TypeDefPrimitive::I64 => {
            err_if_compact!(is_compact, ty);
            let n = i64::decode(data).map_err(|e| e.into())?;
            visitor.visit_i64(n, ty_id)
        }
        TypeDefPrimitive::I128 => {
            err_if_compact!(is_compact, ty);
            let n = i128::decode(data).map_err(|e| e.into())?;
            visitor.visit_i128(n, ty_id)
        }
        TypeDefPrimitive::I256 => {
            err_if_compact!(is_compact, ty);
            let arr = decode_32_bytes(data)?;
            visitor.visit_i256(arr, ty_id)
        }
    }
}

fn decode_bit_sequence_value<'scale, 'info, V: Visitor>(
    data: &mut &'scale [u8],
    ty_id: TypeId,
    ty: &'info TypeDefBitSequence<PortableForm>,
    types: &'info PortableRegistry,
    visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
    use scale_bits::Format;

    let format = Format::from_metadata(ty, types).map_err(DecodeError::BitSequenceError)?;
    let mut bitseq = BitSequence::new(format, data);
    let res = visitor.visit_bitsequence(&mut bitseq, ty_id);

    // Move to the bytes after the bit sequence.
    *data = bitseq.bytes_after()?;

    res
}
