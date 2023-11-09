use desub::{Chain, Decoder};
use subxt::{
	backend::{
		legacy::{
			rpc_methods::{Bytes, NumberOrHex},
			LegacyRpcMethods,
		},
		rpc::{rpc_params, RpcClient},
	},
	config::PolkadotConfig,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	pretty_env_logger::init();

	// Connect to a node with an RPC client:
	let rpc_client = RpcClient::from_url("wss://rpc.polkadot.io").await?;
	let methods = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());

	let mut bn = 1;
	let mut decoder = Decoder::new(Chain::Polkadot);

	loop {
		let hash = methods.chain_get_block_hash(Some(NumberOrHex::Number(bn))).await?.unwrap();
		let runtime_version = methods.state_get_runtime_version(Some(hash)).await?;
		let spec_version = runtime_version.spec_version;
		let block = methods.chain_get_block(Some(hash)).await?.unwrap();

		let ext_bytes = make_extrinsic_bytes(block.block.extrinsics);

		if !decoder.has_version(spec_version) {
			// download the relevant metadata bytes, since the decoder doesn't have it yet.
			let md: Bytes = rpc_client.request("state_getMetadata", rpc_params![hash]).await?;
			decoder.register_version(spec_version, &md.0)?;
		}

		println!("# Decoding exts for block {bn}");
		let decoded_exts = decoder.decode_extrinsics(spec_version, &ext_bytes)?;

		println!("{decoded_exts}");
		bn += 10_000;
	}
}

// A hack because we get the exts back as a vec of bytes and
// desub wants the whole block body as bytes.
fn make_extrinsic_bytes(exts: Vec<Bytes>) -> Vec<u8> {
	use subxt::ext::codec::Encode;
	use subxt::utils::Encoded;
	// The inner `Bytes` are already encoded and contain the compact length etc,
	// so don't encode them again by wrapping them in `Encoded`.
	let e: Vec<Encoded> = exts.into_iter().map(|ext| Encoded(ext.0)).collect();
	e.encode()
}
