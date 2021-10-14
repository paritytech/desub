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
	metadata::{Type, TypeDef, TypeId},
	value::{BitSequence, Composite, Primitive, Sequence, Value, Variant},
};
use codec::{Compact, Decode};
use scale_info::{
	form::PortableForm, Field, PortableRegistry, TypeDefArray, TypeDefBitSequence, TypeDefCompact, TypeDefComposite,
	TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

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
	CannotDecodeCompactIntoType(Type),
}

/// Decode data according to the [`SubstrateType`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_type(data: &mut &[u8], ty: &Type, types: &PortableRegistry) -> Result<Value, DecodeTypeError> {
	match ty.type_def() {
		TypeDef::Composite(inner) => decode_composite_type(data, inner, types).map(Value::Composite),
		TypeDef::Variant(inner) => decode_variant_type(data, inner, types).map(Value::Variant),
		TypeDef::Sequence(inner) => decode_sequence_type(data, inner, types).map(Value::Sequence),
		TypeDef::Array(inner) => decode_array_type(data, inner, types).map(Value::Sequence),
		TypeDef::Tuple(inner) => decode_tuple_type(data, inner, types).map(Value::Sequence),
		TypeDef::Primitive(inner) => decode_primitive_type(data, inner).map(Value::Primitive),
		TypeDef::Compact(inner) => decode_compact_type(data, inner, types),
		TypeDef::BitSequence(inner) => decode_bit_sequence_type(data, inner, types).map(Value::BitSequence),
	}
}

/// Decode data according to the type ID provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_type_by_id(data: &mut &[u8], ty_id: &TypeId, types: &PortableRegistry) -> Result<Value, DecodeTypeError> {
	let inner_ty = types.resolve(ty_id.id()).ok_or(DecodeTypeError::TypeIdNotFound(ty_id.id()))?;
	decode_type(data, inner_ty, types)
}

fn decode_composite_type(
	data: &mut &[u8],
	ty: &TypeDefComposite<PortableForm>,
	types: &PortableRegistry,
) -> Result<Composite, DecodeTypeError> {
	decode_fields(data, ty.fields(), types)
}

fn decode_variant_type(
	data: &mut &[u8],
	ty: &TypeDefVariant<PortableForm>,
	types: &PortableRegistry,
) -> Result<Variant, DecodeTypeError> {
	let index = *data.get(0).ok_or(DecodeTypeError::Eof)?;
	*data = &data[1..];

	// Does a variant exist with the index we're looking for?
	let variant = ty
		.variants()
		.iter()
		.find(|v| v.index() == index)
		.ok_or_else(|| DecodeTypeError::VariantNotFound(index, ty.clone()))?;

	let fields = decode_fields(data, variant.fields(), types)?;
	Ok(Variant { name: variant.name().clone(), fields })
}

/// Variant and Composite types both have fields; this will decode them into values.
fn decode_fields(
	data: &mut &[u8],
	fields: &[Field<PortableForm>],
	types: &PortableRegistry,
) -> Result<Composite, DecodeTypeError> {
	let are_named = fields.iter().any(|f| f.name().is_some());
	let named_field_vals = fields.iter().map(|f| {
		let name = f.name().cloned().unwrap_or(String::new());
		decode_type_by_id(data, f.ty(), types).map(|val| (name, val))
	});

	if are_named {
		let vals = named_field_vals.collect::<Result<_, _>>()?;
		Ok(Composite::Named(vals))
	} else {
		let vals = named_field_vals.map(|r| r.map(|(_, v)| v)).collect::<Result<_, _>>()?;
		Ok(Composite::Unnamed(vals))
	}
}

fn decode_sequence_type(
	data: &mut &[u8],
	ty: &TypeDefSequence<PortableForm>,
	types: &PortableRegistry,
) -> Result<Sequence, DecodeTypeError> {
	// We assume that the sequence is preceeded by a compact encoded length, so that
	// we know how many values to try pulling out of the data.
	let len = Compact::<u64>::decode(data)?;
	let values: Vec<_> =
		(0..len.0).map(|_| decode_type_by_id(data, ty.type_param(), types)).collect::<Result<_, _>>()?;

	Ok(values)
}

fn decode_array_type(
	data: &mut &[u8],
	ty: &TypeDefArray<PortableForm>,
	types: &PortableRegistry,
) -> Result<Sequence, DecodeTypeError> {
	// The length is known based on the type we want to decode into, so we pull out the number of items according
	// to that, and don't need a length to exist in the SCALE encoded bytes
	let values: Vec<_> =
		(0..ty.len()).map(|_| decode_type_by_id(data, ty.type_param(), types)).collect::<Result<_, _>>()?;

	Ok(values)
}

fn decode_tuple_type(
	data: &mut &[u8],
	ty: &TypeDefTuple<PortableForm>,
	types: &PortableRegistry,
) -> Result<Sequence, DecodeTypeError> {
	let values: Vec<_> = ty.fields().iter().map(|f| decode_type_by_id(data, f, types)).collect::<Result<_, _>>()?;

	Ok(values)
}

fn decode_primitive_type(data: &mut &[u8], ty: &TypeDefPrimitive) -> Result<Primitive, DecodeTypeError> {
	let val = match ty {
		TypeDefPrimitive::Bool => Primitive::Bool(bool::decode(data)?),
		TypeDefPrimitive::Char => {
			// [jsdw] TODO: There isn't a `char::decode`. Why? Is it wrong to use u32 or is there a more "proper" way?
			let val = u32::decode(data)?;
			Primitive::Char(char::from_u32(val).ok_or(DecodeTypeError::InvalidChar(val))?)
		}
		TypeDefPrimitive::Str => Primitive::Str(String::decode(data)?),
		TypeDefPrimitive::U8 => Primitive::U8(u8::decode(data)?),
		TypeDefPrimitive::U16 => Primitive::U16(u16::decode(data)?),
		TypeDefPrimitive::U32 => Primitive::U32(u32::decode(data)?),
		TypeDefPrimitive::U64 => Primitive::U64(u64::decode(data)?),
		TypeDefPrimitive::U128 => Primitive::U128(u128::decode(data)?),
		TypeDefPrimitive::U256 => Primitive::U256(<[u8; 32]>::decode(data)?),
		TypeDefPrimitive::I8 => Primitive::I8(i8::decode(data)?),
		TypeDefPrimitive::I16 => Primitive::I16(i16::decode(data)?),
		TypeDefPrimitive::I32 => Primitive::I32(i32::decode(data)?),
		TypeDefPrimitive::I64 => Primitive::I64(i64::decode(data)?),
		TypeDefPrimitive::I128 => Primitive::I128(i128::decode(data)?),
		TypeDefPrimitive::I256 => Primitive::I256(<[u8; 32]>::decode(data)?),
	};
	Ok(val)
}

fn decode_compact_type(
	data: &mut &[u8],
	ty: &TypeDefCompact<PortableForm>,
	types: &PortableRegistry,
) -> Result<Value, DecodeTypeError> {
	fn decode_compact(data: &mut &[u8], inner: &Type, types: &PortableRegistry) -> Result<Value, DecodeTypeError> {
		use TypeDefPrimitive::*;
		let val = match inner.type_def() {
			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
			TypeDef::Primitive(U8) => Value::Primitive(Primitive::U8(Compact::<u8>::decode(data)?.0)),
			TypeDef::Primitive(U16) => Value::Primitive(Primitive::U16(Compact::<u16>::decode(data)?.0)),
			TypeDef::Primitive(U32) => Value::Primitive(Primitive::U32(Compact::<u32>::decode(data)?.0)),
			TypeDef::Primitive(U64) => Value::Primitive(Primitive::U64(Compact::<u64>::decode(data)?.0)),
			TypeDef::Primitive(U128) => Value::Primitive(Primitive::U128(Compact::<u128>::decode(data)?.0)),
			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
			TypeDef::Composite(composite) => {
				if composite.fields().len() != 1 {
					return Err(DecodeTypeError::CannotDecodeCompactIntoType(inner.clone()));
				}

				// What type is the 1 field that we are able to decode?
				let field = &composite.fields()[0];
				let field_type_id = field.ty().id();
				let inner_ty = types.resolve(field_type_id).ok_or(DecodeTypeError::TypeIdNotFound(field_type_id))?;

				// Decode this inner type via compact decoding. This can recurse, in case
				// the inner type is also a 1-field composite type.
				let inner_value = decode_compact(data, inner_ty, types)?;

				// Wrap the inner type in a representation of this outer composite type.
				let composite = match field.name() {
					Some(name) => Composite::Named(vec![(name.clone(), inner_value)]),
					None => Composite::Unnamed(vec![inner_value]),
				};

				Value::Composite(composite)
			}
			// For now, we give up if we have been asked for any other type:
			_cannot_decode_from => return Err(DecodeTypeError::CannotDecodeCompactIntoType(inner.clone())),
		};

		Ok(val)
	}

	// Pluck the inner type out and run it through our compact decoding logic.
	let inner = types.resolve(ty.type_param().id()).ok_or(DecodeTypeError::TypeIdNotFound(ty.type_param().id()))?;
	decode_compact(data, inner, types)
}

fn decode_bit_sequence_type(
	data: &mut &[u8],
	_ty: &TypeDefBitSequence<PortableForm>,
	_types: &PortableRegistry,
) -> Result<BitSequence, DecodeTypeError> {
	// [jsdw] TODO: might be worth checking the bit_store and bit_order types
	// and trying to work out whether they look like Lsb0 and u8, which is what
	// we assume here.
	let bit_vec: BitSequence = Decode::decode(data)?;
	Ok(bit_vec)
}

#[cfg(test)]
mod test {

	use super::*;
	use codec::Encode;

	/// Given a type definition, return the PortableType and PortableRegistry
	/// that our decode functions expect.
	fn make_type(ty: scale_info::Type) -> (Type, PortableRegistry) {
		use scale_info::IntoPortable;
		let mut types = scale_info::Registry::new();
		let portable_ty: Type = ty.into_portable(&mut types);
		(portable_ty, types.into())
	}

	/// Given a value to encode, and a representation of the decoded value, check that our decode functions
	/// successfully decodes the type to the expected value, based on the implicit SCALE type info that the type
	/// carries
	fn encode_decode_check<T: Encode + scale_info::TypeInfo>(val: T, ex: Value) {
		encode_decode_check_explicit_info(val, T::type_info(), ex)
	}

	/// Given a value to encode, a type to decode it back into, and a representation of
	/// the decoded value, check that our decode functions successfully decodes as expected.
	fn encode_decode_check_explicit_info<T: Encode, Ty: Into<scale_info::Type>>(val: T, ty: Ty, ex: Value) {
		let encoded = val.encode();
		let encoded = &mut &*encoded;

		let (portable_ty, portable_registry) = make_type(ty.into());

		// Can we decode?
		let val = decode_type(encoded, &portable_ty, &portable_registry).expect("decoding failed");
		// Is the decoded value what we expected?
		assert_eq!(val, ex, "decoded value does not look like what we expected");
		// Did decoding consume all of the encoded bytes, as expected?
		assert_eq!(encoded.len(), 0, "decoding did not consume all of the encoded bytes");
	}

	#[test]
	fn decode_primitives() {
		use scale_info::TypeDefPrimitive;

		encode_decode_check(true, Value::Primitive(Primitive::Bool(true)));
		encode_decode_check(false, Value::Primitive(Primitive::Bool(false)));
		encode_decode_check_explicit_info('a' as u32, TypeDefPrimitive::Char, Value::Primitive(Primitive::Char('a')));
		encode_decode_check("hello", Value::Primitive(Primitive::Str("hello".into())));
		encode_decode_check(
			"hello".to_string(), // String or &str (above) decode OK
			Value::Primitive(Primitive::Str("hello".into())),
		);
		encode_decode_check(123u8, Value::Primitive(Primitive::U8(123)));
		encode_decode_check(123u16, Value::Primitive(Primitive::U16(123)));
		encode_decode_check(123u32, Value::Primitive(Primitive::U32(123)));
		encode_decode_check(123u64, Value::Primitive(Primitive::U64(123)));
		encode_decode_check_explicit_info(
			[123u8; 32], // Anything 32 bytes long will do here
			TypeDefPrimitive::U256,
			Value::Primitive(Primitive::U256([123u8; 32])),
		);
		encode_decode_check(123i8, Value::Primitive(Primitive::I8(123)));
		encode_decode_check(123i16, Value::Primitive(Primitive::I16(123)));
		encode_decode_check(123i32, Value::Primitive(Primitive::I32(123)));
		encode_decode_check(123i64, Value::Primitive(Primitive::I64(123)));
		encode_decode_check_explicit_info(
			[123u8; 32], // Anything 32 bytes long will do here
			TypeDefPrimitive::I256,
			Value::Primitive(Primitive::I256([123u8; 32])),
		);
	}

	#[test]
	fn decode_compact_primitives() {
		encode_decode_check(Compact(123u8), Value::Primitive(Primitive::U8(123)));
		encode_decode_check(Compact(123u16), Value::Primitive(Primitive::U16(123)));
		encode_decode_check(Compact(123u32), Value::Primitive(Primitive::U32(123)));
		encode_decode_check(Compact(123u64), Value::Primitive(Primitive::U64(123)));
		encode_decode_check(Compact(123u128), Value::Primitive(Primitive::U128(123)));
	}

	#[test]
	fn decode_compact_named_wrapper_struct() {
		// A struct that can be compact encoded:
		#[derive(Encode, scale_info::TypeInfo)]
		struct MyWrapper {
			inner: u32,
		}
		impl From<Compact<MyWrapper>> for MyWrapper {
			fn from(val: Compact<MyWrapper>) -> MyWrapper {
				val.0
			}
		}
		impl codec::CompactAs for MyWrapper {
			type As = u32;

			fn encode_as(&self) -> &Self::As {
				&self.inner
			}
			fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
				Ok(MyWrapper { inner })
			}
		}

		encode_decode_check(
			Compact(MyWrapper { inner: 123 }),
			Value::Composite(Composite::Named(vec![("inner".to_string(), Value::Primitive(Primitive::U32(123)))])),
		);
	}

	#[test]
	fn decode_compact_unnamed_wrapper_struct() {
		// A struct that can be compact encoded:
		#[derive(Encode, scale_info::TypeInfo)]
		struct MyWrapper(u32);
		impl From<Compact<MyWrapper>> for MyWrapper {
			fn from(val: Compact<MyWrapper>) -> MyWrapper {
				val.0
			}
		}
		impl codec::CompactAs for MyWrapper {
			type As = u32;

			fn encode_as(&self) -> &Self::As {
				&self.0
			}
			fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
				Ok(MyWrapper(inner))
			}
		}

		encode_decode_check(
			Compact(MyWrapper(123)),
			Value::Composite(Composite::Unnamed(vec![Value::Primitive(Primitive::U32(123))])),
		);
	}

	#[test]
	fn decode_sequence_array_tuple_types() {
		encode_decode_check(
			vec![1i32, 2, 3],
			Value::Sequence(vec![
				Value::Primitive(Primitive::I32(1)),
				Value::Primitive(Primitive::I32(2)),
				Value::Primitive(Primitive::I32(3)),
			]),
		);
		encode_decode_check(
			[1i32, 2, 3], //compile-time length known
			Value::Sequence(vec![
				Value::Primitive(Primitive::I32(1)),
				Value::Primitive(Primitive::I32(2)),
				Value::Primitive(Primitive::I32(3)),
			]),
		);
		encode_decode_check(
			(1i32, true, 123456u128),
			Value::Sequence(vec![
				Value::Primitive(Primitive::I32(1)),
				Value::Primitive(Primitive::Bool(true)),
				Value::Primitive(Primitive::U128(123456)),
			]),
		);
	}

	#[test]
	fn decode_variant_types() {
		#[derive(Encode, scale_info::TypeInfo)]
		enum MyEnum {
			Foo(bool),
			Bar { hi: String, other: u128 },
		}

		encode_decode_check(
			MyEnum::Foo(true),
			Value::Variant(Variant {
				name: "Foo".to_string(),
				fields: Composite::Unnamed(vec![Value::Primitive(Primitive::Bool(true))]),
			}),
		);
		encode_decode_check(
			MyEnum::Bar { hi: "hello".to_string(), other: 123 },
			Value::Variant(Variant {
				name: "Bar".to_string(),
				fields: Composite::Named(vec![
					("hi".to_string(), Value::Primitive(Primitive::Str("hello".to_string()))),
					("other".to_string(), Value::Primitive(Primitive::U128(123))),
				]),
			}),
		);
	}

	#[test]
	fn decode_composite_types() {
		#[derive(Encode, scale_info::TypeInfo)]
		struct Unnamed(bool, String, Vec<u8>);

		#[derive(Encode, scale_info::TypeInfo)]
		struct Named {
			is_valid: bool,
			name: String,
			bytes: Vec<u8>,
		}

		encode_decode_check(
			Unnamed(true, "James".into(), vec![1, 2, 3]),
			Value::Composite(Composite::Unnamed(vec![
				Value::Primitive(Primitive::Bool(true)),
				Value::Primitive(Primitive::Str("James".to_string())),
				Value::Sequence(vec![
					Value::Primitive(Primitive::U8(1)),
					Value::Primitive(Primitive::U8(2)),
					Value::Primitive(Primitive::U8(3)),
				]),
			])),
		);
		encode_decode_check(
			Named { is_valid: true, name: "James".into(), bytes: vec![1, 2, 3] },
			Value::Composite(Composite::Named(vec![
				("is_valid".into(), Value::Primitive(Primitive::Bool(true))),
				("name".into(), Value::Primitive(Primitive::Str("James".to_string()))),
				(
					"bytes".into(),
					Value::Sequence(vec![
						Value::Primitive(Primitive::U8(1)),
						Value::Primitive(Primitive::U8(2)),
						Value::Primitive(Primitive::U8(3)),
					]),
				),
			])),
		);
	}

	#[test]
	fn decode_bit_sequence() {
		use bitvec::{bitvec, order::Lsb0};

		encode_decode_check(
			bitvec![Lsb0, u8; 0, 1, 1, 0, 1, 0],
			Value::BitSequence(bitvec![Lsb0, u8; 0, 1, 1, 0, 1, 0]),
		);
	}
}
