use crate::decoder::MetadataError;
use codec::Error as CodecError;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Codec {:?}", _0)]
    Codec(#[fail(cause)] CodecError),
    #[fail(display = "{:?}", _0)]
    Metadata(#[fail(cause)] MetadataError),
    #[fail(display = "decoding failed")]
    DecodeFail,
    #[fail(display = "error: {}", _0)]
    Fail(String),
}
impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Fail(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Error {
        Error::Fail(err.to_string())
    }
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
