use crate::{error::Error, types::Decodable};
use codec::{Decode, Encode, FullCodec, Input};
use runtime_metadata::RuntimeMetadataPrefixed;
use serde::Serialize;
use std::{any::Any, collections::HashMap, rc::Rc};

pub struct Decoder<Ty: FullCodec> {
    types: HashMap<String, Box<dyn Decodable<T = Box<Ty>>>>,
    // meta: RuntimeMetadataPrefixed,
}

impl<Ty: FullCodec> Decoder<Ty> {
    pub fn new(/*_meta: RuntimeMetadataPrefixed*/) -> Self {
        Self {
            // meta,
            types: HashMap::new(),
        }
    }

    pub fn register_type<R>(&mut self, name: &str, ty: R)
    where
        R: Decodable<T = Box<Ty>> + 'static,
    {
        self.types.insert(name.to_string(), Box::new(ty));
    }

    pub fn decode(&self, name: &str, input: Vec<u8>) -> Option<Result<Box<Ty>, Error>> {
        if let Some(base) = self.types.get(name) {
            Some(base.decode(input).map_err(Into::into))
        } else {
            None
        }
    }
}

pub trait NodeTrait: Any {
    fn next(&self) -> Option<Rc<Box<dyn NodeTrait>>>;
    fn down(&self) -> Box<dyn Any>;
}

pub struct Node<T: FullCodec> {
    inner: T,
    list: Option<Box<dyn NodeTrait>>,
}

impl<T> NodeTrait for Node<T>
where
    T: FullCodec + 'static,
{
    fn next(&self) -> Option<Rc<Box<dyn NodeTrait>>> {
        self.list.as_ref().map(|l| l.next()).flatten()
    }
}

impl<T> Node<T>
where
    T: FullCodec,
{
    pub fn new(typedef: T) -> Self {
        Self {
            inner: typedef,
            list: None,
        }
    }

    pub fn append(&mut self, next: Box<dyn NodeTrait>) {
        self.list = Some(next);
    }
}

pub struct TypeList {
    head: Box<dyn NodeTrait>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn add_types() {}

    #[test]
    #[should_panic]
    fn it_works_too() {
        assert!(1 + 1 == 3)
    }
}
