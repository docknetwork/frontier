use codec::{Decode, Encode};
use fp_evm::Precompile;
use frame_support::{assert_noop, StorageMap};

use sp_core::{H160, U256};

use crate::common::params::Params;

use super::*;
use crate::mock::{test_storage::*, *};

static DUMMY_CTX: &'static evm::Context = &evm::Context {
    address: H160([0; 20]),
    caller: H160([0; 20]),
    apparent_value: U256([0; 4]),
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

        let input = RawStorageReaderInput::new(&raw_key[..], Params::None);
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

        let input = RawStorageReaderInput::new(non_existent_key, Params::None);
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output, vec![0]);
    })
}

#[test]
fn raw_access_with_params() {
    ext().execute_with(|| {
        MapWithBlake2_128::insert(Bytes::with_len(10), Bytes::with_len(100));
        let raw_key = vec![
            67, 72, 185, 244, 78, 99, 49, 57, 216, 168, 24, 127, 78, 234, 212, 96, 213, 134, 173,
            83, 23, 191, 182, 238, 194, 232, 224, 37, 134, 115, 248, 8, 199, 181, 78, 40, 234, 211,
            141, 15, 124, 148, 20, 131, 114, 161, 6, 42,
        ];

        let input = RawStorageReaderInput::new(&raw_key[..], Params::Offset(10));
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output[0], 1);
        assert_eq!(
            RawBytes::decode(&mut &out.output[1..]).unwrap(),
            RawBytes(Bytes::from_to(8, 100))
        );

        let input = RawStorageReaderInput::new(&raw_key[..], Params::Len(10));
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output[0], 1);
        assert_eq!(
            RawBytes::decode(&mut &out.output[1..]).unwrap(),
            RawBytes(Bytes([&[145, 1][..], &Bytes::from_to(0, 8).0[..]].concat()))
        );

        let input = RawStorageReaderInput::new(
            &raw_key[..],
            Params::OffsetAndLen {
                offset: 10,
                len: 10,
            },
        );
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(out.output[0], 1);
        assert_eq!(
            RawBytes::decode(&mut &out.output[1..]).unwrap(),
            RawBytes(Bytes::from_to(8, 18))
        );
    })
}

#[test]
fn cost() {
    ext().execute_with(|| {
        MapWithBlake2_128::insert(Bytes::with_len(10), Bytes::with_len(100));
        let raw_key = vec![
            67, 72, 185, 244, 78, 99, 49, 57, 216, 168, 24, 127, 78, 234, 212, 96, 213, 134, 173,
            83, 23, 191, 182, 238, 194, 232, 224, 37, 134, 115, 248, 8, 199, 181, 78, 40, 234, 211,
            141, 15, 124, 148, 20, 131, 114, 161, 6, 42,
        ];

        let input = RawStorageReaderInput::new(&raw_key[..], Params::None);
        let out = RawStorageReader::<Runtime>::execute(
            &input.encode(),
            Some(RawStorageReader::<Runtime>::base_gas_cost() + 5_000_000),
            DUMMY_CTX,
        )
        .unwrap();

        assert_eq!(
            out.cost,
            <Runtime as frame_system::Config>::DbWeight::get().reads(1)
                + <Runtime as pallet_evm::Config>::ByteReadWeight::get()
                    * (out.output.len() as u64 - 1)
        );

        let invalid_cost = <Runtime as frame_system::Config>::DbWeight::get().reads(1);
        assert_noop!(
            RawStorageReader::<Runtime>::execute(&input.encode(), Some(invalid_cost), DUMMY_CTX,),
            evm::ExitError::OutOfGas
        );
    });
}
