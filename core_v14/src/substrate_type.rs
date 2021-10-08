use scale_info::PortableRegistry;
use std::fmt::{ Debug };
use crate::util::{ for_each_between, ForEachBetween };

/// The portable form of a scale info type definition, as obtained from metadata.
type ScaleInfoTypeDef = scale_info::TypeDef<scale_info::form::PortableForm>;

/// The portable form of a [`scale_info::Type`], as obtained from metadata.
type ScaleInfoType = scale_info::Type<scale_info::form::PortableForm>;

/// This roughly mirrors [`scale_info::TypeDef`], but is slightly simplified
/// and fully public, so that it's possible to construct types ourselves.
pub enum SubstrateType {
    Composite(CompositeType),
    Variant(VariantType),
    Sequence(SequenceType),
    Array(ArrayType),
    Tuple(TupleType),
    Primitive(PrimitiveType),
    Compact(CompactType),
    BitSequence(BitSequenceType),
}

impl Debug for SubstrateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstrateType::Composite(item) => Debug::fmt(item, f),
            SubstrateType::Variant(item) => Debug::fmt(item, f),
            SubstrateType::Sequence(item) => Debug::fmt(item, f),
            SubstrateType::Array(item) => Debug::fmt(item, f),
            SubstrateType::Tuple(item) => Debug::fmt(item, f),
            SubstrateType::Primitive(item) => Debug::fmt(item, f),
            SubstrateType::Compact(item) => Debug::fmt(item, f),
            SubstrateType::BitSequence(item) => Debug::fmt(item, f),
        }
    }
}

/// Named or unnamed struct or variant fields.
pub struct CompositeType {
    /// The name of the type. If the type is a Struct, this will
    /// be the path to it, and if it's a Variant type it'll be the
    /// variant name.
    pub name: String,
    /// The structure of the fields of this type.
    pub fields: CompositeTypeFields,
}

pub enum CompositeTypeFields {
    /// Eg `{ foo: u32, bar: bool }`
    Named(Vec<(String, SubstrateType)>),
    /// Eg `(u32, bool)`
    Unnamed(Vec<SubstrateType>),
}

impl Debug for CompositeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fields {
            CompositeTypeFields::Named(fields) => {
                let struc = f.debug_struct(&self.name);
                for (name, ty) in &fields {
                    struc.field(name, ty);
                }
                struc.finish()
            },
            CompositeTypeFields::Unnamed(fields) => {
                let struc = f.debug_tuple(&self.name);
                for ty in &fields {
                    struc.field(ty);
                }
                struc.finish()
            }
        }
    }
}

/// A representation of some enum variants, where each has a name
/// and 0 or more named or unnamed fields associaetd with it.
pub struct VariantType {
    /// The name of the Variant. This will be a path, and is not used
    /// for (en|de)coding, but is useful for diagnostic messages.
    pub name: String,
    /// Each variant is represented as a tuple of its name, and
    /// a composite type representing the fields of the variant.
    pub variants: Vec<CompositeType>,
}

impl Debug for VariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" { ")?;
        for item in for_each_between(&self.variants) {
            match item {
                ForEachBetween::Item(variant) => {
                    Debug::fmt(variant, f)?;
                },
                ForEachBetween::Between => {
                    f.write_str(", ")?;
                }
            }
        }
        Ok(())
    }
}

/// A sequence of values of some type, like `Vec<T>`.
pub struct SequenceType {
    /// The type of values in the sequence.
    pub values: Box<SubstrateType>
}

impl Debug for SequenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sequence<")?;
        Debug::fmt(&*self.values, f)?;
        f.write_str(">")
    }
}

/// An array type with a fixed length, like `[u8; 32]`.
pub struct ArrayType {
    /// The type of values in the array.
    pub values: Box<SubstrateType>,
    /// The number of items in the array.
    pub len: u32
}

impl Debug for ArrayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        Debug::fmt(&*self.values, f)?;
        f.write_str("; ")?;
        Debug::fmt(&self.len, f)?;
        f.write_str("]")
    }
}

/// A tuple of values of possibly different types.
pub struct TupleType {
    /// The type of each field in the tuple.
    pub fields: Vec<SubstrateType>
}

impl Debug for TupleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tup = f.debug_tuple("");
        for field in &self.fields {
            tup.field(field);
        }
        tup.finish()
    }
}

/// Primitive types. Copied from [`scale_info::TypeDefPrimitive`] but with
/// different Debug impl.
pub enum PrimitiveType {
    Bool, Char, Str,
    U8, U16, U32, U64, U128, U256,
    I8, I16, I32, I64, I128, I256,
}

impl From<scale_info::TypeDefPrimitive> for PrimitiveType {
    fn from(ty: scale_info::TypeDefPrimitive) -> Self {
        match ty {
            scale_info::TypeDefPrimitive::Bool => PrimitiveType::Bool,
            scale_info::TypeDefPrimitive::Char => PrimitiveType::Char,
            scale_info::TypeDefPrimitive::Str => PrimitiveType::Str,
            scale_info::TypeDefPrimitive::U8 => PrimitiveType::U8,
            scale_info::TypeDefPrimitive::U16 => PrimitiveType::U16,
            scale_info::TypeDefPrimitive::U32 => PrimitiveType::U32,
            scale_info::TypeDefPrimitive::U64 => PrimitiveType::U64,
            scale_info::TypeDefPrimitive::U128 => PrimitiveType::U128,
            scale_info::TypeDefPrimitive::U256 => PrimitiveType::U256,
            scale_info::TypeDefPrimitive::I8 => PrimitiveType::I8,
            scale_info::TypeDefPrimitive::I16 => PrimitiveType::I16,
            scale_info::TypeDefPrimitive::I32 => PrimitiveType::I32,
            scale_info::TypeDefPrimitive::I64 => PrimitiveType::I64,
            scale_info::TypeDefPrimitive::I128 => PrimitiveType::I128,
            scale_info::TypeDefPrimitive::I256 => PrimitiveType::I256,
        }
    }
}

impl Debug for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Bool => f.write_str("bool"),
            PrimitiveType::Char => f.write_str("char"),
            PrimitiveType::Str => f.write_str("str"),
            PrimitiveType::U8 => f.write_str("u8"),
            PrimitiveType::U16 => f.write_str("u16"),
            PrimitiveType::U32 => f.write_str("u32"),
            PrimitiveType::U64 => f.write_str("u64"),
            PrimitiveType::U128 => f.write_str("u128"),
            PrimitiveType::U256 => f.write_str("u256"),
            PrimitiveType::I8 => f.write_str("i8"),
            PrimitiveType::I16 => f.write_str("i16"),
            PrimitiveType::I32 => f.write_str("i32"),
            PrimitiveType::I64 => f.write_str("i64"),
            PrimitiveType::I128 => f.write_str("i128"),
            PrimitiveType::I256 => f.write_str("i256"),
        }
    }
}


/// A type that has been compact-encoded.
pub struct CompactType {
    /// Type of the value that has been compact-encoded.
    pub value: Box<SubstrateType>
}

impl Debug for CompactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Compact<")?;
        Debug::fmt(&*self.value, f)?;
        f.write_str(">")
    }
}

/// Represent a `bitvec`.
pub struct BitSequenceType {
    /// What order do we read the bits from the underlying type.
    pub bit_order_type: Box<SubstrateType>,
    /// I believe this is what the underlying storage looks like; eg are
    /// the bits packed into u8's or u16's etc.
    pub bit_store_type: Box<SubstrateType>
}

impl Debug for BitSequenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BitSequence")
    }
}

/// This represents an error when attempting to convert from a [`scale_info::Type`] to a [`SubstrateType`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConvertError {
    #[error("could not find type with ID {0} in the type registry")]
    TypeNotFound(u32)
}

impl SubstrateType {
    /// Convert a type from a [`ScaleInfoType`] to a [`SubstrateType`]. We do this because we can't manually
    /// construct and work with some of the [`ScaleInfoType`] values, but we'd like to be able to do so to
    /// manually drive and test decoding. This also removes the reliance on a separate type registry, by
    /// inlining the relevant type information (at the expense of more allocations/space).
    pub fn from_scale_info_type(ty: &ScaleInfoType, registry: &PortableRegistry) -> Result<SubstrateType, ConvertError> {
        let def = ty.type_def();
        match def {
            ScaleInfoTypeDef::Array(inner) => {
                let len = inner.len();
                let values_ty = resolve_to_substrate_type(inner.type_param(), registry)?;
                let values = Box::new(values_ty);
                Ok(SubstrateType::Array(ArrayType { len, values }))
            },
            ScaleInfoTypeDef::Composite(inner) => {
                let name = join_path(ty.path().segments());
                let fields = composite_fields(inner.fields(), registry)?;
                Ok(SubstrateType::Composite(CompositeType { name, fields }))
            },
            ScaleInfoTypeDef::Variant(inner) => {
                let name = join_path(ty.path().segments());
                let variants_iter = inner.variants().iter().map(|variant| {
                    let name = variant.name().clone();
                    let fields = composite_fields(variant.fields(), registry)?;
                    Ok(CompositeType { name, fields })
                });
                Ok(SubstrateType::Variant(VariantType {
                    name,
                    variants: variants_iter.collect::<Result<_,_>>()?
                }))
            },
            ScaleInfoTypeDef::Sequence(inner) => {
                let inner_ty = resolve_to_substrate_type(inner.type_param(), registry)?;
                Ok(SubstrateType::Sequence(SequenceType { values: Box::new(inner_ty) }))
            },
            ScaleInfoTypeDef::Array(inner) => {
                let len = inner.len();
                let inner_ty = resolve_to_substrate_type(inner.type_param(), registry)?;
                Ok(SubstrateType::Array(ArrayType { len, values: Box::new(inner_ty) }))
            },
            ScaleInfoTypeDef::Tuple(inner) => {
                let fields_iter = inner
                    .fields()
                    .iter()
                    .map(|ty| resolve_to_substrate_type(ty, registry));
                let fields = fields_iter.collect::<Result<_,_>>()?;
                Ok(SubstrateType::Tuple(TupleType { fields }))
            },
            ScaleInfoTypeDef::Primitive(inner) => {
                Ok(SubstrateType::Primitive(inner.clone().into()))
            },
            ScaleInfoTypeDef::Compact(inner) => {
                let inner_ty = resolve_to_substrate_type(inner.type_param(), registry)?;
                Ok(SubstrateType::Compact(CompactType { value: Box::new(inner_ty) }))
            },
            ScaleInfoTypeDef::BitSequence(inner) => {
                let bit_order_type = Box::new(resolve_to_substrate_type(inner.bit_order_type(), registry)?);
                let bit_store_type = Box::new(resolve_to_substrate_type(inner.bit_store_type(), registry)?);
                Ok(SubstrateType::BitSequence(BitSequenceType { bit_order_type, bit_store_type }))
            },
        }
    }
}

/// Convert a slice of [`scale_info::Field`] into our [`CompositeTypeFields`] representation.
fn composite_fields(fields: &[scale_info::Field<scale_info::form::PortableForm>], registry: &PortableRegistry) -> Result<CompositeTypeFields, ConvertError> {
    let are_fields_named = fields.iter().any(|f| f.name().is_some());
    let named_fields = fields.iter().map(|f| {
        let name = f.name().cloned().unwrap_or(String::new());
        let ty = resolve_type(f.ty(), registry)?;
        let substrate_ty = SubstrateType::from_scale_info_type(ty, registry)?;
        Ok((name, substrate_ty))
    });

    match are_fields_named {
        true => {
            let named = named_fields.collect::<Result<_,_>>()?;
            Ok(CompositeTypeFields::Named(named))
        },
        false => {
            let unnamed = named_fields
                .map(|f| f.map(|t| t.1))
                .collect::<Result<_,_>>()?;
            Ok(CompositeTypeFields::Unnamed(unnamed))
        }
    }
}

/// Convert a scale-info type symbol into a [`SubstrateType`].
fn resolve_to_substrate_type(ty: &<scale_info::form::PortableForm as scale_info::form::Form>::Type, registry: &PortableRegistry) -> Result<SubstrateType, ConvertError> {
    let ty = resolve_type(ty, registry)?;
    SubstrateType::from_scale_info_type(ty, registry)
}

/// Convert a scale-info type symbol into a [`ScaleInfoType`].
fn resolve_type<'a>(ty: &<scale_info::form::PortableForm as scale_info::form::Form>::Type, registry: &'a PortableRegistry) -> Result<&'a ScaleInfoType, ConvertError> {
    registry.resolve(ty.id()).ok_or(ConvertError::TypeNotFound(ty.id()))
}

/// Join a path slice like `["foo", "bar", "Wibble"]` into `"foo::bar::Wibble"`.
fn join_path(pieces: &[String]) -> String {
    let mut s = String::new();
    for item in for_each_between(pieces) {
        match item {
            ForEachBetween::Item(p) => s.push_str(p),
            ForEachBetween::Between => s.push_str("::")
        }
    }
    s
}