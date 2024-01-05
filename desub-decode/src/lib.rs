#![deny(missing_docs)]

extern crate alloc;

#[cfg(feature = "current")]
pub mod current;

#[cfg(feature = "legacy")]
pub mod legacy;

pub mod visitor;

/// Something implementing this trait is capable of decoding a type, given a [`Decoder::TypeId`].
pub trait Decoder {
    /// Something that identifies a type. in V14/V15 metadata
    /// this might be a u32, and in earlier versions it might be
    /// some struct with chain and type name (maybe spec too).
    type TypeId: ?Sized;

    /// Error type we'll return if decoding fails.
    type Error<VisitorError>;

    /// Given a type ID and a cursor, attempt to decode the type into a
    /// `scale_value::Value`, consuming from the cursor. The Value will
    /// contain the `TypeId` as context, so that we have information on the
    /// type that the Value came from.
    fn decode_type<'info, 'scale, V: scale_decode::Visitor>(
        &'info self,
        type_id: &Self::TypeId,
        bytes: &mut &'scale [u8],
        visitor: V
    ) -> Result<V::Value<'scale, 'info>, Self::Error<V::Error>>;
}

