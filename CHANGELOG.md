# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] – 2021-12-21

### Added

- Update to latest scale-coded ^ V13 type-definitions + metadata ([#41](https://github.com/paritytech/desub/pull/41))
- V14 Decoding ([#45](https://github.com/paritytech/desub/pull/45))
- TxDecoder for integration testing ([#42](https://github.com/paritytech/desub/pull/42)), ([#46](https://github.com/paritytech/desub/pull/46))
- Support deserializing from `Value`to arbitrary type via `serde` ([#53](https://github.com/paritytech/desub/pull/53))
- implement `Deserialize` for `core_v14::Value` ([#56](https://github.com/paritytech/desub/pull/56))
- parse signed payload and expose call data type information ([#66](https://github.com/paritytech/desub/pull/66))
- Decode storage for v14 metadata ([#75](https://github.com/paritytech/desub/pull/75))
- Parameterise `Value` type to add context. For instance this is useful for attaching `TypeId` to `Value`. ([#79](https://github.com/paritytech/desub/pull/79))

### Changed

- use [`frame-metadata`](https://crates.io/crates/frame-metadata) everywhere ([#48](https://github.com/paritytech/desub/pull/48))
- move to Rust 2021 ([#76](https://github.com/paritytech/desub/pull/76))

### Removed

### Fixed

- Fix serde `u128` bug caused by `flatten` attribute ([#77](https://github.com/paritytech/desub/pull/77))
