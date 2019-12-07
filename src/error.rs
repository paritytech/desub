use failure::Fail;
use codec::Error as CodecError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Codec {:?}", _0)]
    Codec(#[fail(cause)] CodecError)
}

impl From<CodecError> for Error {
    fn from(err: CodecError) -> Error {
        Error::Codec(err)
    }
}
