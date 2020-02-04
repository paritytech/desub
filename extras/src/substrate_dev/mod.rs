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

use core::{decoder::Decoder, metadata::Metadata as RawMetadata};
use node_runtime::{Runtime as RuntimeLatest, VERSION as VERSION_LATEST};
use node_primitives::{AccountId, AccountIndex, Balance, BlockNumber, Hash, Index, Moment, Signature};
use codec::Encode;
use type_metadata::{Metadata, tuple_meta_type};

pub fn register() -> Decoder {
    env_logger::init();
    let mut decoder = Decoder::new();
    let meta = RawMetadata::new(&RuntimeLatest::metadata().encode());
    decoder.register_version(meta, VERSION_LATEST);
    system(&mut decoder);
    println!("\n{:?}", decoder);
    decoder
}

fn system(decoder: &mut Decoder) {

    decoder.register::<TempOrigin,_>(
        &VERSION_LATEST,
        "System",
        "T::Origin"
    );

    /*decoder.register::<node_runtime::Call, _>(
        &VERSION_LATEST,
        "System",
        "T::Call"
);

    decoder.register::<TempIndex, _>(
        &VERSION_LATEST,
        "System",
        "T::Index"
    );*/
    decoder.register::<node_primitives::BlockNumber, _>(
        &VERSION_LATEST,
        "System",
        "T::BlockNumber"
    );

    /*
    #[derive(Metadata)]
    struct TempHash(node_primitives::Hash);
    decoder.register::<TempHash, _>(
        &VERSION_LATEST,
        "System",
        "T::Hash"
    );
     */
    /* decoder.register::<node_runtime::Hashing, _>(
        &VERSION_LATEST,
        "System",
        "T::Hashing",
);*/
    /*
    decoder.register::<node_primitives::AccountId, _>(
        &VERSION_LATEST,
        "System",
        "T::AccountId"
    );
    */
    /*decoder.register::<node_runtime::Lookup, _>(
        &VERSION_LATEST,
        "System",
        "T::Lookup"
);
    decoder.register::<node_primitives::Header, _>(
        &VERSION_LATEST,
        "System",
        "T::Header"
    );
    */
    /*decoder.register::<node_runtime::Event, _>(
        &VERSION_LATEST,
        "System",
        "T::Event"
);*/
    /*
    decoder.register::<node_runtime::BlockHashCount, _>(
        &VERSION_LATEST,
        "System",
        "T::BlockHashCount"
);

    decoder.register::<node_runtime::MaximumBlockWeight, _>(
        &VERSION_LATEST,
        "System",
        "T::MaximumBlockWeight"
    );
    decoder.register::<node_runtime::MaximumBlockLength, _>(
        &VERSION_LATEST,
        "System",
        "T::MaxiumumBlockLength"
    );
    decoder.register::<node_runtime::AvailableBlockRatio, _>(
        &VERSION_LATEST,
        "System",
        "T::AvailableBlockRatio"
    );
    decoder.register::<node_runtime::Version, _>(
        &VERSION_LATEST,
        "System",
        "T::Version"
    );
    decoder.register::<node_runtime::ModuleToIndex, _>(
        &VERSION_LATEST,
        "System",
        "T::ModuleToIndex"
    );
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_metadata() {
        let decoder = register();
        log::info!("{:#?}", decoder);
    }
}
