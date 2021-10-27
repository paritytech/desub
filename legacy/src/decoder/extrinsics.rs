// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version. //
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

//! Generic Extrinsic Type and Functions

use crate::substrate_types::SubstrateType;
use serde::Serialize;
use std::fmt;
#[derive(Debug, Serialize)]
pub struct ExtrinsicArgument {
	pub name: String,
	pub arg: SubstrateType,
}

impl fmt::Display for ExtrinsicArgument {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, " arg: {}, Ty: {} ", self.name, self.arg)
	}
}

#[derive(Debug, Serialize)]
pub struct GenericCall {
	name: String,
	module: String,
	args: Vec<ExtrinsicArgument>,
}

impl fmt::Display for GenericCall {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::from("");
		s.push_str(&self.name);
		s.push_str(":   ");
		for val in self.args.iter() {
			s.push_str(&format!("{}", val));
		}
		write!(f, "{}", s)
	}
}

/// Generic Extrinsic Type
#[derive(Debug, Serialize)]
pub struct GenericExtrinsic {
	signature: Option<GenericSignature>,
	call: GenericCall,
}

impl fmt::Display for GenericExtrinsic {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::from("");
		if let Some(v) = &self.signature {
			s.push_str(&format!("{}", v));
		} else {
			s.push_str(&"None".to_string());
		}
		s.push('\n');
		s.push_str("CALL");
		s.push('\n');
		s.push_str(&format!("{}", self.call));
		write!(f, "{}", s)
	}
}

impl GenericExtrinsic {
	/// create a new generic extrinsic type
	pub fn new(sig: Option<SubstrateType>, call: Vec<(String, SubstrateType)>, name: String, module: String) -> Self {
		let call =
			call.into_iter().map(|c| ExtrinsicArgument { name: c.0, arg: c.1 }).collect::<Vec<ExtrinsicArgument>>();
		let call = GenericCall { name, module, args: call };
		Self { signature: sig.map(GenericSignature::new), call }
	}

	pub fn is_signed(&self) -> bool {
		self.signature.is_some()
	}

	pub fn signature(&self) -> Option<&GenericSignature> {
		self.signature.as_ref()
	}

	pub fn call(&self) -> &GenericCall {
		&self.call
	}

	pub fn ext_module(&self) -> &str {
		&self.call.module
	}

	pub fn ext_call(&self) -> &str {
		&self.call.name
	}

	pub fn args(&self) -> &[ExtrinsicArgument] {
		&self.call.args
	}
}

#[derive(Debug, Serialize)]
pub struct GenericSignature {
	#[serde(serialize_with = "crate::util::as_substrate_address")]
	address: SubstrateType,
	signature: SubstrateType,
	extra: SubstrateType,
}

impl GenericSignature {
	pub fn new(signature: SubstrateType) -> Self {
		Self::split(signature)
	}

	/// returns address signature and extra as a tuple
	pub fn parts(&self) -> (&SubstrateType, &SubstrateType, &SubstrateType) {
		(&self.address, &self.signature, &self.extra)
	}

	fn split(sig: SubstrateType) -> Self {
		match sig {
			SubstrateType::Composite(mut v) => {
				v.reverse();
				Self {
					address: v.pop().expect("Address must must be present in signature"),
					signature: v.pop().expect("Signature must be present"),
					extra: v.pop().expect("Extra must be present"),
				}
			}
			_ => panic!("Signature should always be a tuple of Address, Signature, Extra"),
		}
	}
}

impl fmt::Display for GenericSignature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Address {}\n Signature {}\n SignedExtra {}\n", self.address, self.signature, self.extra)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_serialize_generic_extrinsic() {
		let call = GenericCall {
			name: "set".to_string(),
			module: "Timestamp".to_string(),
			args: vec![ExtrinsicArgument { name: "Some Arg".to_string(), arg: SubstrateType::U32(32) }],
		};
		let ext = GenericExtrinsic {
			signature: Some(GenericSignature::new(SubstrateType::Composite(vec![
				SubstrateType::Composite(vec![
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
					0u8.into(),
				]),
				SubstrateType::U64(64),
				SubstrateType::U128(128),
			]))),
			call,
		};
		let serialized = serde_json::to_string(&ext).unwrap();
		assert_eq!(
			serialized,
			r#"{"signature":{"address":"5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM","signature":64,"extra":128},"call":{"name":"set","module":"Timestamp","args":[{"name":"Some Arg","arg":32}]}}"#
		);
	}
}
