![tests](https://github.com/insipx/desub/workflows/Rust/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/insipx/desub/badge.svg?branch=master)](https://coveralls.io/github/insipx/desub?branch=master)
# Desub

Encompassing decoder for substrate/polkadot/kusama types.

Gets type definitions from polkadot-js via JSON and decodes them into components
that outline types and make decoding byte-strings possible, as long as the
module/generic type name are known. 

Supports Metadata versions from v7, which means all of Kusama (from CC1). Older networks are not supported (E.G Alexander).
   
   - makes decoding generic types from the substrate rpc possible
   - requires parsing JSON with type definitions, and implementing traits
      `TypeDetective` and `Decoder` in order to work for arbitrary chains.
      However, if the JSON follows the same format as PolkadotJS definitions
      (look at `definitions.json` and `overrides.json`) it would be possible to
      simply deserialize into Polkadot structs and utilize those. The decoding
      itself is generic enough to allow it.
   - types must adhere to the conventions set out by polkadot decoding
      - type definitions for Polkadot (Kusama) are taken from Polkadot.js and deserialized into Rust (extras/polkadot)
   - type-metadata support (IE, self-referential types) will be supported once
    they are included in substrate proper

Currently Supported Metadata Versions (From Kusama CC1):
- [ ] V0
- [ ] V1
- [ ] V2
- [ ] V3 
- [ ] V4
- [ ] V5
- [ ] V6
- [ ] V5
- [ ] V6
- [x] v7
- [x] V8
- [x] V9
- [x] V10
- [x] V11
