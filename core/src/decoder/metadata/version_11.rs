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

use super::{
    CallArgMetadata, CallMetadata, Error, EventArg, Metadata, ModuleEventMetadata,
    ModuleMetadata, StorageMetadata,
};
use crate::regex;
use runtime_metadata_latest::{
    DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed, META_RESERVED,
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
    type Error = Error;

    fn try_from(metadata: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
        if metadata.0 != META_RESERVED {
            // 'meta' warn endiannes
            return Err(Error::InvalidPrefix);
        }
        let meta = match metadata.1 {
            RuntimeMetadata::V11(meta) => meta,
            _ => return Err(Error::InvalidVersion),
        };
        let mut modules = HashMap::new();
        let (mut modules_by_event_index, mut modules_by_call_index) =
            (HashMap::new(), HashMap::new());
        let (mut event_index, mut call_index) = (0, 0);
        for (i, module) in convert(meta.modules)?.into_iter().enumerate() {
            let module_name = convert(module.name.clone())?;
            if module.calls.is_some() {
                modules_by_call_index.insert(call_index, module_name.clone());
                call_index += 1;
            }
            if module.event.is_none() {
                modules_by_event_index.insert(event_index, module_name.clone());
                event_index += 1;
            }
            let module_metadata = convert_module(i, module)?;
            modules.insert(module_name, module_metadata);
        }
        Ok(Metadata {
            modules,
            modules_by_event_index,
            modules_by_call_index,
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
    module: runtime_metadata_latest::ModuleMetadata,
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
            let args = convert(call.arguments)?
                .iter()
                .map(|a| {
                    let ty = convert(a.ty.clone())?;
                    let name = convert(a.name.clone())?;
                    let arg = CallArgMetadata {
                        name,
                        ty: regex::parse(&ty).ok_or(Error::InvalidType(ty))?,
                    };
                    Ok(arg)
                })
                .collect::<Result<Vec<CallArgMetadata>, Error>>()?;
            let meta = CallMetadata {
                name: name.clone(),
                index: index as u8,
                arguments: args,
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
    event: runtime_metadata_latest::EventMetadata,
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
    entry: runtime_metadata_latest::StorageEntryMetadata,
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
