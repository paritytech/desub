use crate::{
    substrate_type::{
        ArrayType,
        BitSequenceType,
        CompactType,
        CompositeType,
        PrimitiveType,
        SequenceType,
        SubstrateType,
        TupleType,
        VariantType
    },
    substrate_value::{
        SubstrateValue,
        CompositeValue,
        PrimitiveValue,
        SequenceValue,
        VariantValue
    }
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeTypeError {}

/// Decode data according to the [`SubstrateType`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_type(
	data: &mut &[u8],
	ty: &SubstrateType,
) -> Result<SubstrateValue, DecodeTypeError> {
    match ty {
        SubstrateType::Composite(inner) => decode_composite_type(data, inner).map(SubstrateValue::Composite),
        SubstrateType::Variant(inner) => decode_variant_type(data, inner).map(SubstrateValue::Variant),
        SubstrateType::Sequence(inner) => decode_sequence_type(data, inner).map(SubstrateValue::Sequence),
        SubstrateType::Array(inner) => decode_array_type(data, inner).map(SubstrateValue::Sequence),
        SubstrateType::Tuple(inner) => decode_tuple_type(data, inner).map(SubstrateValue::Sequence),
        SubstrateType::Primitive(inner) => decode_primitive_type(data, inner).map(SubstrateValue::Primitive),
        SubstrateType::Compact(inner) => decode_compact_type(data, inner),
        SubstrateType::BitSequence(inner) => decode_bit_sequence_type(data, inner).map(SubstrateValue::Sequence),
    }
}

fn decode_composite_type(
	data: &mut &[u8],
	ty: &CompositeType,
) -> Result<CompositeValue, DecodeTypeError> {
    todo!()
}

fn decode_variant_type(
	data: &mut &[u8],
	ty: &VariantType,
) -> Result<VariantValue, DecodeTypeError> {
    todo!()
}

fn decode_sequence_type(
	data: &mut &[u8],
	ty: &SequenceType,
) -> Result<SequenceValue, DecodeTypeError> {
    todo!()
}

fn decode_array_type(
	data: &mut &[u8],
	ty: &ArrayType,
) -> Result<SequenceValue, DecodeTypeError> {
    todo!()
}

fn decode_tuple_type(
	data: &mut &[u8],
	ty: &TupleType,
) -> Result<SequenceValue, DecodeTypeError> {
    todo!()
}

fn decode_primitive_type(
	data: &mut &[u8],
	ty: &PrimitiveType,
) -> Result<PrimitiveValue, DecodeTypeError> {
    todo!()
}

fn decode_compact_type(
	data: &mut &[u8],
	ty: &CompactType,
) -> Result<SubstrateValue, DecodeTypeError> {
    todo!()
}

fn decode_bit_sequence_type(
	data: &mut &[u8],
	ty: &BitSequenceType,
) -> Result<SequenceValue, DecodeTypeError> {
    todo!()
}