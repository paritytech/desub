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

use crate::{Error, SubstrateType};
use serde::{
	ser::{self, SerializeSeq},
	Serializer,
};
use sp_core::crypto::Ss58Codec;
use std::convert::TryFrom;

// Utility function to serialize from slice/vec to hex
// If the SubstrateType is a collection of u8s, will serialize as hex
pub fn as_hex<S: Serializer>(elements: &[SubstrateType], serializer: S) -> Result<S::Ok, S::Error> {
	if elements.iter().any(|ty| !matches!(ty, SubstrateType::U8(_))) {
		let mut seq = serializer.serialize_seq(Some(elements.len()))?;
		for e in elements.iter() {
			seq.serialize_element(&e)?;
		}
		seq.end()
	} else {
		let bytes = elements
			.iter()
			.map(|v| match v {
				SubstrateType::U8(byte) => *byte,
				_ => unreachable!(),
			})
			.collect::<Vec<u8>>();
		let mut hex_str = String::from("0x");
		hex_str.push_str(&hex::encode(bytes.as_slice()));
		serializer.serialize_str(&hex_str)
	}
}

/// Serialize a Substrate Type as a ss58 Address
/// # Panics
/// Panics if a SubstrateType can not be serialized into an ss58 address type
pub fn as_substrate_address<S: Serializer>(ty: &SubstrateType, serializer: S) -> Result<S::Ok, S::Error> {
	match ty {
		SubstrateType::Composite(_) => {
			let bytes: Vec<u8> = TryFrom::try_from(ty).map_err(|err: Error| ser::Error::custom(err.to_string()))?;
			if bytes.len() != 32 {
				return Err(ser::Error::custom("address length is incorrect".to_string()));
			}
			let mut addr: [u8; 32] = Default::default();
			for (i, b) in bytes.into_iter().enumerate() {
				addr[i] = b;
			}
			let addr = sp_core::crypto::AccountId32::from(addr).to_ss58check();
			serializer.serialize_str(&addr)
		}
		SubstrateType::Address(v) => match v {
			sp_runtime::MultiAddress::Id(ref i) => {
				let addr = i.to_ss58check();
				serializer.serialize_str(&addr)
			}
			sp_runtime::MultiAddress::Index(i) => serializer.serialize_str(&format!("{}", i)),
			sp_runtime::MultiAddress::Raw(bytes) => serializer.serialize_str(&format!("{:?}", bytes)),
			sp_runtime::MultiAddress::Address32(ary) => serializer.serialize_str(&format!("{:?}", ary)),
			sp_runtime::MultiAddress::Address20(ary) => serializer.serialize_str(&format!("{:?}", ary)),
		},
		_ => Err(ser::Error::custom(format!("Could not format {:?} as Ss58 Address", ty))),
	}
}
