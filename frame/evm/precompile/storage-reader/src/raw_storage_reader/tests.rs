use codec::{Decode, Encode};
use fp_evm::Precompile;
use frame_support::StorageMap;

use sp_core::{H160, U256};
use sp_std::prelude::*;

use crate::common::params::Params;

use super::*;
use crate::mock::{test_storage::*, *};

static DUMMY_CTX: &'static evm::Context = &evm::Context {
    address: H160([0; 20]),
    caller: H160([0; 20]),
    apparent_value: U256([32; 4]),
};

pub fn ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    t.into()
}

#[test]
fn raw_access() {
    ext().execute_with(|| {
        MapWithBlake2_128::insert(Bytes::with_len(10), Bytes::with_len(100));
        let raw_key = vec![
            67, 72, 185, 244, 78, 99, 49, 57, 216, 168, 24, 127, 78, 234, 212, 96, 213, 134, 173,
            83, 23, 191, 182, 238, 194, 232, 224, 37, 134, 115, 248, 8, 199, 181, 78, 40, 234, 211,
            141, 15, 124, 148, 20, 131, 114, 161, 6, 42,
        ];

        let input = RawStorageReaderInput {
            key: raw_key,
            params: Params::None,
        };
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output[0], 1);
        assert_eq!(
            Bytes::decode(&mut &out.output[1..]).unwrap(),
            Bytes::with_len(100)
        );

        let non_existent_key = vec![1, 2, 3];

        let input = RawStorageReaderInput {
            key: non_existent_key,
            params: Params::None,
        };
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output, vec![0]);
    })
}
