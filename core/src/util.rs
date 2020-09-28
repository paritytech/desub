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

use serde::{Serializer, ser::{self, SerializeSeq}};
use primitives::crypto::{Ss58AddressFormat, Ss58Codec};
use std::convert::{TryFrom, TryInto};
use crate::{SubstrateType, Error};


// Utility function to serialize from slice/vec to hex
// If the SubstrateType is a collection of u8s, will serialize as hex
pub fn as_hex<S: Serializer>(elements: &Vec<SubstrateType>, serializer: S) -> Result<S::Ok, S::Error> {
    if elements.iter().any(|ty| !matches!(ty, SubstrateType::U8(_))) {
        let mut seq = serializer.serialize_seq(Some(elements.len()))?;
        for e in elements.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    } else {
        let bytes = elements.iter().map(|v| {
            match v {
                SubstrateType::U8(byte) => *byte,
                _ => unreachable!()
            } 
        }).collect::<Vec<u8>>();
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
             let bytes: Vec<u8> = TryFrom::try_from(ty)
                .map_err(|err: Error| ser::Error::custom(format!("{}", err.to_string())))?;
            if bytes.len() != 32 {
                return Err(ser::Error::custom(format!("{}", "address length is incorrect")));
            }
            let mut addr: [u8; 32] = Default::default();
            for (i, b) in bytes.into_iter().enumerate() {
                addr[i] = b;
            }
            let addr = primitives::crypto::AccountId32::from(addr).to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount);
            serializer.serialize_str(&addr)
        },
        SubstrateType::Address(v) => {
            match v {
                pallet_indices::address::Address::Id(ref i) => {
                    let addr = i.to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount);
                    serializer.serialize_str(&addr)
                },
                pallet_indices::address::Address::Index(i) => serializer.serialize_str(&format!("{}", i))
            }
        },
        _ => Err(ser::Error::custom(format!("Could not format {:?} as Ss58 Address", ty)))
    }
}


/*
pub fn from_hex<'a, D>(deserializer: D) -> Result<Vec<SubstrateType>, D::Error> 
where
    D: Deserializer<'a>
{
    String::deserialize(deserializer)
        .and_then(|string| hex::decode(&string).map_err(|err| de::Error::custom(err.to_string())))
}
*/
