// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use phf::phf_ordered_map;

pub fn kusama_upgrade_block(version: &u32) -> Option<u64> {
	KUSAMA_RUNTIMES.get(version).copied()
}

static KUSAMA_RUNTIMES: phf::OrderedMap<u32, u64> = phf_ordered_map! {
	1020u32 => 0,
	1021u32 => 26669,
	1022u32 => 38245,
	1023u32 => 54248,
	1024u32 => 59659,
	1025u32 => 67651,
	1027u32 => 82191,
	1028u32 => 83238,
	1029u32 => 101503,
	1030u32 => 203466,
	1031u32 => 295787,
	1032u32 => 461692,
	1033u32 => 504329,
	1038u32 => 569327,
	1039u32 => 587687,
	1040u32 => 653183,
	1042u32 => 693488,
	1045u32 => 901442,
	1050u32 => 1375086,
	1051u32 => 1445458,
	1052u32 => 1472960,
	1053u32 => 1475648,
	1054u32 => 1491596,
	1055u32 => 1574408,
	1058u32 => 2064961,
	1062u32 => 2201991,
	2005u32 => 2671528,
	2007u32 => 2704202,
	2008u32 => 2728002,
	2011u32 => 2832534,
	2012u32 => 2962294,
	2013u32 => 3240000,
	2015u32 => 3274408,
	2019u32 => 3323565,
	2022u32 => 3534175,
	2023u32 => 3860281,
	2024u32 => 4143129,
	2025u32 => 4401242,
	2026u32 => 4841367,
	2027u32 => 5961600,
	2028u32 => 6137912,
	2029u32 => 6561855,
	2030u32 => 7100891,
	9010u32 => 7468792,
	9030u32 => 7668600,
	9040u32 => 7812476,
	9050u32 => 8010981,
	9070u32 => 8073833,
	9080u32 => 8555825,
	9090u32 => 8945245
};

pub fn polkadot_upgrade_block(version: &u32) -> Option<u64> {
	POLKADOT_RUNTIMES.get(version).copied()
}

static POLKADOT_RUNTIMES: phf::OrderedMap<u32, u64> = phf_ordered_map! {
	0u32    => 0,
	1u32    => 29231,
	5u32    => 188836,
	6u32    => 199405,
	7u32    => 214264,
	8u32    => 244358,
	9u32    => 303079,
	10u32   => 314201,
	11u32   => 342400,
	12u32   => 443963,
	13u32   => 528470,
	14u32   => 687751,
	15u32   => 746085,
	16u32   => 787923,
	17u32   => 799302,
	18u32   => 1205128,
	23u32   => 1603423,
	24u32   => 1733218,
	25u32   => 2005673,
	26u32   => 2436698,
	27u32   => 3613564,
	28u32   => 3899547,
	29u32   => 4345767,
	30u32   => 4876134,
	9050u32 => 5661442,
	9080u32 => 6321619,
	9090u32 => 6713249
};

pub fn westend_upgrade_block(version: &u32) -> Option<u64> {
	WESTEND_RUNTIMES.get(version).copied()
}

static WESTEND_RUNTIMES: phf::OrderedMap<u32, u64> = phf_ordered_map! {
	4u32    => 214356,
	7u32    => 392764,
	8u32    => 409740,
	20u32   => 809976,
	24u32   => 877581,
	25u32   => 879238,
	26u32   => 889472,
	27u32   => 902937,
	28u32   => 932751,
	29u32   => 991142,
	31u32   => 1030162,
	32u32   => 1119657,
	33u32   => 1199282,
	34u32   => 1342534,
	35u32   => 1392263,
	36u32   => 1431703,
	37u32   => 1433369,
	41u32   => 1490972,
	43u32   => 2087397,
	44u32   => 2316688,
	45u32   => 2549864,
	46u32   => 3925782,
	47u32   => 3925843,
	48u32   => 4207800,
	49u32   => 4627944,
	50u32   => 5124076,
	900u32  => 5478664,
	9000u32 => 5482450,
	9010u32 => 5584305,
	9030u32 => 5784566,
	9031u32 => 5879822,
	9032u32 => 5896856,
	9033u32 => 5897316,
	9050u32 => 6117927,
	9070u32 => 6210274,
	9080u32 => 6379314,
	9090u32 => 6979141,
};
