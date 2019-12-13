use runtime_metadata::RuntimeMetadataPrefixed;
use std::collections::HashMap;
use type_metadata::{
    form::{CompactForm, Form, MetaForm},
    Metadata, IntoCompact,
    Namespace, Registry
};

pub struct Decoder {
    types: HashMap<String, SubstrateMetaType<CompactForm>>,
    registry: Registry,
    metadata: RuntimeMetadataPrefixed,
}

impl Decoder {
    pub fn new(metadata: RuntimeMetadataPrefixed) -> Self {
        Self {
            types: HashMap::new(),
            registry: Registry::new(),
            metadata,
        }
    }

    pub fn register<T>(&mut self, name: &'static str)
    where
        T: Metadata,
    {
        self.types.insert(name.into(), SubstrateMetaType::with_name_str::<T>(name).into_compact(&mut self.registry));
    }
}



/// A type from substrate metadata.
///
/// This contains the actual type as well as an optional compile-time
/// known displayed representation of the type. This is useful for cases
/// where the type is used through a type alias in order to provide
/// information about the alias name.
/// The name of the type from substrates Metadata, however similar to `display_name`
/// is not optional
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
        S: IntoIterator<Item = <MetaForm as Form>::String>
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
            display_name: self.display_name.into_compact(registry)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        first_test_struct: TestType
    }

    #[test]
    fn add_types() {
        let mut reg = Registry::new();

        let t: SubstrateMetaType<_> = SubstrateMetaType::with_name_str::<TestType>("TestType");
        println!("{:?}", t);
        println!("================");

        let x: SubstrateMetaType<CompactForm> = SubstrateMetaType::with_name_str::<TestType2>("TestType").into_compact(&mut reg);
        println!("PRELUDE: {:?}", Namespace::prelude());
        println!("{:#?}\n\n", x);
        println!("{:#?}", reg);
        println!("JSON\n\n");
        let serialized = serde_json::to_string_pretty(&reg).unwrap();
        println!("{}", serialized);
    }
}
