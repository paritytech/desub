// Copyright 2019 Parity Technologies (UK) Ltd.
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

use super::Modules;
use crate::error::Error;

/// deserializes raw json definitions into modules
pub fn definitions(raw_json: &str) -> Result<Modules, Error> {
    let types: Modules = serde_json::from_str(raw_json)?;
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    const RAW_JSON: &'static str = r#"
{
	"runtime": {
		"types": {
			"Extrinsic": "GenericExtrinsic",
			"hash": "H512",
			"BlockNumber": "u64",
			"ChangesTrieConfiguration": {
				"digestInterval": "u32",
				"digestLevels": "u32"
			},
			"DispatchInfo": {
				"weight": "Weight",
				"class": "DispatchClass",
				"paysFee": "bool"
			},
			"MultiSignature": {
				"_enum": {
					"Ed25519": "Ed25519Signature",
					"Sr25519": "Sr25519Signature",
					"Ecdsa": "EcdsaSignature"
				}
			},
			"Reasons": {
				"_enum": [
					"Fee",
					"Misc",
					"All"
				]
			},
			"WithdrawReasons": {
				"_set": {
					"TransactionPayment": 1,
					"Transfer": 2,
					"Reserve": 4,
					"Fee": 8,
					"Tip": 16
				}
			}
		}
	}
}
"#;

    #[test]
    fn should_deserialize() -> Result<(), Error> {
        let types = definitions(DEFS)?;
        dbg!(&types);
        Ok(())
    }

    #[test]
    fn should_deserialize_correctly() -> Result<(), Error> {
        let deser_dot_types = definitions(RAW_JSON)?;
        let mut modules = HashMap::new();
        let mut types = HashMap::new();
        types.insert(
            "Extrinsic".to_string(),
            RustTypeMarker::TypePointer("GenericExtrinsic".to_string()),
        );
        types.insert(
            "hash".to_string(),
            RustTypeMarker::TypePointer("H512".to_string()),
        );
        types.insert("BlockNumber".to_string(), RustTypeMarker::U64);
        types.insert(
            "ChangesTrieConfiguration".to_string(),
            RustTypeMarker::Struct(vec![
                StructField {
                    name: "digestInterval".to_string(),
                    ty: RustTypeMarker::U32,
                },
                StructField {
                    name: "digestLevels".to_string(),
                    ty: RustTypeMarker::U32,
                },
            ]),
        );
        types.insert(
            "DispatchInfo".to_string(),
            RustTypeMarker::Struct(vec![
                StructField {
                    name: "weight".to_string(),
                    ty: RustTypeMarker::TypePointer("Weight".to_string()),
                },
                StructField {
                    name: "class".to_string(),
                    ty: RustTypeMarker::TypePointer("DispatchClass".to_string()),
                },
                StructField {
                    name: "paysFee".to_string(),
                    ty: RustTypeMarker::Bool,
                },
            ]),
        );
        types.insert(
            "MultiSignature".to_string(),
            RustTypeMarker::Enum(vec![
                EnumField {
                    variant_name: Some("Ed25519".to_string()),
                    ty: StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                        "Ed25519Signature".to_string(),
                    )),
                },
                EnumField {
                    variant_name: Some("Sr25519".to_string()),
                    ty: StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                        "Sr25519Signature".to_string(),
                    )),
                },
                EnumField {
                    variant_name: Some("Ecdsa".to_string()),
                    ty: StructUnitOrTuple::Tuple(RustTypeMarker::TypePointer(
                        "EcdsaSignature".to_string(),
                    )),
                },
            ]),
        );
        types.insert(
            "Reasons".to_string(),
            RustTypeMarker::Enum(vec![
                EnumField {
                    variant_name: None,
                    ty: StructUnitOrTuple::Unit("Fee".to_string()),
                },
                EnumField {
                    variant_name: None,
                    ty: StructUnitOrTuple::Unit("Misc".to_string()),
                },
                EnumField {
                    variant_name: None,
                    ty: StructUnitOrTuple::Unit("All".to_string()),
                },
            ]),
        );
        types.insert(
            "WithdrawReasons".to_string(),
            RustTypeMarker::Set(vec![
                SetField {
                    name: "TransactionPayment".to_string(),
                    num: 1,
                },
                SetField {
                    name: "Transfer".to_string(),
                    num: 2,
                },
                SetField {
                    name: "Reserve".to_string(),
                    num: 4,
                },
                SetField {
                    name: "Fee".to_string(),
                    num: 8,
                },
                SetField {
                    name: "Tip".to_string(),
                    num: 16,
                },
            ]),
        );

        for (key, val) in types.iter() {
            assert_eq!(val, &deser_dot_types.modules["runtime"].types[key]);
        }

        let mod_types = ModuleTypes { types };
        modules.insert("runtime".to_string(), mod_types);
        let dot_types = Modules { modules };
        assert_eq!(dot_types, deser_dot_types);
        Ok(())
    }
}
