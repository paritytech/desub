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

use super::{
    Error, EventArg, Metadata, ModuleEventMetadata, ModuleMetadata, StorageMetadata,
    CallMetadata, CallArgMetadata
};
use crate::regex;
use runtime_metadata08::{
    DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed, StorageEntryModifier,
    StorageEntryType, StorageHasher, META_RESERVED,
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    rc::Rc,
};

type DecodeDifferentStr = DecodeDifferent<&'static str, String>;
type LatestDecodeDifferentStr =
    runtime_metadata_latest::DecodeDifferent<&'static str, String>;

impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
    type Error = Error;

    fn try_from(metadata: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
        if metadata.0 != META_RESERVED {
            // 'meta' warn endiannes
            return Err(Error::InvalidPrefix);
        }
        let meta = match metadata.1 {
            RuntimeMetadata::V8(meta) => meta,
            _ => return Err(Error::InvalidVersion),
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
                event_index += 1;
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
    index: usize,
    module: runtime_metadata08::ModuleMetadata,
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
            let name = convert(call.name)?;
            let index = vec![index as u8];
            let args = convert(call.arguments)?.iter().map(|a| {
                let ty = convert(a.ty.clone())?;
                let name = convert(a.name.clone())?;
                let arg = CallArgMetadata {
                    name,
                    ty: regex::parse(&ty).ok_or(Error::InvalidType(ty))?
                };
                Ok(arg)
            }).collect::<Result<Vec<CallArgMetadata>, Error>>()?;
            let meta = CallMetadata {
                name: name.clone(),
                index,
                arguments: args
            };
            call_map.insert(name, meta);
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
    event: runtime_metadata08::EventMetadata,
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
    prefix: String,
    entry: runtime_metadata08::StorageEntryMetadata,
) -> Result<StorageMetadata, Error> {
    let default = convert(entry.default)?;
    let documentation = convert(entry.documentation)?;
    Ok(StorageMetadata {
        prefix,
        modifier: StorageEntryModifierTemp(entry.modifier).into(),
        ty: StorageEntryTypeTemp(entry.ty).into(),
        default,
        documentation: documentation
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    })
}

/// Temporary struct for converting between `StorageEntryModifier`
/// and `runtime_metadata_latest::StorageEntryModifier`
struct StorageEntryModifierTemp(StorageEntryModifier);
impl From<StorageEntryModifierTemp> for runtime_metadata_latest::StorageEntryModifier {
    fn from(
        entry: StorageEntryModifierTemp,
    ) -> runtime_metadata_latest::StorageEntryModifier {
        let entry = entry.0;
        match entry {
            StorageEntryModifier::Optional => {
                runtime_metadata_latest::StorageEntryModifier::Optional
            }
            StorageEntryModifier::Default => {
                runtime_metadata_latest::StorageEntryModifier::Default
            }
        }
    }
}

/// Temporary struct for converting between `StorageEntryType`
/// and `runtime_metadata_latest::StorageEntryType`
struct StorageEntryTypeTemp(StorageEntryType);
impl From<StorageEntryTypeTemp> for runtime_metadata_latest::StorageEntryType {
    fn from(entry: StorageEntryTypeTemp) -> runtime_metadata_latest::StorageEntryType {
        let entry = entry.0;
        match entry {
            StorageEntryType::Plain(d) => {
                runtime_metadata_latest::StorageEntryType::Plain(
                    TempDecodeDifferentStr(d).into(),
                )
            }
            StorageEntryType::Map {
                hasher,
                key,
                value,
                is_linked,
            } => runtime_metadata_latest::StorageEntryType::Map {
                hasher: TempStorageHasher(hasher).into(),
                key: TempDecodeDifferentStr(key).into(),
                value: TempDecodeDifferentStr(value).into(),
                is_linked,
            },
            StorageEntryType::DoubleMap {
                hasher,
                key1,
                key2,
                value,
                key2_hasher,
            } => runtime_metadata_latest::StorageEntryType::DoubleMap {
                hasher: TempStorageHasher(hasher).into(),
                key1: TempDecodeDifferentStr(key1).into(),
                key2: TempDecodeDifferentStr(key2).into(),
                value: TempDecodeDifferentStr(value).into(),
                key2_hasher: TempStorageHasher(key2_hasher).into(),
            },
        }
    }
}

/// Temprorary struct for converting between `StorageHasher` and
/// `runtime_metadata_latest::StorageHasher`
struct TempStorageHasher(StorageHasher);
impl From<TempStorageHasher> for runtime_metadata_latest::StorageHasher {
    fn from(hasher: TempStorageHasher) -> runtime_metadata_latest::StorageHasher {
        let hasher = hasher.0;
        match hasher {
            StorageHasher::Blake2_128 => {
                runtime_metadata_latest::StorageHasher::Blake2_128
            }
            StorageHasher::Blake2_256 => {
                runtime_metadata_latest::StorageHasher::Blake2_256
            }
            StorageHasher::Twox128 => runtime_metadata_latest::StorageHasher::Twox128,
            StorageHasher::Twox256 => runtime_metadata_latest::StorageHasher::Twox256,
            StorageHasher::Twox64Concat => {
                runtime_metadata_latest::StorageHasher::Twox64Concat
            }
        }
    }
}

/// Temporary struct for converting between `DecodeDifferentStr` and
/// `DecodeDifferentStrLatest`
struct TempDecodeDifferentStr(DecodeDifferentStr);
impl From<TempDecodeDifferentStr> for LatestDecodeDifferentStr {
    fn from(decode_str: TempDecodeDifferentStr) -> LatestDecodeDifferentStr {
        let decode_str = decode_str.0;
        match decode_str {
            DecodeDifferent::Encode(b) => {
                runtime_metadata_latest::DecodeDifferent::Encode(b)
            }
            DecodeDifferent::Decoded(o) => {
                runtime_metadata_latest::DecodeDifferent::Decoded(o)
            }
        }
    }
}
