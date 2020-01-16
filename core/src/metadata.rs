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

// taken directly and modified from substrate-subxt:
// https://github.com/paritytech/substrate-subxt

use codec::{Decode, Encode, EncodeAsRef, HasCompact};
use failure::Fail;
use runtime_metadata::{
    DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed, StorageEntryModifier,
    StorageEntryType, StorageHasher, META_RESERVED,
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    marker::PhantomData,
    rc::Rc,
    str::FromStr,
};
use substrate_primitives::storage::StorageKey;

/// Newtype struct around a Vec<u8> (vector of bytes)
#[derive(Clone)]
pub struct Encoded(pub Vec<u8>);

impl Encode for Encoded {
    fn encode(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

pub fn compact<T: HasCompact>(t: T) -> Encoded {
    let encodable: <<T as HasCompact>::Type as EncodeAsRef<'_, T>>::RefType =
        From::from(&t);
    Encoded(encodable.encode())
}

#[derive(Debug, Clone, derive_more::Display)]
pub enum MetadataError {
    ModuleNotFound(String),
    CallNotFound(&'static str),
    EventNotFound(u8),
    StorageNotFound(&'static str),
    StorageTypeError,
    MapValueTypeError,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    modules: HashMap<String, Rc<ModuleMetadata>>,
    modules_by_event_index: HashMap<u8, String>,
}

impl Metadata {
    /// returns an iterate over all Modules
    pub fn modules(&self) -> impl Iterator<Item = &Rc<ModuleMetadata>> {
        self.modules.values()
    }

    /// returns a weak reference to a module from it's name
    pub fn module<S>(&self, name: S) -> Result<Rc<ModuleMetadata>, MetadataError>
    where
        S: ToString,
    {
        let name = name.to_string();
        self.modules
            .get(&name)
            .ok_or(MetadataError::ModuleNotFound(name))
            .map(|m| (*m).clone())
    }

    /// get the name of a module given it's event index
    pub fn module_name(&self, module_index: u8) -> Result<String, MetadataError> {
        self.modules_by_event_index
            .get(&module_index)
            .cloned()
            .ok_or(MetadataError::EventNotFound(module_index))
    }

    /// print out a detailed but human readable description of the module
    /// metadata
    pub fn detailed_pretty(&self) -> String {
        let mut string = String::new();
        for (name, module) in &self.modules {
            string.push_str(name.as_str());
            string.push('\n');
            for (storage, meta) in &module.storage {
                string.push_str(" S  ");
                string.push_str(storage.as_str());
                string.push_str(format!(" TYPE {:?}", meta.ty).as_str());
                string.push_str(format!(" MOD {:?}", meta.modifier).as_str());
                string.push('\n');
            }
            for (call, _) in &module.calls {
                string.push_str(" C  ");
                string.push_str(call.as_str());
                string.push('\n');
            }
            for (_, event) in &module.events {
                string.push_str(" E  ");
                string.push_str(event.name.as_str());
                string.push('\n');
            }
        }
        string
    }

    /// print out a human readable description of the metadata
    pub fn pretty(&self) -> String {
        let mut string = String::new();
        for (name, module) in &self.modules {
            string.push_str(name.as_str());
            string.push('\n');
            for (storage, _) in &module.storage {
                string.push_str(" s  ");
                string.push_str(storage.as_str());
                string.push('\n');
            }
            for (call, _) in &module.calls {
                string.push_str(" c  ");
                string.push_str(call.as_str());
                string.push('\n');
            }
            for (_, event) in &module.events {
                string.push_str(" e  ");
                string.push_str(event.name.as_str());
                string.push('\n');
            }
        }
        string
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMetadata {
    /// index of the module within StorageMetadata 'Entries'
    index: u8,
    /// name of the module
    name: String,
    /// Name of storage entry -> Metadata of storage entry
    storage: HashMap<String, StorageMetadata>,
    /// Calls in the module, CallName -> encoded calls
    calls: HashMap<String, Vec<u8>>,
    events: HashMap<u8, ModuleEventMetadata>,
    // constants
}

impl ModuleMetadata {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// return the SCALE-encoded Call with parameters appended and parameters
    pub fn call<T: Encode>(
        &self, function: &'static str, params: T,
    ) -> Result<Encoded, MetadataError> {
        let fn_bytes = self
            .calls
            .get(function)
            .ok_or(MetadataError::CallNotFound(function))?;
        let mut bytes = vec![self.index];
        bytes.extend(fn_bytes);
        bytes.extend(params.encode());
        Ok(Encoded(bytes))
    }

    /// Return a storage entry by its key
    pub fn storage(&self, key: &'static str) -> Result<&StorageMetadata, MetadataError> {
        self.storage
            .get(key)
            .ok_or(MetadataError::StorageNotFound(key))
    }

    /// an iterator over all possible events for this module
    pub fn events(&self) -> impl Iterator<Item = &ModuleEventMetadata> {
        self.events.values()
    }

    // TODO Transfer to Subxt
    /// iterator over all possible calls in this module
    pub fn calls(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.calls.iter()
    }

    /// iterator over all storage keys in this module
    pub fn storage_keys(&self) -> impl Iterator<Item = (&String, &StorageMetadata)> {
        self.storage.iter()
    }

    /// get an event by its index in the module
    pub fn event(&self, index: u8) -> Result<&ModuleEventMetadata, MetadataError> {
        self.events
            .get(&index)
            .ok_or(MetadataError::EventNotFound(index))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StorageMetadata {
    prefix: String,
    modifier: StorageEntryModifier,
    ty: StorageEntryType,
    default: Vec<u8>,
    documentation: Vec<String>,
}

impl StorageMetadata {
    pub fn get_map<K: Encode, V: Decode + Clone>(
        &self,
    ) -> Result<StorageMap<K, V>, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { hasher, .. } => {
                let prefix = self.prefix.as_bytes().to_vec();
                let hasher = hasher.to_owned();
                let default = Decode::decode(&mut &self.default[..])
                    .map_err(|_| MetadataError::MapValueTypeError)?;
                Ok(StorageMap {
                    _marker: PhantomData,
                    prefix,
                    hasher,
                    default,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StorageMap<K, V> {
    _marker: PhantomData<K>,
    prefix: Vec<u8>,
    hasher: StorageHasher,
    default: V,
}

impl<K: Encode, V: Decode + Clone> StorageMap<K, V> {
    pub fn key(&self, key: K) -> StorageKey {
        let mut bytes = self.prefix.clone();
        bytes.extend(key.encode());
        let hash = match self.hasher {
            StorageHasher::Blake2_128 => {
                substrate_primitives::blake2_128(&bytes).to_vec()
            }
            StorageHasher::Blake2_256 => {
                substrate_primitives::blake2_256(&bytes).to_vec()
            }
            StorageHasher::Blake2_128Concat => {
                substrate_primitives::blake2_128(&bytes).to_vec()
            }
            StorageHasher::Twox128 => substrate_primitives::twox_128(&bytes).to_vec(),
            StorageHasher::Twox256 => substrate_primitives::twox_256(&bytes).to_vec(),
            StorageHasher::Twox64Concat => substrate_primitives::twox_64(&bytes).to_vec(),
        };
        StorageKey(hash)
    }

    pub fn default(&self) -> V {
        self.default.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleEventMetadata {
    pub name: String,
    pub(crate) arguments: HashSet<EventArg>,
}

impl ModuleEventMetadata {
    pub fn arguments(&self) -> Vec<EventArg> {
        self.arguments.iter().cloned().collect()
    }
}

/// Naive representation of event argument types, supports current set of
/// substrate EventArg types. If and when Substrate uses `type-metadata`, this
/// can be replaced.
///
/// Used to calculate the size of a instance of an event variant without having
/// the concrete type, so the raw bytes can be extracted from the encoded
/// `Vec<EventRecord<E>>` (without `E` defined).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventArg {
    Primitive(String),
    Vec(Box<EventArg>),
    Tuple(Vec<EventArg>),
}

impl FromStr for EventArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Vec<") {
            if s.ends_with('>') {
                Ok(EventArg::Vec(Box::new(s[4 .. s.len() - 1].parse()?)))
            } else {
                Err(Error::InvalidEventArg(
                    s.to_string(),
                    "Expected closing `>` for `Vec`",
                ))
            }
        } else if s.starts_with("(") {
            if s.ends_with(")") {
                let mut args = Vec::new();
                for arg in s[1 .. s.len() - 1].split(',') {
                    let arg = arg.trim().parse()?;
                    args.push(arg)
                }
                Ok(EventArg::Tuple(args))
            } else {
                Err(Error::InvalidEventArg(
                    s.to_string(),
                    "Expecting closing `)` for tuple",
                ))
            }
        } else {
            Ok(EventArg::Primitive(s.to_string()))
        }
    }
}

impl EventArg {
    /// Returns all primitive types for this EventArg
    pub fn primitives(&self) -> Vec<String> {
        match self {
            EventArg::Primitive(p) => vec![p.clone()],
            EventArg::Vec(arg) => arg.primitives(),
            EventArg::Tuple(args) => {
                let mut primitives = Vec::new();
                for arg in args {
                    primitives.extend(arg.primitives())
                }
                primitives
            }
        }
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Invalid Prefix")]
    InvalidPrefix,
    #[fail(display = " Invalid Version")]
    InvalidVersion,
    #[fail(display = "Expected Decoded")]
    ExpectedDecoded,
    #[fail(display = "Invalid Event {}:{}", _0, _1)]
    InvalidEventArg(String, &'static str),
}

impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
    type Error = Error;

    fn try_from(metadata: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
        if metadata.0 != META_RESERVED {
            Err(Error::InvalidPrefix)?;
        }
        let meta = match metadata.1 {
            RuntimeMetadata::V10(meta) => meta,
            _ => Err(Error::InvalidVersion)?,
        };
        let mut modules = HashMap::new();
        let mut modules_by_event_index = HashMap::new();
        let mut event_index = 0;
        for (i, module) in convert(meta.modules)?.into_iter().enumerate() {
            let module_name = convert(module.name.clone())?;
            let module_metadata = convert_module(i, module)?;
            // modules with no events have no corresponding definition in the
            // top level enum
            if !module_metadata.events.is_empty() {
                modules_by_event_index.insert(event_index, module_name.clone());
                event_index = event_index + 1;
            }
            modules.insert(module_name, Rc::new(module_metadata));
        }
        Ok(Metadata {
            modules,
            modules_by_event_index,
        })
    }
}

fn convert<B: 'static, O: 'static>(dd: DecodeDifferent<B, O>) -> Result<O, Error> {
    match dd {
        DecodeDifferent::Decoded(value) => Ok(value),
        _ => Err(Error::ExpectedDecoded),
    }
}

fn convert_module(
    index: usize, module: runtime_metadata::ModuleMetadata,
) -> Result<ModuleMetadata, Error> {
    let mut storage_map = HashMap::new();
    if let Some(storage) = module.storage {
        let storage = convert(storage)?;
        let prefix = convert(storage.prefix)?;
        for entry in convert(storage.entries)?.into_iter() {
            let entry_name = convert(entry.name.clone())?;
            let entry_prefix = format!("{} {}", prefix, entry_name);
            let entry = convert_entry(entry_prefix, entry)?;
            storage_map.insert(entry_name, entry);
        }
    }
    let mut call_map = HashMap::new();
    if let Some(calls) = module.calls {
        for (index, call) in convert(calls)?.into_iter().enumerate() {
            // HERE modify
            let name = convert(call.name)?;
            call_map.insert(name, vec![index as u8]);
        }
    }
    let mut event_map = HashMap::new();
    if let Some(events) = module.event {
        for (index, event) in convert(events)?.into_iter().enumerate() {
            event_map.insert(index as u8, convert_event(event)?);
        }
    }
    Ok(ModuleMetadata {
        index: index as u8,
        name: convert(module.name)?,
        storage: storage_map,
        calls: call_map,
        events: event_map,
    })
}

fn convert_event(
    event: runtime_metadata::EventMetadata,
) -> Result<ModuleEventMetadata, Error> {
    let name = convert(event.name)?;
    let mut arguments = HashSet::new();
    for arg in convert(event.arguments)? {
        let arg = arg.parse::<EventArg>()?;
        arguments.insert(arg);
    }
    Ok(ModuleEventMetadata { name, arguments })
}

fn convert_entry(
    prefix: String, entry: runtime_metadata::StorageEntryMetadata,
) -> Result<StorageMetadata, Error> {
    let default = convert(entry.default)?;
    let documentation = convert(entry.documentation)?;
    Ok(StorageMetadata {
        prefix,
        modifier: entry.modifier,
        ty: entry.ty,
        default,
        documentation: documentation
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    type DecodeDifferentStr = DecodeDifferent<&'static str, String>;

    pub fn test_metadata() -> Metadata {
        Metadata {
            modules: module_metadata_mock(),
            modules_by_event_index: HashMap::new(), // not testing this
        }
    }

    fn module_metadata_mock() -> HashMap<String, Rc<ModuleMetadata>> {
        let mut map = HashMap::new();

        map.insert(
            "TestModule0".to_string(),
            Rc::new(ModuleMetadata {
                index: 0,
                name: "TestModule0".to_string(),
                storage: storage_mock(),
                calls: call_mock(),
                events: event_mock(),
            }),
        );

        map.insert(
            "TestModule1".to_string(),
            Rc::new(ModuleMetadata {
                index: 1,
                name: "TestModule1".to_string(),
                storage: storage_mock(),
                calls: call_mock(),
                events: event_mock(),
            }),
        );

        map.insert(
            "TestModule2".to_string(),
            Rc::new(ModuleMetadata {
                index: 2,
                name: "TestModule2".to_string(),
                storage: storage_mock(),
                calls: call_mock(),
                events: event_mock(),
            }),
        );

        map
    }

    fn storage_mock() -> HashMap<String, StorageMetadata> {
        let mut map = HashMap::new();
        let moment = DecodeDifferentStr::Decoded("T::Moment".to_string());
        let usize_t = DecodeDifferentStr::Decoded("usize".to_string());
        // TODO supposed to be float type but type-metadata does not support
        // floats yet
        let precision = DecodeDifferentStr::Decoded("F::Precision".to_string());

        map.insert(
            "TestStorage0".to_string(),
            StorageMetadata {
                prefix: "TestStorage0".to_string(),
                modifier: StorageEntryModifier::Default,
                ty: StorageEntryType::Plain(moment.clone()),
                default: vec![112, 23, 0, 0, 0, 0, 0, 0],
                documentation: vec!["Some Kind of docs".to_string()],
            },
        );

        map.insert(
            "TestStorage1".to_string(),
            StorageMetadata {
                prefix: "TestStorage1".to_string(),
                modifier: StorageEntryModifier::Default,
                ty: StorageEntryType::Plain(usize_t),
                default: vec![0, 0, 0, 0, 0, 0, 0, 0],
                documentation: vec!["Some Kind of docs 2".to_string()],
            },
        );

        map.insert(
            "TestStorage2".to_string(),
            StorageMetadata {
                prefix: "TestStorage2".to_string(),
                modifier: StorageEntryModifier::Optional,
                ty: StorageEntryType::Plain(moment),
                default: vec![0, 0, 0, 0, 0, 0, 0, 0],
                documentation: vec!["Some Kind of docs 2".to_string()],
            },
        );

        map.insert(
            "TestStorage3".to_string(),
            StorageMetadata {
                prefix: "TestStorage3".to_string(),
                modifier: StorageEntryModifier::Optional,
                ty: StorageEntryType::Plain(precision),
                default: vec![0, 0, 0, 0, 0, 0, 0, 0],
                documentation: vec!["Some Kind of docs 3".to_string()],
            }
        );
        map
    }

    fn call_mock() -> HashMap<String, Vec<u8>> {
        let mut map = HashMap::new();
        map.insert("TestCall0".to_string(), vec![01, 02, 03, 04, 05]);
        map.insert("TestCall1".to_string(), vec![11, 12, 13, 14, 15, 16, 17]);
        map.insert(
            "TestCall2".to_string(),
            vec![21, 22, 23, 24, 25, 26, 27, 28, 29],
        );
        map.insert(
            "TestCall3".to_string(),
            vec![31, 32, 33, 34, 35, 36, 37, 38, 39],
        );
        map
    }

    fn event_mock() -> HashMap<u8, ModuleEventMetadata> {
        let mut map = HashMap::new();

        let event_arg_0 = EventArg::Primitive("TestEvent0".to_string());
        let event_arg_1 = EventArg::Primitive("TestEvent1".to_string());
        let event_arg_2 = EventArg::Primitive("TestEvent2".to_string());

        let mut arguments = HashSet::new();
        arguments.insert(event_arg_0);
        arguments.insert(event_arg_1);
        arguments.insert(event_arg_2);
        let module_event_metadata = ModuleEventMetadata {
            name: "TestEvent0".to_string(),
            arguments,
        };

        map.insert(0, module_event_metadata);
        map
    }
}
