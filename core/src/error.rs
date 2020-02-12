use codec::Error as CodecError;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Codec {:?}", _0)]
    Codec(#[fail(cause)] CodecError),
    #[fail(display = "{} Not Found", _0)]
    NotFound(String)
}

impl From<CodecError> for Error {
    fn from(err: CodecError) -> Error {
        Error::Codec(err)
    }
}
