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

use super::metadata::{Metadata as RawSubstrateMetadata, ModuleMetadata};
use runtime_version::RuntimeVersion;
use std::collections::HashMap;
use std::rc::Rc;
use type_metadata::{
    form::{CompactForm, Form, MetaForm},
    IntoCompact, Metadata, Namespace, Registry,
};

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
    types: HashMap<SpecVersion, HashMap<String, SubstrateMetaEntry>>,
    /// all supported versions
    versions: Vec<SubstrateMetadata>,
    /// the type registry cache
    registry: Registry,
}

/// holds one unit of metadata
/// the version of the metadata
/// and the metadata itself
#[derive(Debug)]
pub struct SubstrateMetadata {
    version: RuntimeVersion,
    metadata: RawSubstrateMetadata,
}

/// One entry of the substrate metadata
/// augmented with type-metadata in the case of generic type definitions
/// not totally handled by substrate
#[derive(Debug)]
pub struct SubstrateMetaEntry {
    /// vector holding generic type definitions of the runtime
    types: Vec<SubstrateMetaType<CompactForm>>,
    /// pointer to original metadata entry
    runtime_entry: Rc<ModuleMetadata>,
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
            types: HashMap::new(),
            versions: Vec::new(),
            registry: Registry::new(),
        }
    }

    /// register a version that this application will support
    ///
    /// register versions to associate exported metadata with a specific runtime
    /// ensuring that type definitions do not co-mingle with other runtime
    /// versions, if they are different
    ///
    /// # Note
    ///
    /// All versions should be registered before registering any types,
    /// lest desub will panic
    pub fn register_version(
        &mut self, metadata: RawSubstrateMetadata, version: RuntimeVersion,
    ) {
        self.insert_version(SubstrateMetadata {
            version, metadata,
        });
    }

    /// Builder for registering types from the runtime into the registry
    /// only types that are defined within runtime module trait definitions
    /// and types that are custom structs need be included
    ///
    /// Type definitions are matched against RawSubstrateMetadata
    /// so that their definitions can be decoded during runtime with
    /// SCALE codec
    ///
    /// # Panics
    ///
    /// panics when metadata coinciding with runtime version cannot be found
    /// panics when module cannot be found in the metadata
    // TODO Should return an error, not panic!
    pub fn register<T, S>(
        &mut self, version: &RuntimeVersion, module: S, type_name: &'static str,
    ) where
        S: Into<String>,
        T: Metadata,
    {
        let module: String = module.into();
        let raw_metadata = self
            .get_version_metadata(version)
            .module(&module)
            .expect("metatadata not found"); // TODO remove panic

        let type_map = match self.types.get_mut(&version.spec_version) {
            Some(m) => m,
            None => {
                // create a new hashmap for a new runtime version
                self.types
                    .insert(version.spec_version.clone(), HashMap::new());
                self.types
                    .get_mut(&version.spec_version)
                    .expect("Inserted and immediately read")
            }
        };

        // TODO check that the type_name exists in raw_metadata
        // to prevent bloating of the data structure
        // we don't want non-existant types committed
        // fixes test `should_panic_on_nonexistant_type`

        if let Some(entry) = type_map.get_mut(&module) {
            entry.types.push(
                SubstrateMetaType::with_name_str::<T>(type_name)
                    .into_compact(&mut self.registry),
            )
        } else {
            let mut types = Vec::new();
            types.push(
                SubstrateMetaType::with_name_str::<T>(type_name)
                    .into_compact(&mut self.registry),
            );

            let entry = SubstrateMetaEntry {
                types,
                runtime_entry: raw_metadata,
            };
            type_map.insert(module.into(), entry);
        }
    }

    /// Internal API to insert a Metadata with Version attached into a sorted
    /// array
    ///
    /// # Note
    ///
    /// all version inserts should be done before any call to
    /// `get_version_metadata`
    fn insert_version(&mut self, sub_meta: SubstrateMetadata) {
        match self
            .versions
            .as_slice()
            .binary_search_by_key(&sub_meta.version.spec_version, |s| {
                s.version.spec_version
            }) {
            Ok(_) => (),
            Err(i) => self.versions.insert(i, sub_meta),
        }
    }

    /// internal api to get runtime version
    /// panics if a version is not found
    ///
    /// get runtime version in less than linear time with binary search
    ///
    /// # Panics
    ///
    /// panics if the given version does not exist
    fn get_version_metadata(&self, version: &RuntimeVersion) -> &RawSubstrateMetadata {
        match self
            .versions
            .as_slice()
            .binary_search_by_key(&version.spec_version, |s| s.version.spec_version)
        {
            Ok(v) => &self.versions[v].metadata,
            Err(_) => panic!("such a version does not exist"),
        }
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
        &self, spec: SpecVersion, module: String, ty: String, data: Vec<u8>,
    ) {
        // have to go to registry and get by TypeId
        let types = self.types.get(&spec).expect("Spec does not exist");
        let types = types.get(&module).expect("Module not found");

        log::debug!("Types: {:?}", types);
        log::debug!("Type: {}", ty);
        // check if the concrete types are already included in
        // RawSubstrateMetadata if not, fall back to type-metadata
        // exported types
    }

    /// Decode an extrinsic
    pub fn decode_extrinsic(_ty: String, _spec: SpecVersion, _data: Vec<u8>) {
        unimplemented!()
    }
}

/// A type from substrate metadata.
///
/// This contains the actual type as well as an optional compile-time
/// known displayed representation of the type. This is useful for cases
/// where the type is used through a type alias in order to provide
/// information about the alias name.
/// The name of the type from substrates Metadata, however similar to
/// `display_name` is not optional
#[derive(Debug)]
pub struct SubstrateMetaType<F: Form = MetaForm> {
    ty: F::TypeId,
    display_name: Namespace<F>,
}

// copied from ink!
// https://github.com/paritytech/ink/blob/master/abi/src/specs.rs#L596
impl SubstrateMetaType {

    /// Creates a new type specification without a display name.
    pub fn new<T>() -> Self
    where
        T: Metadata,
    {
        Self {
            ty: T::meta_type(),
            display_name: Namespace::prelude(),
        }
    }

    /// Creates a new type specification with a display name.
    ///
    /// The name is any valid Rust identifier or path.
    ///
    /// # Examples
    ///
    /// Valid display names are `foo`, `foo::bar`, `foo::bar::Baz`, etc.
    ///
    /// # Panics
    ///
    /// Panics if the given display name is invalid.
    pub fn with_name_str<T>(display_name: &'static str) -> Self
    where
        T: Metadata,
    {
        Self::with_name_segs::<T, _>(display_name.split("::"))
    }

    /// Creates a new type specification with a display name
    /// represented by the given path segments.
    ///
    /// The display name segments all must be valid Rust identifiers.
    ///
    /// # Examples
    ///
    /// Valid display names are `foo`, `foo::bar`, `foo::bar::Baz`, etc.
    ///
    /// # Panics
    ///
    /// Panics if the given display name is invalid.
    pub fn with_name_segs<T, S>(segments: S) -> Self
    where
        T: Metadata,
        S: IntoIterator<Item = <MetaForm as Form>::String>,
    {
        Self {
            ty: T::meta_type(),
            display_name: Namespace::new(segments).expect("display name is invalid"),
        }
    }
}

impl IntoCompact for SubstrateMetaType {
    type Output = SubstrateMetaType<CompactForm>;
    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        SubstrateMetaType {
            ty: registry.register_type(&self.ty),
            display_name: self.display_name.into_compact(registry),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::test_suite as meta_test_suite;
    use crate::test_suite;

    #[derive(Metadata)]
    #[allow(dead_code)]
    pub struct TestType {
        foo: u8,
        name: String,
    }

    #[derive(Metadata)]
    #[allow(dead_code)]
    pub struct TestType2 {
        super_simple_type: u8,
        some_kind_of_name: String,
        first_test_struct: TestType,
    }

    #[test]
    fn add_types() {
        let mut reg = Registry::new();

        let t: SubstrateMetaType<_> =
            SubstrateMetaType::with_name_str::<TestType>("TestType");
        println!("{:?}", t);
        println!("================");

        let x: SubstrateMetaType<CompactForm> =
            SubstrateMetaType::with_name_str::<TestType2>("TestType")
                .into_compact(&mut reg);
        println!("PRELUDE: {:?}", Namespace::prelude());
        println!("{:#?}\n\n", x);
        println!("{:#?}", reg);
        println!("JSON\n\n");
        let serialized = serde_json::to_string_pretty(&reg).unwrap();
        println!("{}", serialized);
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

    #[derive(Metadata)]
    struct TestEvent {
        some_str: String,
        some_num: u32,
    }

    #[test]
    fn should_get_version_metadata() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        let meta = meta_test_suite::test_metadata();
        decoder.insert_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: meta.clone(),
        });
        let _other_meta = decoder.get_version_metadata(&rt_version);
        assert_eq!(meta, _other_meta.clone())
    }

    #[test]
    fn should_register_types() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        decoder.insert_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: meta_test_suite::test_metadata(),
        });
        decoder.register::<<TestTraitImpl as TestTrait>::Moment, _>(
            &rt_version,
            "TestModule0",
            "T::Moment",
        );
        decoder.register::<<TestTraitImpl2 as TestTrait2>::Precision, _>(
            &rt_version,
            "TestModule0",
            "F::Precision",
        );
        decoder.register::<TestEvent, _>(&rt_version, "TestModule0", "TestEvent0");
        dbg!(&decoder);
    }

    #[test]
    #[should_panic]
    #[ignore] // not a feature yet
    // TODO: Make this a feature
    fn should_panic_on_nonexistant_type() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        decoder.insert_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: meta_test_suite::test_metadata(),
        });

        decoder.register::<u32, _>(&rt_version, "TestModule0", "R::IDontExist");
    }

    #[test]
    #[should_panic]
    fn should_panic_on_nonexistant_module() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        decoder.insert_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: meta_test_suite::test_metadata(),
        });

        decoder.register::<u32, _>(&rt_version, "IDontExist", "T::Moment");
    }
}
