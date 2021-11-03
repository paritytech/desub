// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

use core_v14::{decoder::SignedExtensionWithAdditional, value, Decoder, Metadata, Value};

static V14_METADATA_POLKADOT_SCALE: &[u8] = include_bytes!("data/v14_metadata_polkadot.scale");

fn decoder() -> Decoder {
	let m = Metadata::from_bytes(V14_METADATA_POLKADOT_SCALE).expect("valid metadata");
	Decoder::with_metadata(m)
}

fn to_bytes(hex_str: &str) -> Vec<u8> {
	let hex_str = hex_str.strip_prefix("0x").expect("0x should prefix hex encoded bytes");
	hex::decode(hex_str).expect("valid bytes from hex")
}

fn unnamed_value(x: Vec<Value>) -> Value {
	Value::Composite(value::Composite::Unnamed(x))
}

fn empty_value() -> Value {
	unnamed_value(vec![])
}

fn singleton_value(x: Value) -> Value {
	unnamed_value(vec![x])
}

fn prim_u8_value(x: u8) -> Value {
	Value::Primitive(value::Primitive::U8(x))
}

fn prim_u32_value(x: u32) -> Value {
	Value::Primitive(value::Primitive::U32(x))
}

fn prim_u128_value(x: u128) -> Value {
	Value::Primitive(value::Primitive::U128(x))
}

fn hash_value(xs: Vec<u8>) -> Value {
	singleton_value(Value::Composite(value::Composite::Unnamed(xs.iter().map(|x| prim_u8_value(*x)).collect())))
}

fn variant_value(name: &str, c: value::Composite) -> Value {
	Value::Variant(value::Variant { name: name.to_string(), values: c })
}

// These tests are intended to roughly check that we can decode a range of "real" extrinsics into something
// that looks sane.
//
// How did I write these tests (and how can you add more)?
//
// 1. Start up a polkadot node locally using `cargo run -- --dev --tmp` in the polkadot repo (I happened to
//    be on polkadot master, commit 6da1cf6233728a8142e4b9cebdcf29cd67eb8352).
//
// 2. I downloaded the SCALE encoded metadata from it and save in this repo to include in the tests. You don't
//    need to do this again (unless you'd like to make sure it's fully uptodate).
//
//    curl -sX POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"state_getMetadata", "id": 1}' localhost:9933 \
//        | jq .result \
//        | cut -d '"' -f 2 \
//        | xxd -r -p > ./tests/node_runtime.scale
//
// 3. Navigate to https://polkadot.js.org/apps/#/explorer and switch it to pointing at a local development node.
//    (the one you just started up in step 1).
//
// 4. In "Developer -> Extrinsics", we can now build, sign and submit extrinsics.
//    - If you want the hex str for a signed extrinsic, Keep the network tab open and find the open WS connection. When an extrinsic is
//      submitted, You'll see a new message to the method "author_submitAndWatchExtrinsic".
//    - If you want an unsigned extrinsic, just copy the "call data" hex string and prepend a "04" after the "0x" and before everything
//      else, to turn the call data into a V4 unsigned extrinsic (minus the byte length, which would normally be first). We can test
//      decoding of this using `decode_unwrapped_extrinsic`.
//
// 5. With that in mind, see the tests below for examples of how we can test decoding of this extrinsic hex string you've acquired.

#[test]
fn balance_transfer_signed() {
	let d = decoder();

	// Balances.transfer (amount: 12345)
	let ext_bytes = &mut &*to_bytes("0x31028400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d016ada9b477ef454972200e098f1186d4a2aeee776f1f6a68609797f5ba052906ad2427bdca865442158d118e2dfc82226077e4dfdff975d005685bab66eefa38a150200000500001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07ce5c0");
	let ext = d.decode_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "Balances");
	assert_eq!(&*ext.call_data.ty.name(), "transfer");
	assert_eq!(ext.call_data.arguments.len(), 2);
	assert_eq!(ext.call_data.arguments[1], prim_u128_value(12345));
}

#[test]
fn balance_transfer_all_signed() {
	let d = decoder();

	// Balances.transfer_all (keepalive: false)
	let ext_bytes = &mut &*to_bytes("0x2d028400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d01f0431ffe387134b4f84d92d3c3f1ac18c0f42237ad7dbd455bb0cf8a18efb1760528f052b2219ad1601d9a4719e1a446cf307bf6d7e9c56175bfe6e7bf8cbe81450304000504001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c00");
	let ext = d.decode_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "Balances");
	assert_eq!(&*ext.call_data.ty.name(), "transfer_all");
	assert_eq!(ext.call_data.arguments.len(), 2);
	assert_eq!(ext.call_data.arguments[1], Value::Primitive(value::Primitive::Bool(false)));
}

/// This test is interesting because:
/// a) The Auctions pallet index is not the same as where it is listed in the list of pallets.
/// b) One of the arguments is a compact-encoded wrapper struct, which caused a hiccup.
#[test]
fn auctions_bid_unsigned() {
	let d = decoder();

	// Auctions.bid (Args: (1,), 2, 3, 4, 5, all compact encoded).
	let ext_bytes = &mut &*to_bytes("0x04480104080c1014");
	let ext = d.decode_unwrapped_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "Auctions");
	assert_eq!(&*ext.call_data.ty.name(), "bid");
	assert_eq!(ext.call_data.arguments.len(), 5);

	assert_eq!(
		ext.call_data.arguments,
		vec![
			singleton_value(prim_u32_value(1)),
			prim_u32_value(2),
			prim_u32_value(3),
			prim_u32_value(4),
			prim_u128_value(5),
		]
	);
}

#[test]
fn system_fill_block_unsigned() {
	let d = decoder();

	// System.fill_block (Args: Perblock(1234)).
	let ext_bytes = &mut &*to_bytes("0x040000d2040000");
	let ext = d.decode_unwrapped_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "System");
	assert_eq!(&*ext.call_data.ty.name(), "fill_block");
	assert_eq!(ext.call_data.arguments.len(), 1);

	assert_eq!(ext.call_data.arguments, vec![singleton_value(prim_u32_value(1234))]);
}

/// This test is interesting because you provide a nested enum representing a call
/// as an argument to this call.
#[test]
fn technical_committee_execute_unsigned() {
	let d = decoder();

	// TechnicalCommittee.execute (Args: Balances.transfer(Alice -> Bob, 12345), 500).
	let ext_bytes =
		&mut &*to_bytes("0x0410010500001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07ce5c0d107");
	let ext = d.decode_unwrapped_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "TechnicalCommittee");
	assert_eq!(&*ext.call_data.ty.name(), "execute");
	assert_eq!(ext.call_data.arguments.len(), 2);

	// It's a bit hard matching the entire thing, so we just verify that the first arg looks like
	// a variant representing a call to "Balances.transfer".
	assert!(matches!(&ext.call_data.arguments[0],
		Value::Variant(value::Variant {
			name,
			values: value::Composite::Unnamed(args)
		})
		if &*name == "Balances"
		&& matches!(&args[0], Value::Variant(value::Variant { name, ..}) if &*name == "transfer")
	));
	assert_eq!(&ext.call_data.arguments[1], &prim_u32_value(500));
}

#[test]
fn tips_report_awesome_unsigned() {
	let d = decoder();

	// Tips.report_awesome (Args: b"This person rocks!", AccountId).
	let ext_bytes = &mut &*to_bytes("0x042300485468697320706572736f6e20726f636b73211cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c");
	let ext = d.decode_unwrapped_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "Tips");
	assert_eq!(&*ext.call_data.ty.name(), "report_awesome");
	assert_eq!(ext.call_data.arguments.len(), 2);

	assert_eq!(
		&ext.call_data.arguments[0],
		&Value::Composite(value::Composite::Unnamed("This person rocks!".bytes().map(prim_u8_value).collect()))
	);
}

// Named structs shouldn't be an issue; this extrinsic contains one.
#[test]
fn vesting_force_vested_transfer_unsigned() {
	let d = decoder();

	// Vesting.force_vested_transfer (Args: AccountId, AccountId, { locked: 1u128, perBlock: 2u128, startingBlock: 3u32 }).
	let ext_bytes = &mut &*to_bytes("0x04190300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48010000000000000000000000000000000200000000000000000000000000000003000000");
	let ext = d.decode_unwrapped_extrinsic(ext_bytes).expect("can decode extrinsic");

	assert!(ext_bytes.is_empty(), "No more bytes expected");
	assert_eq!(ext.call_data.pallet_name, "Vesting");
	assert_eq!(&*ext.call_data.ty.name(), "force_vested_transfer");
	assert_eq!(ext.call_data.arguments.len(), 3);

	assert_eq!(
		&ext.call_data.arguments[2],
		&Value::Composite(value::Composite::Named(vec![
			("locked".into(), prim_u128_value(1)),
			("per_block".into(), prim_u128_value(2)),
			("starting_block".into(), prim_u32_value(3)),
		]))
	);
}

#[test]
fn test_signer_payload() {
	let d = decoder();
	let signer_payload = &mut &*to_bytes("0x0706b9340000962300000800000091b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c31c81d421f68281950ad2901291603b5e49fc5c872f129e75433f4b55f07ca072");

	let r = d.decode_signer_payload(signer_payload).expect("can decode signer payload");

	assert_eq!(signer_payload.len(), 0);
	assert_eq!(r.call_data.pallet_name, "Staking");
	assert_eq!(&*r.call_data.ty.name(), "chill");
	assert_eq!(r.call_data.arguments, vec![]);

	assert_eq!(
		r.extensions,
		vec![
			(
				"CheckSpecVersion".into(),
				SignedExtensionWithAdditional { extension: empty_value(), additional: prim_u32_value(9110) }
			),
			(
				"CheckTxVersion".into(),
				SignedExtensionWithAdditional { extension: empty_value(), additional: prim_u32_value(8) }
			),
			(
				"CheckGenesis".into(),
				SignedExtensionWithAdditional {
					extension: empty_value(),
					additional: hash_value(to_bytes(
						"0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"
					))
				}
			),
			(
				"CheckMortality".into(),
				SignedExtensionWithAdditional {
					extension: singleton_value(variant_value(
						"Mortal185",
						value::Composite::Unnamed(vec![prim_u8_value(52)])
					)),
					additional: hash_value(to_bytes(
						"0x1c81d421f68281950ad2901291603b5e49fc5c872f129e75433f4b55f07ca072"
					))
				}
			),
			(
				"CheckNonce".into(),
				SignedExtensionWithAdditional {
					extension: singleton_value(prim_u32_value(0)),
					additional: empty_value()
				}
			),
			(
				"CheckWeight".into(),
				SignedExtensionWithAdditional { extension: empty_value(), additional: empty_value() }
			),
			(
				"ChargeTransactionPayment".into(),
				SignedExtensionWithAdditional {
					extension: singleton_value(prim_u128_value(0)),
					additional: empty_value()
				}
			),
			(
				"PrevalidateAttests".into(),
				SignedExtensionWithAdditional { extension: empty_value(), additional: empty_value() }
			),
		],
	);
}
