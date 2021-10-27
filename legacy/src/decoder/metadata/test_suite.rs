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

//! Functions creating data to mock the `Metadata` struct

use super::*;
use crate::RustTypeMarker;
use std::sync::Arc;

pub fn test_metadata() -> Metadata {
	Metadata {
		modules: module_metadata_mock(),
		modules_by_event_index: HashMap::new(),
		modules_by_call_index: HashMap::new(),
		extrinsics: None,
	}
}

fn module_metadata_mock() -> HashMap<String, Arc<ModuleMetadata>> {
	let mut map = HashMap::new();

	map.insert(
		"TestModule0".to_string(),
		Arc::new(ModuleMetadata {
			index: 0,
			name: "TestModule0".to_string(),
			storage: storage_mock(),
			calls: call_mock(),
			events: event_mock(),
		}),
	);

	map.insert(
		"TestModule1".to_string(),
		Arc::new(ModuleMetadata {
			index: 1,
			name: "TestModule1".to_string(),
			storage: storage_mock(),
			calls: call_mock(),
			events: event_mock(),
		}),
	);

	map.insert(
		"TestModule2".to_string(),
		Arc::new(ModuleMetadata {
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
	let moment = RustTypeMarker::TypePointer("T::Moment".to_string());
	let precision = RustTypeMarker::U32;
	let u64_t = RustTypeMarker::U64;

	map.insert(
		"TestStorage0".to_string(),
		StorageMetadata {
			prefix: "TestStorage0".to_string(),
			modifier: StorageEntryModifier::Default,
			ty: StorageType::Plain(moment.clone()),
			default: vec![112, 23, 0, 0, 0, 0, 0, 0],
			documentation: vec!["Some Kind of docs".to_string()],
		},
	);

	map.insert(
		"TestStorage1".to_string(),
		StorageMetadata {
			prefix: "TestStorage1".to_string(),
			modifier: StorageEntryModifier::Default,
			ty: StorageType::Plain(u64_t),
			default: vec![0, 0, 0, 0, 0, 0, 0, 0],
			documentation: vec!["Some Kind of docs 2".to_string()],
		},
	);

	map.insert(
		"TestStorage2".to_string(),
		StorageMetadata {
			prefix: "TestStorage2".to_string(),
			modifier: StorageEntryModifier::Optional,
			ty: StorageType::Plain(moment),
			default: vec![0, 0, 0, 0, 0, 0, 0, 0],
			documentation: vec!["Some Kind of docs 2".to_string()],
		},
	);

	map.insert(
		"TestStorage3".to_string(),
		StorageMetadata {
			prefix: "TestStorage3".to_string(),
			modifier: StorageEntryModifier::Optional,
			ty: StorageType::Plain(precision),
			default: vec![0, 0, 0, 0, 0, 0, 0, 0],
			documentation: vec!["Some Kind of docs 3".to_string()],
		},
	);
	map
}

fn call_mock() -> HashMap<String, CallMetadata> {
	let mut map = HashMap::new();

	map.insert(
		"TestCall0".to_string(),
		CallMetadata {
			name: "foo_function0".to_string(),
			index: 3,
			arguments: vec![CallArgMetadata { name: "foo_arg".to_string(), ty: RustTypeMarker::I8 }],
		},
	);
	map.insert(
		"TestCall1".to_string(),
		CallMetadata {
			name: "foo_function1".to_string(),
			index: 2,
			arguments: vec![CallArgMetadata { name: "foo_arg".to_string(), ty: RustTypeMarker::U64 }],
		},
	);
	map.insert(
		"TestCall2".to_string(),
		CallMetadata {
			name: "foo_function2".to_string(),
			index: 1,
			arguments: vec![CallArgMetadata {
				name: "foo_arg".to_string(),
				ty: RustTypeMarker::TypePointer("SomeType".to_string()),
			}],
		},
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
	let module_event_metadata = ModuleEventMetadata { name: "TestEvent0".to_string(), arguments };

	map.insert(0, module_event_metadata);
	map
}
