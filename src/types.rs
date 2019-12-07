use codec::{Encode, Decode, Input, FullCodec, Error as CodecError};

pub trait Decodable {
    type T: FullCodec;

    fn decode(&self, input: Vec<u8>) -> Result<Self::T, CodecError> {
        Self::T::decode(&mut input.as_slice())
    }
}


#[cfg(substrate)] // put cfg on `mod` declaration
pub fn register_standard() {

}
