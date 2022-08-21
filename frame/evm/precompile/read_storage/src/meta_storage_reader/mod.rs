use core::{convert::TryInto, marker::PhantomData};

use evm::{ExitError, ExitSucceed};
use fp_evm::LinearCostPrecompile;
use frame_support::log::trace;
use sp_std::prelude::*;
use sp_std::{borrow::Cow, vec::Vec};

use crate::meta_storage_reader::input::Input;
use codec::Decode;
use frame_metadata::StorageEntryModifier;

use crate::raw_storage_reader::RawStorageReader;
pub use pallet_storage_metadata_provider::*;
use utils::*;

pub mod input;
pub mod key;
#[cfg(test)]
mod mock;
mod pallet_storage_metadata_provider;
mod utils;

#[derive(Default, Debug)]
pub struct MetaStorageReader<M> {
    _marker: PhantomData<M>,
}

impl<M: PalletStorageMetadataProvider> LinearCostPrecompile for MetaStorageReader<M> {
    const BASE: u64 = 3000;
    const WORD: u64 = 0;

    fn execute(mut input: &[u8], cost: u64) -> Result<(ExitSucceed, Vec<u8>), ExitError> {
        trace!("`MetaStorageReader` input: {:?}, cost: {}", input, cost);

        let Input {
            pallet,
            entry,
            key,
            params,
        } = Input::decode(&mut input).map_err(Error::Decoding)?;

        trace!(
            "`MetaStorageReader` decoded input: pallet = {:?}, entry = {:?}, key = {:?}, params = {:?}",
            pallet,
            entry,
            key,
            params
        );

        let entry_meta =
            M::pallet_storage_entry_metadata(&pallet, &entry)?.ok_or(Error::MemberNotFound)?;
        let default_byte_getter = (entry_meta.modifier == StorageEntryModifier::Default)
            .then(|| entry_meta.default.to_left().ok_or(Error::InvalidMetadata))
            .transpose()?;

        let storage_key = key
            .to_pallet_entry_storage_key(&pallet, &entry, &entry_meta.ty)
            .ok_or(Error::InvalidKey)?;
        let params = params.try_into()?;

        RawStorageReader::read(&storage_key, params, cost)
            .or_else(|error| {
                if error == super::raw_storage_reader::Error::ItemNotFound.into() {
                    let value = default_byte_getter
                        .map(|byte_getter| byte_getter.0.default_byte())
                        .map(Ok)
                        .unwrap_or(Err(error))?;

                    let (lower, upper) = params.lower_upper()?;
                    let sliced_value = value
                        .get(lower..upper.unwrap_or(value.len()).min(value.len()))
                        .map(<[_]>::to_vec)
                        .unwrap_or_default();

                    Ok(sliced_value)
                } else {
                    Err(error)
                }
            })
            .map(|res| (ExitSucceed::Returned, res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MemberNotFound,
    InvalidMetadata,
    InvalidKey,
    ItemNotFound,
    Decoding(codec::Error),
}

impl From<Error> for ExitError {
    fn from(err: Error) -> Self {
        let msg = match err {
            Error::InvalidMetadata => "Invalid metadata",
            Error::MemberNotFound => "Member not found",
            Error::InvalidKey => "Invalid key",
            Error::ItemNotFound => "Item not found",
            Error::Decoding(_) => "Failed to decode input",
        };

        ExitError::Other(Cow::Borrowed(msg))
    }
}

#[cfg(test)]
mod tests {
    use codec::{Decode, Encode};
    use core::str::FromStr;
    use frame_support::{StorageDoubleMap, StorageMap, StorageValue};
    use std::collections::BTreeMap;

    use pallet_evm::GenesisAccount;
    use sp_core::{H160, U256};
    use sp_std::prelude::*;

    use crate::raw_storage_reader::input::InputParams;

    use super::key::*;
    use super::mock::{test_storage::*, *};
    use super::*;

    pub fn ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        let mut accounts = BTreeMap::new();
        accounts.insert(
            H160::from_str("1000000000000000000000000000000000000001").unwrap(),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1000000),
                storage: Default::default(),
                code: vec![
                    0x00, // STOP
                ],
            },
        );
        accounts.insert(
            H160::from_str("1000000000000000000000000000000000000002").unwrap(),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1000000),
                storage: Default::default(),
                code: vec![
                    0xff, // INVALID
                ],
            },
        );

        t.into()
    }

    #[derive(Eq, PartialEq, Debug)]
    enum ErrorWrapper {
        Execution(ExitError),
        Decoding(codec::Error),
    }

    impl From<ExitError> for ErrorWrapper {
        fn from(err: ExitError) -> Self {
            Self::Execution(err)
        }
    }

    impl From<codec::Error> for ErrorWrapper {
        fn from(err: codec::Error) -> Self {
            Self::Decoding(err)
        }
    }

    macro_rules! assert_returned_value {
        ($expr: expr, $val: expr) => {{
            assert_eq!(
                $expr
                    .map_err(ErrorWrapper::Execution)
                    .and_then(|(exit_code, bytes)| {
                        assert_eq!(exit_code, ExitSucceed::Returned);

                        Decode::decode(&mut &bytes[..]).map_err(ErrorWrapper::Decoding)
                    }),
                $val
            );
        }};
    }

    #[test]
    fn invalid_input() {
        ext().execute_with(|| {
            let input = Input::new("Pallet", "Version", NoKey, InputParams::None);
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Err::<Bytes, _>(ExitError::from(super::Error::MemberNotFound).into())
            );

            let input = Input::new("TestStorage", "Abcde", NoKey, InputParams::None);
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Err::<Bytes, _>(ExitError::from(super::Error::MemberNotFound).into())
            );

            let input = Input::new("TestStorage", "MapDefault", NoKey, InputParams::None);
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Err::<Bytes, _>(ExitError::from(super::Error::InvalidKey).into())
            );

            let input = Input::new(
                "TestStorage",
                "DoubleMap",
                MapKey::new(Bytes::default()),
                InputParams::None,
            );
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Err::<Bytes, _>(ExitError::from(super::Error::InvalidKey).into())
            );
        })
    }

    #[test]
    fn entity_access() {
        ext().execute_with(|| {
            let input = Input::new("TestStorage", "SingleDefault", NoKey, InputParams::None);

            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Ok(Bytes::default())
            );
        })
    }

    #[test]
    fn large_entity_access() {
        ext().execute_with(|| {
            let input = Input::new(
                "TestStorage",
                "LargeSingleDefault",
                NoKey,
                InputParams::None,
            );

            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Ok(LargeBytes::default())
            );

            LargeSingleDefault::put(LargeBytes::default());

            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Ok(LargeBytes::default())
            );
        })
    }

    macro_rules! with_all_maps {
        (for $map: ident with $name: ident as name: $expr: expr) => {{
            use MapWithBlake2_128 as $map;
            let $name = "MapWithBlake2_128";
            $expr
        }
        {
            use MapWithBlake2_256 as $map;
            let $name = "MapWithBlake2_256";
            $expr
        }
        {
            use MapWithBlake2_128Concat as $map;
            let $name = "MapWithBlake2_128Concat";
            $expr
        }
        {
            use MapWithTwox128 as $map;
            let $name = "MapWithTwox128";
            $expr
        }
        {
            use MapWithTwox256 as $map;
            let $name = "MapWithTwox256";
            $expr
        }
        {
            use MapWithTwox64Concat as $map;
            let $name = "MapWithTwox64Concat";
            $expr
        }
        {
            use MapWithIdentity as $map;
            let $name = "MapWithIdentity";
            $expr
        }};
    }

    #[test]
    fn map_access() {
        ext().execute_with(|| {
            with_all_maps!(
                for map with name as name: {
                    map::insert(Bytes::with_len(10), Bytes::from_to(100, 1000));
                    map::insert(Bytes(name.as_bytes().to_vec()), Bytes(name.as_bytes().to_vec()));

                    let input = Input::new(
                        "TestStorage",
                        name,
                        MapKey::new(Bytes::with_len(10)),
                        InputParams::None,
                    );
                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                        Ok(Bytes::from_to(100, 1000))
                    );

                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(&
                            input
                                .with_replaced_key(MapKey::new(Bytes(name.as_bytes().to_vec())))
                                .encode(), 0),
                        Ok(Bytes(name.as_bytes().to_vec()))
                    );

                    let non_existent_key = Bytes::with_len(11);
                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(
                            &input
                                .with_replaced_key(MapKey::new(non_existent_key))
                                .encode(),
                            0
                        ),
                        Err::<Bytes, _>(ExitError::from(super::Error::ItemNotFound).into())
                    );
                }
            );
        })
    }

    #[test]
    fn map_access_with_params() {
        ext().execute_with(|| {
            with_all_maps!(
                for map with name as name: {
                    map::insert(Bytes::with_len(10), Bytes::from_to(100, 1000));
                    map::insert(Bytes(name.as_bytes().to_vec()), Bytes(name.as_bytes().to_vec()));
                    #[derive(Encode, Decode, PartialEq, Eq, Debug)]
                    struct CompactU32(
                        #[codec(compact)]
                        u32
                    );

                    let input = Input::new(
                        "TestStorage",
                        name,
                        MapKey::new(Bytes::with_len(10)),
                        InputParams::Len(4),
                    );
                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                        Ok(CompactU32(900))
                    );

                    let input = Input::new(
                        "TestStorage",
                        name,
                        MapKey::new(Bytes::with_len(10)),
                        InputParams::Offset(10),
                    );
                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                        Ok(RawBytes(Bytes::from_to(108, 1000)))
                    );

                    let input = Input::new(
                        "TestStorage",
                        name,
                        MapKey::new(Bytes::with_len(10)),
                        InputParams::OffsetAndLen { offset: 10, len: 50 },
                    );
                    assert_returned_value!(
                        MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                        Ok(RawBytes(Bytes::from_to(108, 158)))
                    );
                }
            );
        })
    }

    #[test]
    fn double_map_access() {
        ext().execute_with(|| {
            DoubleMap::insert(
                Bytes::with_len(10),
                Bytes::with_len(20),
                Bytes::with_len(30),
            );

            let input = Input::new(
                "TestStorage",
                "DoubleMap",
                DoubleMapKey::new(Bytes::with_len(10), Bytes::with_len(20)),
                InputParams::None,
            );
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(&input.encode(), 0),
                Ok(Bytes::with_len(30))
            );

            let non_existent_key = Bytes::with_len(11);
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(
                    &input
                        .with_replaced_key(DoubleMapKey::new(non_existent_key, Bytes::with_len(20)))
                        .encode(),
                    0
                ),
                Err::<Bytes, _>(ExitError::from(super::Error::ItemNotFound).into())
            );

            let non_existent_second_key = Bytes::with_len(21);
            assert_returned_value!(
                MetaStorageReader::<Runtime>::execute(
                    &input
                        .with_replaced_key(DoubleMapKey::new(
                            Bytes::with_len(10),
                            non_existent_second_key
                        ))
                        .encode(),
                    0
                ),
                Err::<Bytes, _>(ExitError::from(super::Error::ItemNotFound).into())
            );
        })
    }
}
