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

//! Stucture for registering substrate types

// use super::metadata::{Metadata as RawSubstrateMetadata, ModuleMetadata};
use type_metadata::{
    form::{CompactForm, Form, MetaForm},
    IntoCompact, Metadata, Namespace, Registry,
};

/// One substrate type
/// with the associated bytes
/// augmented with type-metadata in the case of generic type definitions
/// not totally handled by substrate
#[derive(Debug)]
pub struct SubstrateType {
    /// vector holding generic type definitions of the runtime
    meta: SubstrateMetaType<CompactForm>,
    /// raw data of the type
    data: Vec<u8>,
}

impl SubstrateType {
    /// Builder for registering types from the runtime into the registry
    /// only types that are defined within runtime module trait definitions
    /// and types that are custom structs need be included
    ///
    /// Type definitions are matched against RuntimeMetadataPrefixed
    /// so that their definitions can be decoded during runtime with
    /// SCALE codec
    // TODO Should return an error, not panic!
    pub fn new<T>(
        &mut self,
        type_name: &'static str,
        data: &[u8],
        registry: &mut Registry,
    ) -> Self
    where
        T: Metadata,
    {
        let meta =
            SubstrateMetaType::with_name_str::<T>(type_name).into_compact(registry);
        Self {
            meta,
            data: data.to_vec(),
        }
    }

    pub fn downcast<T>(&self) -> T {
        unimplemented!()
    }

    pub fn as_json(&self) -> String {
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
    use crate::metadata::tests;
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
            metadata: tests::test_metadata(),
        });
        decoder.insert_version(SubstrateMetadata {
            version: test_suite::mock_runtime(1),
            metadata: tests::test_metadata(),
        });
        decoder.insert_version(SubstrateMetadata {
            version: test_suite::mock_runtime(2),
            metadata: tests::test_metadata(),
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
        let meta = tests::test_metadata();
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
            metadata: tests::test_metadata(),
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
    fn should_panic_on_nonexistant_type() {
        let mut decoder = Decoder::new();
        let rt_version = test_suite::mock_runtime(0);
        decoder.insert_version(SubstrateMetadata {
            version: rt_version.clone(),
            metadata: tests::test_metadata(),
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
            metadata: tests::test_metadata(),
        });

        decoder.register::<u32, _>(&rt_version, "IDontExist", "T::Moment");
    }
}
