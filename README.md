![tests](https://github.com/insipx/desub/workflows/Rust/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/paritytech/desub/badge.svg?branch=master&service=github)](https://coveralls.io/github/paritytech/desub?branch=master&service=github)
# De[code] [S]ubstrate

<sub><sup>† This software is experimental, and not intended for production use yet. Use at your own risk.

Encompassing decoder for substrate/polkadot/kusama types.

Gets type definitions from polkadot-js via JSON and decodes them into components
that outline types and make decoding byte-strings possible, as long as the
module/generic type name are known. 

Supports Metadata versions from v8, which means all of Kusama (from CC1). Older networks are not supported (E.G Alexander).
   - makes decoding generic types from the substrate rpc possible
   - requires parsing JSON with type definitions, and implementing traits
      `TypeDetective` and `Decoder` in order to work for arbitrary chains.
      However, if the JSON follows the same format as PolkadotJS definitions
      (look at `definitions.json` and `overrides.json`) it would be possible to
      simply deserialize into Polkadot structs and utilize those. The decoding
      itself is generic enough to allow it.
   - types must adhere to the conventions set out by polkadot decoding
      - type definitions for Polkadot (Kusama) are taken from Polkadot.js and deserialized into Rust (extras/polkadot)


|  Chain       | Extrinsics | Storage | Map | Double Map | Blake\_2* hashes | Twox_* Hashes | Identity Hash |
| ------------ | ---------- | ---     | --- | ---------- | ---------------- | ------------- | ------------- |
|  Kusama      |  ✅	    | ✅      | ✅  |     ✅     |        ✅        |        ✅     |               |
|  Polkadot    |   	    |         |     |            |                  |               |               |
|  Westend     |   	    |         |     |            |                  |               |               |
|  Kulupu      |   	    |         |     |            |                  |               |               |



Currently Supported Metadata Versions (From Kusama CC1):
- [x] V8
- [x] V9
- [x] V10
- [x] V11
