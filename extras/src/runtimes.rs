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

pub fn kusama_upgrade_block(version: &u64) -> Option<u64> {
	KUSAMA_RUNTIMES.get(version).copied()
}

static KUSAMA_RUNTIMES: phf::OrderedMap<u64, u64> = phf_ordered_map! {
	0u64       => 1020,
	26669u64   => 1021,
	38245u64   => 1022,
	54248u64   => 1023,
	59659u64   => 1024,
	67651u64   => 1025,
	82191u64   => 1027,
	83238u64   => 1028,
	101503u64  => 1029,
	203466u64  => 1030,
	295787u64  => 1031,
	461692u64  => 1032,
	504329u64  => 1033,
	569327u64  => 1038,
	587687u64  => 1039,
	653183u64  => 1040,
	693488u64  => 1042,
	901442u64  => 1045,
	1375086u64 => 1050,
	1445458u64 => 1051,
	1472960u64 => 1052,
	1475648u64 => 1053,
	1491596u64 => 1054,
	1574408u64 => 1055,
	2064961u64 => 1058,
	2201991u64 => 1062,
	2671528u64 => 2005,
	2704202u64 => 2007,
	2728002u64 => 2008,
	2832534u64 => 2011,
	2962294u64 => 2012,
	3240000u64 => 2013,
	3274408u64 => 2015,
	3323565u64 => 2019,
	3534175u64 => 2022,
	3860281u64 => 2023,
	4143129u64 => 2024,
	4401242u64 => 2025,
	4841367u64 => 2026,
	5961600u64 => 2027,
	6137912u64 => 2028,
	6561855u64 => 2029,
	7100891u64 => 2030,
	7468792u64 => 9010,
	7668600u64 => 9030,
	7812476u64 => 9040,
	8010981u64 => 9050,
	8073833u64 => 9070,
	8555825u64 => 9080,
	8945245u64 => 9090
};

pub fn polkadot_upgrade_block(version: &u64) -> Option<u64> {
	POLKADOT_RUNTIMES.get(version).copied()
}

static POLKADOT_RUNTIMES: phf::OrderedMap<u64, u64> = phf_ordered_map! {
	0u64       => 0,
	29231u64   => 1,
	188836u64  => 5,
	199405u64  => 6,
	214264u64  => 7,
	244358u64  => 8,
	303079u64  => 9,
	314201u64  => 10,
	342400u64  => 11,
	443963u64  => 12,
	528470u64  => 13,
	687751u64  => 14,
	746085u64  => 15,
	787923u64  => 16,
	799302u64  => 17,
	1205128u64 => 18,
	1603423u64 => 23,
	1733218u64 => 24,
	2005673u64 => 25,
	2436698u64 => 26,
	3613564u64 => 27,
	3899547u64 => 28,
	4345767u64 => 29,
	4876134u64 => 30,
	5661442u64 => 9050,
	6321619u64 => 9080,
	6713249u64 => 9090
};

pub fn westend_upgrade_block(version: &u64) -> Option<u64> {
	WESTEND_RUNTIMES.get(version).copied()
}

static WESTEND_RUNTIMES: phf::OrderedMap<u64, u64> = phf_ordered_map! {
	214356u64  => 4,
	392764u64  => 7,
	409740u64  => 8,
	809976u64  => 20,
	877581u64  => 24,
	879238u64  => 25,
	889472u64  => 26,
	902937u64  => 27,
	932751u64  => 28,
	991142u64  => 29,
	1030162u64 => 31,
	1119657u64 => 32,
	1199282u64 => 33,
	1342534u64 => 34,
	1392263u64 => 35,
	1431703u64 => 36,
	1433369u64 => 37,
	1490972u64 => 41,
	2087397u64 => 43,
	2316688u64 => 44,
	2549864u64 => 45,
	3925782u64 => 46,
	3925843u64 => 47,
	4207800u64 => 48,
	4627944u64 => 49,
	5124076u64 => 50,
	5478664u64 => 900,
	5482450u64 => 9000,
	5584305u64 => 9010,
	5784566u64 => 9030,
	5879822u64 => 9031,
	5896856u64 => 9032,
	5897316u64 => 9033,
	6117927u64 => 9050,
	6210274u64 => 9070,
	6379314u64 => 9080,
	6979141u64 => 9090,
};
