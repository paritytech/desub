use super::Decoder;

pub enum LegacyDecodeError<VisitorError> {
    Visitor(VisitorError)
}

/// Given a valid [`TypeMapping`], we are able to decode types
/// based on their string identifiers. This can be used to decode
/// legacy types where we don't have any `PortableRegistry` to hand
/// (ie when the runtime metadata version is < V14).
pub struct LegacyDecoder<TM>(TM);

impl <TM: TypeMapping> Decoder for LegacyDecoder<TM> {
    type TypeId = str;
    type Error<VisitorError> = LegacyDecodeError<VisitorError>;

    fn decode_type<'info, 'scale, V: scale_decode::Visitor>(
        &'info self,
        type_id: &Self::TypeId,
        bytes: &mut &'scale [u8],
        visitor: V
    ) -> Result<V::Value<'scale, 'info>, Self::Error<V::Error>> {
        todo!()
    }
}

/// This trait can be implemented by anything that, given a string
/// identifier for a type, can return a [`TypeMarker`] which describes
/// the shape of the type (or `None` if it's not found).
pub trait TypeMapping {
    fn type_mapping(&self, type_identifier: &str) -> Option<TypeMarker>;
}

/// This defines what the shape of a type is, and thus how we should decode it.
pub enum TypeMarker {

}