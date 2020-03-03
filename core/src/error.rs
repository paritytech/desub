use codec::Error as CodecError;
use crate::decoder::MetadataError;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Codec {:?}", _0)]
    Codec(#[fail(cause)] CodecError),
    #[fail(display = "{:?}", _0)]
    Metadata(#[fail(cause)]  MetadataError),
    #[fail(display = "decoding failed")]
    DecodeFail
}

impl From<CodecError> for Error {
    fn from(err: CodecError) -> Error {
        Error::Codec(err)
    }
}

impl From<MetadataError> for Error {
    fn from(err: MetadataError) -> Error {
        Error::Metadata(err)
    }
}
