// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
//test the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

use sp_version::RuntimeVersion;
use std::borrow::Cow;

pub fn mock_runtime(num: u32) -> RuntimeVersion {
	RuntimeVersion {
		spec_name: "test-runtime".into(),
		impl_name: "test-runtime-impl".into(),
		authoring_version: num,
		spec_version: num,
		impl_version: num,
		apis: Cow::from(Vec::new()),
		transaction_version: 4,
	}
}
