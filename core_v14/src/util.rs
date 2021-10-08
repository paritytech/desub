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

// use crate::substrate_value::SubstrateValue;
// use primitives::crypto::{Ss58AddressFormat, Ss58Codec};
// use serde::{
// 	ser::{self, SerializeSeq},
// 	Serializer,
// };
// use std::convert::TryFrom;

// // Utility function to serialize from slice/vec to hex
// // If the SubstrateValue is a collection of u8s, will serialize as hex
// pub fn as_hex<S: Serializer>(elements: &[SubstrateValue], serializer: S) -> Result<S::Ok, S::Error> {
// 	if elements.iter().any(|ty| !matches!(ty, SubstrateValue::U8(_))) {
// 		let mut seq = serializer.serialize_seq(Some(elements.len()))?;
// 		for e in elements.iter() {
// 			seq.serialize_element(&e)?;
// 		}
// 		seq.end()
// 	} else {
// 		let bytes = elements
// 			.iter()
// 			.map(|v| match v {
// 				SubstrateValue::U8(byte) => *byte,
// 				_ => unreachable!(),
// 			})
// 			.collect::<Vec<u8>>();
// 		let mut hex_str = String::from("0x");
// 		hex_str.push_str(&hex::encode(bytes.as_slice()));
// 		serializer.serialize_str(&hex_str)
// 	}
// }

// /// Serialize a Substrate Type as a ss58 Address
// /// # Panics
// /// Panics if a SubstrateValue can not be serialized into an ss58 address type
// pub fn as_substrate_address<S: Serializer>(ty: &SubstrateValue, serializer: S) -> Result<S::Ok, S::Error> {
// 	match ty {
// 		SubstrateValue::Composite(_) => {
// 			let bytes = <Vec<u8>>::try_from(ty).map_err(|err| ser::Error::custom(err.to_string()))?;
// 			if bytes.len() != 32 {
// 				return Err(ser::Error::custom("address length is incorrect".to_string()));
// 			}
// 			let mut addr: [u8; 32] = Default::default();
// 			for (i, b) in bytes.into_iter().enumerate() {
// 				addr[i] = b;
// 			}
// 			let addr = primitives::crypto::AccountId32::from(addr)
// 				.to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount);
// 			serializer.serialize_str(&addr)
// 		}
// 		SubstrateValue::Address(v) => match v {
// 			runtime_primitives::MultiAddress::Id(ref i) => {
// 				let addr = i.to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount);
// 				serializer.serialize_str(&addr)
// 			}
// 			runtime_primitives::MultiAddress::Index(i) => serializer.serialize_str(&format!("{}", i)),
// 			runtime_primitives::MultiAddress::Raw(bytes) => serializer.serialize_str(&format!("{:?}", bytes)),
// 			runtime_primitives::MultiAddress::Address32(ary) => serializer.serialize_str(&format!("{:?}", ary)),
// 			runtime_primitives::MultiAddress::Address20(ary) => serializer.serialize_str(&format!("{:?}", ary)),
// 		},
// 		_ => Err(ser::Error::custom(format!("Could not format {:?} as Ss58 Address", ty))),
// 	}
// }

/// Run a function for each item in an iterator, and run another function between
/// each item in the iterator.
pub fn for_each_between<I, T>(iter: I) -> impl Iterator<Item = ForEachBetween<T>>
where I: IntoIterator<Item = T>,
{
	let mut peekable = iter.into_iter().peekable();
	let mut item_next = true;
	let mut is_between = true;

	std::iter::from_fn(move || {
		let res = if item_next {
			peekable.next().map(ForEachBetween::Item)
		} else if is_between {
			Some(ForEachBetween::Between)
		} else {
			return None;
		};

		item_next = !item_next;
		is_between = peekable.peek().is_some();
		res
	})
}

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum ForEachBetween<T> {
	Item(T),
	Between
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn is_between_works() {
		let mut iter = for_each_between(vec![1,2,3]);
		assert_eq!(iter.next(), Some(ForEachBetween::Item(1)));
		assert_eq!(iter.next(), Some(ForEachBetween::Between));
		assert_eq!(iter.next(), Some(ForEachBetween::Item(2)));
		assert_eq!(iter.next(), Some(ForEachBetween::Between));
		assert_eq!(iter.next(), Some(ForEachBetween::Item(3)));
		assert_eq!(iter.next(), None);
		assert_eq!(iter.next(), None);
	}
}