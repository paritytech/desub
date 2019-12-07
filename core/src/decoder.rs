use crate::{error::Error, types::Decodable};
use codec::{Decode, Encode, FullCodec, Input};
use runtime_metadata::RuntimeMetadataPrefixed;
use serde::Serialize;
use std::{any::Any, collections::HashMap, rc::Rc};
use type_metadata::{
    form::{CompactForm, Form, MetaForm},
    IntoCompact, MetaType, Namespace, Registry, TypeId,
};

pub struct SubstrateMetaType<F: Form = MetaForm> {
    name: String,
    ty: F::TypeId,
    display: Namespace<F>,
}

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

    pub fn register<T>(&mut self, name: &str) {
        self.types.insert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_types() {
        let reg = Registry::new();
        pub struct TestType {
            foo: u8,
            name: String,
        }
        let t = MetaType::new::<TestType>();
        reg.register_type(&t);
    }

    #[test]
    #[should_panic]
    fn it_works_too() {
        assert!(1 + 1 == 3)
    }
}
