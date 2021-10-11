use crate::{
    substrate_value::{
        SubstrateValue,
        CompositeValue,
        PrimitiveValue,
        SequenceValue,
        VariantValue
    },
    metadata::{
        Metadata,
        Type,
        TypeId,
        TypeDef,
    }
};
use scale_info::{
    TypeDefArray,
    TypeDefBitSequence,
    TypeDefCompact,
    TypeDefComposite,
    TypeDefPrimitive,
    TypeDefSequence,
    TypeDefTuple,
    TypeDefVariant,
    form::PortableForm
};
use codec::{ Compact, Decode };

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeTypeError {
    #[error("{0}")]
    CodecError(#[from] codec::Error),
    #[error("{0} is expected to be a valid char, but is not")]
    InvalidChar(u32),
    #[error("Cannot find type with ID {0}")]
    TypeIdNotFound(u32)
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
        TypeDef::BitSequence(inner) => decode_bit_sequence_type(data, inner, metadata).map(SubstrateValue::Sequence),
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
    let are_named = ty.fields().iter().any(|f| f.name().is_some());
    let named_field_vals = ty.fields()
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

fn decode_variant_type(
	data: &mut &[u8],
	ty: &TypeDefVariant<PortableForm>,
    metadata: &Metadata
) -> Result<VariantValue, DecodeTypeError> {
    todo!()
}

fn decode_sequence_type(
	data: &mut &[u8],
	ty: &TypeDefSequence<PortableForm>,
    metadata: &Metadata
) -> Result<SequenceValue, DecodeTypeError> {
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
    todo!()
}

fn decode_bit_sequence_type(
	data: &mut &[u8],
	ty: &TypeDefBitSequence<PortableForm>,
    metadata: &Metadata
) -> Result<SequenceValue, DecodeTypeError> {
    todo!()
}