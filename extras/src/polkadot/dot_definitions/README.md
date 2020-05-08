## Definitions

overrides.json and definitions.json are taken directly from polkadot-js via [polkadot-json-definitions](https://github.com/insipx/polkadot-json-definitions)
extrinsics.json is created in the same style but maintained directly. Extrinsics.json contains definitions for Signature, Address, and SignedExtra


### Instructions for formatting definitions:
- export with 'polkadot-json-definitions'
- remove 'api{}' tags
- remove `_alias` objects
- make all module names lowercase (imOnline -> imonline)
- add a definition for "Bytes": "Vec<u8>" in the 'runtime' module
