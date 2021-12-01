![tests](https://github.com/insipx/desub/workflows/Rust/badge.svg)
# De[code] Sub[strate]

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

Currently Supported Metadata Versions (From Kusama CC1):
- [x] V8
- [x] V9
- [x] V10
- [x] V11
- [x] V12
- [x] V13
- [x] V14

### (Tentative) Release & Maintenence
#### Note: Release description is in no way complete because of current & active development for legacy desub types & scale-info based types. it is purely here as a record for things that _should_ be taken into account in the future

- Depending on changes in legacy desub code, bump version in Cargo.toml for `desub/`, `desub-current/`, `desub-legacy/`, `desub-common/`, `desub-json-resolver/`
- note `upgrade-blocks` present [here](https://github.com/polkadot-js/api/tree/master/packages/types-known/src/upgrades) and modify the hard-coded upgrade blocks as necessary in the desub `runtimes.rs` file.
- Take note of PR's that have been merged since the last release.
	- look over CHANGELOG. Make sure to include any PR's that were missed in the `Unreleased` section.
	- Move changes in `Unreleased` section to a new section corresponding to the version being released, making sure to keep the `Unreleased` header.
- make a PR with these changes
- once PR is merged, push a tag in the form `vX.X.X` (E.G `v0.1.0`)
```bash
git tag v0.1.0
git push --tags origin master
```
- Once tags are pushed, a github workflow will start that will draft a release. You should be able to find the workflow
running under `Actions` in the github repository.
	- NOTE: If something goes wrong it is OK. Delete the tag from the repo, re-create the tag locally and re-push. The workflow will run whenever a tag with the correct form is pushed. If more changes need to be made to the repo that will require another PR.
- Once the workflow finishes, make changes to the resulting draft release if necessary, and hit `publish`.
- Once published on github, publish each crate that has changed to `crates.io`. Refer to
  [this](https://doc.rust-lang.org/cargo/reference/publishing.html) for how to publish to crates.io.
