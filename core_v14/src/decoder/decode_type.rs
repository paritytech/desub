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

use crate::{
    substrate_value::{
        SubstrateValue,
        CompositeValue,
        PrimitiveValue,
        SequenceValue,
        BitSequenceValue,
        VariantValue
    },
    metadata::{
        Metadata,
        Type,
        TypeId,
        TypeDef,
    }
};
use scale_info::{Field, TypeDefArray, TypeDefBitSequence, TypeDefCompact, TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant, form::PortableForm};
use codec::{ Compact, Decode };

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeTypeError {
    #[error("{0}")]
    CodecError(#[from] codec::Error),
    #[error("{0} is expected to be a valid char, but is not")]
    InvalidChar(u32),
    #[error("Cannot find type with ID {0}")]
    TypeIdNotFound(u32),
    #[error("Ran out of data during decoding")]
    Eof,
    #[error("Could not find variant with index {0} in {1:?}")]
    VariantNotFound(u8, scale_info::TypeDefVariant<PortableForm>),
    #[error("Could not decode compact encoded type into {0:?}")]
    CannotDecodeCompactIntoType(Type)
}

/// Decode data according to the [`SubstrateType`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_type(
	data: &mut &[u8],
	ty: &Type,
    metadata: &Metadata
) -> Result<SubstrateValue, DecodeTypeError> {
    match ty.type_def() {
        TypeDef::Composite(inner) => decode_composite_type(data, inner, metadata).map(SubstrateValue::Composite),
        TypeDef::Variant(inner) => decode_variant_type(data, inner, metadata).map(SubstrateValue::Variant),
        TypeDef::Sequence(inner) => decode_sequence_type(data, inner, metadata).map(SubstrateValue::Sequence),
        TypeDef::Array(inner) => decode_array_type(data, inner, metadata).map(SubstrateValue::Sequence),
        TypeDef::Tuple(inner) => decode_tuple_type(data, inner, metadata).map(SubstrateValue::Sequence),
        TypeDef::Primitive(inner) => decode_primitive_type(data, inner).map(SubstrateValue::Primitive),
        TypeDef::Compact(inner) => decode_compact_type(data, inner, metadata),
        TypeDef::BitSequence(inner) => decode_bit_sequence_type(data, inner, metadata).map(SubstrateValue::BitSequence),
    }
}

fn decode_type_by_id(
	data: &mut &[u8],
	ty_id: &TypeId,
    metadata: &Metadata
) -> Result<SubstrateValue, DecodeTypeError> {
    let inner_ty = metadata.resolve_type(ty_id)
        .ok_or(DecodeTypeError::TypeIdNotFound(ty_id.id()))?;
    decode_type(data, inner_ty, metadata)
}

fn decode_composite_type(
	data: &mut &[u8],
	ty: &TypeDefComposite<PortableForm>,
    metadata: &Metadata
) -> Result<CompositeValue, DecodeTypeError> {
    decode_fields(data, ty.fields(), metadata)
}

fn decode_variant_type(
	data: &mut &[u8],
	ty: &TypeDefVariant<PortableForm>,
    metadata: &Metadata
) -> Result<VariantValue, DecodeTypeError> {
    let index = *data.get(0).ok_or(DecodeTypeError::Eof)?;
    *data = &data[1..];

    // Does a variant exist with the index we're looking for?
    let variant = ty.variants()
        .iter()
        .find(|v| v.index() == index)
        .ok_or_else(|| DecodeTypeError::VariantNotFound(index, ty.clone()))?;

    let fields = decode_fields(data, variant.fields(), metadata)?;
    Ok(VariantValue {
        name: variant.name().clone(),
        fields
    })
}

/// Variant and Composite types both have fields; this will decode them into values.
fn decode_fields(
	data: &mut &[u8],
	fields: &[Field<PortableForm>],
    metadata: &Metadata
) -> Result<CompositeValue, DecodeTypeError> {
    let are_named = fields.iter().any(|f| f.name().is_some());
    let named_field_vals = fields
        .iter()
        .map(|f| {
            let name = f.name().cloned().unwrap_or(String::new());
            decode_type_by_id(data, f.ty(), metadata).map(|val| (name, val))
        });

    if are_named {
        let vals = named_field_vals.collect::<Result<_,_>>()?;
        Ok(CompositeValue::Named(vals))
    } else {
        let vals = named_field_vals
            .map(|r| r.map(|(_,v)| v))
            .collect::<Result<_,_>>()?;
        Ok(CompositeValue::Unnamed(vals))
    }
}

fn decode_sequence_type(
	data: &mut &[u8],
	ty: &TypeDefSequence<PortableForm>,
    metadata: &Metadata
) -> Result<SequenceValue, DecodeTypeError> {
    // We assume that the sequence is preceeded by a compact encoded length, so that
    // we know how many values to try pulling out of the data.
    let len = Compact::<u64>::decode(data)?;
    let values: Vec<_> = (0..len.0)
        .map(|_| decode_type_by_id(data, ty.type_param(), metadata))
        .collect::<Result<_,_>>()?;

    Ok(SequenceValue { values })
}

fn decode_array_type(
	data: &mut &[u8],
	ty: &TypeDefArray<PortableForm>,
    metadata: &Metadata
) -> Result<SequenceValue, DecodeTypeError> {
    // The length is known based on the type we want to decode into, so we pull out the number of items according
    // to that, and don't need a length to exist in the SCALE encoded bytes
    let values: Vec<_> = (0..ty.len())
        .map(|_| decode_type_by_id(data, ty.type_param(), metadata))
        .collect::<Result<_,_>>()?;

    Ok(SequenceValue { values })
}

fn decode_tuple_type(
	data: &mut &[u8],
	ty: &TypeDefTuple<PortableForm>,
    metadata: &Metadata
) -> Result<SequenceValue, DecodeTypeError> {
    let values: Vec<_> = ty.fields().iter()
        .map(|f| decode_type_by_id(data, f, metadata))
        .collect::<Result<_,_>>()?;

    Ok(SequenceValue { values })
}

fn decode_primitive_type(
	data: &mut &[u8],
	ty: &TypeDefPrimitive
) -> Result<PrimitiveValue, DecodeTypeError> {
    let val = match ty {
        TypeDefPrimitive::Bool => PrimitiveValue::Bool(bool::decode(data)?),
        TypeDefPrimitive::Char => {
            // [jsdw] TODO: There isn't a `char::decode`. Why? Is it wrong to use u32 or is there a more "proper" way?
            let val = u32::decode(data)?;
            PrimitiveValue::Char(char::from_u32(val).ok_or(DecodeTypeError::InvalidChar(val))?)
        },
        TypeDefPrimitive::Str => PrimitiveValue::Str(String::decode(data)?),
        TypeDefPrimitive::U8 => PrimitiveValue::U8(u8::decode(data)?),
        TypeDefPrimitive::U16 => PrimitiveValue::U16(u16::decode(data)?),
        TypeDefPrimitive::U32 => PrimitiveValue::U32(u32::decode(data)?),
        TypeDefPrimitive::U64 => PrimitiveValue::U64(u64::decode(data)?),
        TypeDefPrimitive::U128 => PrimitiveValue::U128(u128::decode(data)?),
        TypeDefPrimitive::U256 => PrimitiveValue::U256(<[u8;32]>::decode(data)?),
        TypeDefPrimitive::I8 => PrimitiveValue::I8(i8::decode(data)?),
        TypeDefPrimitive::I16 => PrimitiveValue::I16(i16::decode(data)?),
        TypeDefPrimitive::I32 => PrimitiveValue::I32(i32::decode(data)?),
        TypeDefPrimitive::I64 => PrimitiveValue::I64(i64::decode(data)?),
        TypeDefPrimitive::I128 => PrimitiveValue::I128(i128::decode(data)?),
        TypeDefPrimitive::I256 => PrimitiveValue::I256(<[u8;32]>::decode(data)?),
    };
    Ok(val)
}

fn decode_compact_type(
	data: &mut &[u8],
	ty: &TypeDefCompact<PortableForm>,
    metadata: &Metadata
) -> Result<SubstrateValue, DecodeTypeError> {
    let inner = metadata.resolve_type(ty.type_param())
        .ok_or(DecodeTypeError::TypeIdNotFound(ty.type_param().id()))?;

    use TypeDefPrimitive::*;
    let primitive_val = match inner.type_def() {
        // It's obvious how to decode basic primitive unsigned types, since we have impls for them.
        TypeDef::Primitive(U8) => PrimitiveValue::U8(Compact::<u8>::decode(data)?.0),
        TypeDef::Primitive(U16) => PrimitiveValue::U16(Compact::<u16>::decode(data)?.0),
        TypeDef::Primitive(U32) => PrimitiveValue::U32(Compact::<u32>::decode(data)?.0),
        TypeDef::Primitive(U64) => PrimitiveValue::U64(Compact::<u64>::decode(data)?.0),
        TypeDef::Primitive(U128) => PrimitiveValue::U128(Compact::<u128>::decode(data)?.0),
        // For now, we give up if we have been asked for any other type:
        _cannot_decode_from => {
            return Err(DecodeTypeError::CannotDecodeCompactIntoType(inner.clone()))
        }
    };

    Ok(SubstrateValue::Primitive(primitive_val))
}

fn decode_bit_sequence_type(
	data: &mut &[u8],
	_ty: &TypeDefBitSequence<PortableForm>,
    _metadata: &Metadata
) -> Result<BitSequenceValue, DecodeTypeError> {
    // [jsdw] TODO: might be worth checking the bit_store and bit_order types
    // and trying to work out whether they look like Lsb0 and u8, which is what
    // we assume here.
    let bit_vec: BitSequenceValue = Decode::decode(data)?;
    Ok(bit_vec)
}