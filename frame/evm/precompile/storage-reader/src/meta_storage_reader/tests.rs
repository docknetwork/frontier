use codec::{Decode, Encode};
use fp_evm::Precompile;
use frame_support::{StorageDoubleMap, StorageMap, StorageValue};

use crate::common::output::RawStorageValue;
use pallet_evm_test_vector_support::MockHandle;
use sp_core::{H160, U256};
use sp_std::prelude::*;

use crate::common::params::Params;

use super::{key::*, *};
use crate::mock::{test_storage::*, *};

pub fn ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	t.into()
}

#[derive(Eq, PartialEq, Debug)]
enum ErrorWrapper {
	Execution(PrecompileFailure),
	Decoding(codec::Error),
}

impl From<ExitError> for ErrorWrapper {
	fn from(err: ExitError) -> Self {
		PrecompileFailure::Error { exit_status: err }.into()
	}
}

impl From<PrecompileFailure> for ErrorWrapper {
	fn from(err: PrecompileFailure) -> Self {
		Self::Execution(err)
	}
}

impl From<codec::Error> for ErrorWrapper {
	fn from(err: codec::Error) -> Self {
		Self::Decoding(err)
	}
}

macro_rules! assert_decoded_eq {
	($expr: expr, $val: expr) => {{
		assert_eq!(
			$expr.map_err(ErrorWrapper::Execution).and_then(
				|PrecompileOutput {
				     output,
				     exit_status,
				     ..
				 }| {
					assert_eq!(exit_status, ExitSucceed::Returned);

					let raw = RawStorageValue::decode_from_bytes(&output[..]);
					raw.into_item()
						.map(|bytes| {
							Decode::decode(&mut &bytes[..]).map_err(ErrorWrapper::Decoding)
						})
						.transpose()
				}
			),
			$val
		);
	}};
}

static DUMMY_CTX: &'static evm::Context = &evm::Context {
	address: H160([0; 20]),
	caller: H160([0; 20]),
	apparent_value: U256([0; 4]),
};

#[test]
fn invalid_input() {
	ext().execute_with(|| {
		let input = MetaStorageReaderInput::new("Pallet", "Version", NoKey, Params::None);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Err::<Option<Bytes>, _>(
				PrecompileFailure::from(super::Error::PalletStorageEntryNotFound).into()
			)
		);

		let input = MetaStorageReaderInput::new("TestStorage", "Abcde", NoKey, Params::None);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Err::<Option<Bytes>, _>(
				PrecompileFailure::from(super::Error::PalletStorageEntryNotFound).into()
			)
		);

		let input = MetaStorageReaderInput::new("TestStorage", "MapDefault", NoKey, Params::None);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Err::<Option<Bytes>, _>(PrecompileFailure::from(super::Error::InvalidKey).into())
		);

		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMap",
			MapKey::new_single(Bytes::default()),
			Params::None,
		);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Err::<Option<Bytes>, _>(PrecompileFailure::from(super::Error::InvalidKey).into())
		);
	})
}

#[test]
fn entity_access() {
	ext().execute_with(|| {
		let input =
			MetaStorageReaderInput::new("TestStorage", "SingleDefault", NoKey, Params::None);

		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(Bytes::default()))
		);

		let input = MetaStorageReaderInput::new("TestStorage", "Single", NoKey, Params::None);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(None::<Bytes>)
		);

		Single::put(Bytes::with_len(123));
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(Bytes::with_len(123)))
		);
	})
}

#[test]
fn large_entity_access() {
	ext().execute_with(|| {
		let input =
			MetaStorageReaderInput::new("TestStorage", "LargeSingleDefault", NoKey, Params::None);

		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Err::<Option<LargeBytes>, _>(ExitError::OutOfGas.into())
		);

		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(100_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(LargeBytes::default()))
		);

		LargeSingleDefault::put(LargeBytes(Bytes::with_len(700)));

		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(100_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(LargeBytes(Bytes::with_len(700))))
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

                let input = MetaStorageReaderInput::new(
                    "TestStorage",
                    name,
                    MapKey::new_single(Bytes::with_len(10)),
                    Params::None,
                );
                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone())),
                    Ok(Some(Bytes::from_to(100, 1000)))
                );

                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(
                        &mut MockHandle::new(
                        input
                            .with_replaced_key(MapKey::new_single(Bytes(name.as_bytes().to_vec())))
                            .encode(), Some(30_000_000), DUMMY_CTX.clone())
                    ),
                    Ok(Some(Bytes(name.as_bytes().to_vec())))
                );

                let non_existent_key = Bytes::with_len(11);
                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(
                        &mut MockHandle::new(input
                            .with_replaced_key(MapKey::new_single(non_existent_key))
                            .encode(), Some(30_000_000), DUMMY_CTX.clone())
                    ),
                    Ok(None::<Bytes>)
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

                let input = MetaStorageReaderInput::new(
                    "TestStorage",
                    name,
                    MapKey::new_single(Bytes::with_len(10)),
                    Params::Len(4),
                );
                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone())),
                    Ok(Some(CompactU32(900)))
                );

                let input = MetaStorageReaderInput::new(
                    "TestStorage",
                    name,
                    MapKey::new_single(Bytes::with_len(10)),
                    Params::Offset(10),
                );
                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone())),
                    Ok(Some(RawBytes(Bytes::from_to(108, 1000))))
                );

                let input = MetaStorageReaderInput::new(
                    "TestStorage",
                    name,
                    MapKey::new_single(Bytes::with_len(10)),
                    Params::OffsetAndLen { offset: 10, len: 50 },
                );
                assert_decoded_eq!(
                    MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone())),
                    Ok(Some(RawBytes(Bytes::from_to(108, 158))))
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

		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMap",
			MapKey::new_double(Bytes::with_len(10), Bytes::with_len(20)),
			Params::None,
		);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(Bytes::with_len(30)))
		);

		let non_existent_key = Bytes::with_len(11);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input
					.with_replaced_key(MapKey::new_double(non_existent_key, Bytes::with_len(20)))
					.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(None::<Bytes>)
		);

		let non_existent_second_key = Bytes::with_len(21);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input
					.with_replaced_key(MapKey::new_double(
						Bytes::with_len(10),
						non_existent_second_key
					))
					.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(None::<Bytes>)
		);

		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMapDefault",
			MapKey::new_double(Bytes::with_len(10), Bytes::with_len(20)),
			Params::None,
		);

		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(Bytes::default()))
		);
	})
}

#[test]
fn double_map_access_with_params() {
	ext().execute_with(|| {
		DoubleMap::insert(
			Bytes::with_len(10),
			Bytes::with_len(11),
			Bytes::from_to(100, 1000),
		);
		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMap",
			MapKey::new_double(Bytes::with_len(10), Bytes::with_len(11)),
			Params::Offset(10),
		);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(RawBytes(Bytes::from_to(108, 1000))))
		);

		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMap",
			MapKey::new_double(Bytes::with_len(10), Bytes::with_len(11)),
			Params::OffsetAndLen {
				offset: 10,
				len: 10,
			},
		);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(RawBytes(Bytes::from_to(108, 118))))
		);

		let input = MetaStorageReaderInput::new(
			"TestStorage",
			"DoubleMap",
			MapKey::new_double(Bytes::with_len(10), Bytes::with_len(11)),
			Params::Len(2),
		);

		#[derive(Encode, Decode, Debug, PartialEq, Eq)]
		struct CompactU32(#[codec(compact)] u32);
		assert_decoded_eq!(
			MetaStorageReader::<Runtime>::execute(&mut MockHandle::new(
				input.encode(),
				Some(30_000_000),
				DUMMY_CTX.clone()
			)),
			Ok(Some(RawBytes(Bytes(CompactU32(900).encode()))))
		);
	})
}

#[test]
fn costs() {
	ext().execute_with(|| {
		let input =
			MetaStorageReaderInput::new("TestStorage", "SingleDefault", NoKey, Params::None);

		let mut handle = MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone());
		let res = MetaStorageReader::<Runtime>::execute(&mut handle).unwrap();
		let entry_meta = <Runtime as PalletStorageMetadataProvider>::pallet_storage_entry_metadata(
			"TestStorage",
			"SingleDefault",
		)
		.unwrap();
		assert_eq!(
			handle.gas_used,
			MetaStorageReader::<Runtime>::base_gas_cost(
				"TestStorage",
				"SingleDefault",
				&NoKey.into(),
				&entry_meta.ty
			)
			.unwrap() + MetaStorageReader::<Runtime>::output_gas_cost(res.output.len() - 1)
		);
		assert!(handle.gas_used > RawStorageReader::<Runtime>::base_gas_cost());

		let input =
			MetaStorageReaderInput::new("TestStorage", "SingleDefault", NoKey, Params::Len(100));

		let mut handle = MockHandle::new(input.encode(), Some(30_000_000), DUMMY_CTX.clone());
		MetaStorageReader::<Runtime>::execute(&mut handle).unwrap();
		assert_eq!(
			handle.gas_used,
			MetaStorageReader::<Runtime>::base_gas_cost(
				"TestStorage",
				"SingleDefault",
				&NoKey.into(),
				&entry_meta.ty
			)
			.unwrap() + MetaStorageReader::<Runtime>::output_gas_cost(102)
		);
	});
}
