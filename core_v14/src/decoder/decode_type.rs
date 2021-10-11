use crate::{substrate_type::SubstrateType, substrate_value::SubstrateValue};

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeTypeError {}

pub fn decode_type<'a>(
	data: &'a [u8],
	ty: &SubstrateType,
) -> Result<(SubstrateValue, &'a [u8]), (DecodeTypeError, &'a [u8])> {
	todo!()
}
