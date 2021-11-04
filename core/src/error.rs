use crate::decoder::MetadataError;
use codec::Error as CodecError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Codec {0}")]
	Codec(#[from] CodecError),
	#[error("{0}")]
	Metadata(#[from] MetadataError),
	#[error("Failed to get metadata item because of `{0}`, where cursor is {1} and data is {2}")]
	DetailedMetaFail(MetadataError, usize, String),
	#[error("decoding failed")]
	DecodeFail,
	#[error("error: {0}")]
	Fail(String),
	#[error("parse error {0}")]
	Regex(#[from] onig::Error),
	#[error("Conversion from {0} to {1} not possible")]
	Conversion(String, String),
	#[error("Spec version {0} not present in Decoder")]
	MissingSpec(u32),
}

impl From<&str> for Error {
	fn from(e: &str) -> Error {
		Error::Fail(e.to_string())
	}
}

impl From<String> for Error {
	fn from(e: String) -> Error {
		Error::Fail(e)
	}
}
