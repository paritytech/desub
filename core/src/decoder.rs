// Copyright 2019 Parity Technologies (UK) Ltd.
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

//! A serializable/deserializable Decoder used to encode/decode substrate types
//! from compact SCALE encoded byte arrays
//! with special attention paid to generic types in runtime module trait
//! definitions if serialized, can be deserialized. This allows for portability
//! by not needing to import differently-versioned runtimes
//! as long as all the types of the runtime are registered within the decoder
//!
//! Theoretically, one could upload the deserialized decoder JSON to distribute
//! to different applications that need the type data

mod metadata;

#[cfg(test)]
pub use self::metadata::test_suite;

pub use self::metadata::Metadata;
use std::collections::HashMap;
use crate::TypeDetective;

type SpecVersion = u32;
/// Decoder for substrate types
///
/// hold information about the Runtime Metadata
/// and maps types inside the runtime metadata to self-describing types in
/// type-metadata
#[derive(Default, Debug)]
pub struct Decoder<T: TypeDetective> {
    // reference to an item in 'versions' vector
    // NOTE: possibly a concurrent HashMap
    versions: HashMap<SpecVersion, Metadata>,
    types: T
}

/// The type of Entry
///
/// # Note
///
/// not entirely sure if necessary as of yet
/// however, used for the purpose for narrowing down the context a type is being
/// used in
#[derive(Debug)]
pub enum Entry {
    Call,
    Storage,
    Event,
    Constant,
}

impl<T> Decoder<T> where T: TypeDetective {

    /// Create new Decoder with specified types
    pub fn new(types: T) -> Self {
        Self {
            versions: HashMap::default(),
            types
        }
    }

    /// Insert a Metadata with Version attached
    /// If version exists, it's corresponding metadata will be updated
    pub fn register_version(
        &mut self,
        version: SpecVersion,
        metadata: Metadata,
    ) {
        self.versions.insert(version, metadata);
    }

    /// internal api to get runtime version
    /// panics if a version is not found
    ///
    /// get runtime version in less than linear time with binary search
    ///
    /// # Note
    /// Returns None if version is nonexistant
    pub fn get_version_metadata(
        &self,
        version: SpecVersion,
    ) -> Option<&Metadata> {
        self.versions.get(&version)
    }

    #[allow(dead_code)]
    /// Verifies if all generic types of 'RuntimeMetadata' are present
    fn verify(&self) -> bool {
        // TODO: implement
        unimplemented!()
    }

    /// dynamically Decode a SCALE-encoded byte string into it's concrete rust
    /// types
    pub fn decode(
        &self,
        spec: SpecVersion,
        _module: String,
        _ty: String,
        _data: Vec<u8>,
    ) {
        // have to go to registry and get by TypeId
        let meta = self.versions.get(&spec).expect("Spec does not exist");
        // let types = types.get(&module).expect("Module not found");

        log::debug!("Types: {:?}", meta);
        // log::debug!("Type: {}", ty);
        // check if the concrete types are already included in
        // Metadata if not, fall back to type-metadata
        // exported types
    }

    /// Decode an extrinsic
    pub fn decode_extrinsic(&self, spec: SpecVersion, _data: Vec<u8>) {
        let meta = self.versions.get(&spec).expect("Spec does not exist");

        // should have a list of 'guess type' where
        // types like <T::Lookup as StaticLookup>::Source
        // are 'guessed' to be `Address`
        // this is sort of a hack
        // and should instead be handled in the definitions.json

        log::debug!("Types: {:?}", meta);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        decoder::{
            metadata::test_suite as meta_test_suite,
            Decoder
        },
        TypeDetective, test_suite, RustTypeMarker, Decodable
    };

    struct GenericTypes;
    impl TypeDetective for GenericTypes {
        fn get(&self, _module: &str, _ty: &str, _spec: usize, _chain: &str) -> Option<&dyn Decodable> {
            None
        }
        fn resolve(&self, _module: &str, _ty: &RustTypeMarker) -> Option<&RustTypeMarker> {
            None
        }
    }

    #[test]
    fn should_insert_metadata() {
        // let types = PolkadotTypes::new().unwrap();
        // types.get("balances", "BalanceLock", 1042, "kusama");

        let mut decoder = Decoder::new(GenericTypes);
        decoder.register_version(
            test_suite::mock_runtime(0).spec_version,
            meta_test_suite::test_metadata(),
        );
        decoder.register_version(
            test_suite::mock_runtime(1).spec_version,
            meta_test_suite::test_metadata(),
        );
        decoder.register_version(
            test_suite::mock_runtime(2).spec_version,
            meta_test_suite::test_metadata(),
        );
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(0).spec_version));
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(1).spec_version));
        assert!(decoder
            .versions
            .contains_key(&test_suite::mock_runtime(2).spec_version))
    }

    #[test]
    fn should_get_version_metadata() {
        // let types = PolkadotTypes::new().unwrap();
        let mut decoder = Decoder::new(GenericTypes);
        let rt_version = test_suite::mock_runtime(0);
        let meta = meta_test_suite::test_metadata();
        decoder.register_version(rt_version.spec_version.clone(), meta.clone());
        let _other_meta = decoder.get_version_metadata(rt_version.spec_version);
        assert_eq!(Some(&meta), _other_meta.clone())
    }
}
