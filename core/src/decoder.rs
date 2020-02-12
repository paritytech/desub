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

use super::metadata::{Metadata as SubstrateMetadata, ModuleMetadata};
use runtime_version::RuntimeVersion;
use std::collections::HashMap;
use std::rc::Rc;

type SpecVersion = u32;
/// Decoder for substrate types
///
/// hold information about the Runtime Metadata
/// and maps types inside the runtime metadata to self-describing types in
/// type-metadata
#[derive(Debug)]
pub struct Decoder {
    // reference to an item in 'versions' vector
    // NOTE: possibly a concurrent HashMap
    versions: HashMap<SpecVersion, SubstrateMetadata>,
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

impl Decoder {
    /// Create a new instance of Decoder
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// Insert a Metadata with Version attached
    /// If version exists, it's corresponding metadata will be updated
    pub fn register_version(
        &mut self,
        version: SpecVersion,
        metadata: SubstrateMetadata,
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
    fn get_version_metadata(&self, version: &SpecVersion) -> Option<&SubstrateMetadata> {
        self.versions.get(version)
    }

    #[allow(dead_code)]
    /// Verifies if all generic types of 'RuntimeMetadata' are present
    fn verify(&self) -> bool {
        // TODO: implement
        unimplemented!()
    }

    /// dynamically Decode a SCALE-encoded byte string into it's concrete rust
    /// types
    pub fn decode(&self, spec: SpecVersion, module: String, ty: String, data: Vec<u8>) {
        // have to go to registry and get by TypeId
        let meta = self.versions.get(&spec).expect("Spec does not exist");
        // let types = types.get(&module).expect("Module not found");

        log::debug!("Types: {:?}", meta);
        // log::debug!("Type: {}", ty);
        // check if the concrete types are already included in
        // RawSubstrateMetadata if not, fall back to type-metadata
        // exported types
    }

    /// Decode an extrinsic
    pub fn decode_extrinsic(_ty: String, _spec: SpecVersion, _data: Vec<u8>) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::test_suite as meta_test_suite;
    use crate::test_suite;

    #[allow(dead_code)]
    pub struct TestType {
        foo: u8,
        name: String,
    }

    #[allow(dead_code)]
    pub struct TestType2 {
        super_simple_type: u8,
        some_kind_of_name: String,
        first_test_struct: TestType,
    }

    #[test]
    fn should_insert_metadata() {
        let mut decoder = Decoder::new();
        decoder.insert_version(SubstrateMetadata {
            version: test_suite::mock_runtime(0),
            metadata: meta_test_suite::test_metadata(),
        });
        decoder.insert_version(SubstrateMetadata {
            version: test_suite::mock_runtime(1),
            metadata: meta_test_suite::test_metadata(),
        });
        decoder.insert_version(SubstrateMetadata {
            version: test_suite::mock_runtime(2),
            metadata: meta_test_suite::test_metadata(),
        });
        println!("{:#?}", decoder);
    }

    trait TestTrait {
        type Moment: Copy + Clone + Default;
    }

    struct TestTraitImpl;
    impl TestTrait for TestTraitImpl {
        type Moment = u32;
    }

    trait TestTrait2 {
        type Precision: Copy + Clone + Default;
    }

    struct TestTraitImpl2;
    impl TestTrait2 for TestTraitImpl2 {
        type Precision = i128;
    }

    struct TestEvent {
        some_str: String,
        some_num: u32,
    }

    #[test]
    fn should_get_version_metadata() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        let meta = meta_test_suite::test_metadata();
        decoder.register_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: meta.clone(),
        });
        let _other_meta = decoder.get_version_metadata(&rt_version);
        assert_eq!(Some(meta), _other_meta.clone())
    }
}
