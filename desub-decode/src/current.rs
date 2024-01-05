use super::Decoder;
use scale_info::PortableRegistry;

/// To decode bytes in recent runtimes, you can use a
/// [`scale_info::PortableRegistry`], which this is an alias to.
/// This is contained within the V14 or later metadata which can
/// be retrieved from recent runtimes.
pub type CurrentDecoder = PortableRegistry;

impl Decoder for CurrentDecoder {
    // The ID of a type in the portable registry.
    type TypeId = u32;

    // We just return the error produced by the visitor.
    type Error<VisitorError> = VisitorError;

    fn decode_type<'info, 'scale, V: scale_decode::Visitor>(
        &'info self,
        type_id: &Self::TypeId,
        bytes: &mut &'scale [u8],
        visitor: V
    ) -> Result<V::Value<'scale, 'info>, Self::Error<V::Error>> {
        scale_decode::visitor::decode_with_visitor(bytes, *type_id, self, visitor)
    }
}