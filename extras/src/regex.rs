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

use regex::Regex;

fn rust_array_decl() -> Regex {
    // width of number and unsigned/signed are all in their own capture group
    // size of array is in the last capture group
    Regex::new(r"^\[([uif]{1})(8)?(16)?(32)?(64)?(128)?;\s?([\d]*)]$")
        .expect("Regex expression invalid")
}

fn rust_vec_decl() -> Regex {
    Regex::new(r"Vec<([\w]+)>")
        .expect("Regex expression invalid")
}

fn rust_tuple_decl() -> Regex {
    Regex::new(r"(:?([\w><]+),? *)*")
        .expect("Regex expression invalid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Captures;

    fn caps_to_vec_str<'a>(caps: Captures<'a>) -> Vec<Option<&'a str>> {
        caps.iter()
            .map(|c| c.map(|c| c.as_str()))
            .collect::<Vec<Option<&str>>>()
    }

    #[test]
    fn should_match_array_decl() {
        let re = rust_array_decl();
        assert!(re.is_match("[u8; 16]"));
        assert!(re.is_match("[u16; 16]"));
        assert!(re.is_match("[u32; 16]"));
        assert!(re.is_match("[u64; 16]"));
        assert!(re.is_match("[u128; 16]"));
        assert!(re.is_match("[i8; 16]"));
        assert!(re.is_match("[i16; 16]"));
        assert!(re.is_match("[i32; 16]"));
        assert!(re.is_match("[i64; 16]"));
        assert!(re.is_match("[i128; 16]"));
        assert!(re.is_match("[u16; 128]"));
        assert!(re.is_match("[u32; 64]"));
        assert!(re.is_match("[u64; 99999]"));
        assert!(re.is_match("[u128; 23]"));
    }

    #[test]
    fn should_seperate_args_in_capture_groups() {
        let re = rust_array_decl();
       
        let caps = caps_to_vec_str(re.captures("[u8; 16]").unwrap());
        assert_eq!(vec![Some("[u8; 16]"), Some("u"), Some("8"), None, None, None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i8; 16]").unwrap());
        assert_eq!(vec![Some("[i8; 16]"), Some("i"), Some("8"), None, None, None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[u16; 16]").unwrap());
        assert_eq!(vec![Some("[u16; 16]"), Some("u"), None, Some("16"), None, None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i16; 16]").unwrap());
        assert_eq!(vec![Some("[i16; 16]"), Some("i"), None, Some("16"), None, None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[u32; 16]").unwrap());
        assert_eq!(vec![Some("[u32; 16]"), Some("u"), None, None, Some("32"), None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i32; 16]").unwrap());
        assert_eq!(vec![Some("[i32; 16]"), Some("i"), None, None, Some("32"), None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[u64; 16]").unwrap());
        assert_eq!(vec![Some("[u64; 16]"), Some("u"), None, None, None, Some("64"), None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i64; 16]").unwrap());
        assert_eq!(vec![Some("[i64; 16]"), Some("i"), None, None, None, Some("64"), None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[u128; 16]").unwrap());
        assert_eq!(vec![Some("[u128; 16]"), Some("u"), None, None, None, None, Some("128"), Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i128; 16]").unwrap());
        assert_eq!(vec![Some("[i128; 16]"), Some("i"), None, None, None, None, Some("128"), Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[f32; 16]").unwrap());
        assert_eq!(vec![Some("[f32; 16]"), Some("f"), None, None, Some("32"), None, None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[f64; 16]").unwrap());
        assert_eq!(vec![Some("[f64; 16]"), Some("f"), None, None, None, Some("64"), None, Some("16")], caps);

        let caps = caps_to_vec_str(re.captures("[i128; 9999]").unwrap());
        assert_eq!(vec![Some("[i128; 9999]"), Some("i"), None, None, None, None, Some("128"), Some("9999")], caps);

        let caps = caps_to_vec_str(re.captures("[u128; 9999]").unwrap());
        assert_eq!(vec![Some("[u128; 9999]"), Some("u"), None, None, None, None, Some("128"), Some("9999")], caps);
    }

    #[test]
    fn should_parse_vecs() {
        let re = rust_vec_decl();
        assert!(re.is_match("Vec<RuntimeVersionApi>"));
        assert!(re.is_match("Vec<BlockNumber>"));
        assert!(re.is_match("Vec<SomeStruct>"));
    }

    #[test]
    fn should_get_type_of_vec() {
        let re = rust_vec_decl();
        let caps = caps_to_vec_str(re.captures("Vec<RuntimeVersionApi>").unwrap());
        // first capture group is always entire expression
        assert!(caps[1] == Some("RuntimeVersionApi"));
    }

    #[test]
    fn should_parse_tuples() {
        let re = rust_tuple_decl();
        assert!(re.is_match("(StorageKey, Option<StorageData>)"));
        assert!(re.is_match("(ApiKey, u32)"));
        assert!(re.is_match("(u32,ApiKey,AnotherType)"));
    }

    #[test]
    fn should_get_types_in_tuple() {
        let re = rust_tuple_decl();
        let caps = caps_to_vec_str(re.captures("(StorageKey, Option<StorageData>)").unwrap());
        assert_eq!(vec![Some("(StorageKey, Option<StorageData>)"), Some("StorageKey"), Some("Option<StorageData>")], caps);
       
    }
}
