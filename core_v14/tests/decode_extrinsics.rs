use core_v14::{ Decoder, Metadata, Value, value::{ PrimitiveValue } };

static V14_METADATA_POLKADOT_SCALE: &'static [u8] = include_bytes!("data/v14_metadata_polkadot.scale");

fn decoder() -> Decoder {
    let m = Metadata::from_bytes(V14_METADATA_POLKADOT_SCALE).expect("valid metadata");
    Decoder::with_metadata(m)
}

fn to_bytes(hex_str: &str) -> Vec<u8> {
    let hex_str = hex_str.strip_prefix("0x").expect("0x should prefix hex encoded bytes");
    hex::decode(hex_str).expect("valid bytes from hex")
}

#[test]
fn balance_transfer() {
    let d = decoder();
    let transfer_bytes = to_bytes("0x31028400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d016ada9b477ef454972200e098f1186d4a2aeee776f1f6a68609797f5ba052906ad2427bdca865442158d118e2dfc82226077e4dfdff975d005685bab66eefa38a150200000500001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07ce5c0");
    let transfer_ext = d.decode_extrinsic(&transfer_bytes).expect("can decode extrinsic");

    assert_eq!(transfer_ext.pallet, "Balances".to_string());
    assert_eq!(transfer_ext.call, "transfer".to_string());
    assert_eq!(transfer_ext.arguments.len(), 2);
    assert_eq!(transfer_ext.arguments[1], Value::Primitive(PrimitiveValue::U128(12345)));
}