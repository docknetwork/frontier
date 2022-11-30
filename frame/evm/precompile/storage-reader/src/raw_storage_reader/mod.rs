use core::marker::PhantomData;

use codec::Decode;
use evm::{ExitError, ExitSucceed};
use fp_evm::{Precompile, PrecompileFailure};
use pallet_evm::{GasWeightMapping, PrecompileHandle, PrecompileOutput, PrecompileResult};

use frame_support::{log::debug, traits::Get};
use sp_std::borrow::Cow;

pub mod input;
#[cfg(test)]
mod tests;

use crate::common::output::RawStorageValue;
use input::RawStorageReaderInput;

/// Precompile allowing to read any storage data using provided raw key.
/// Unlike `MetaStorageReader`, default members won't be instantiated in case of absence.
/// If you need this behavior, consider using `MetaStorageReader` instead.
///
/// Output:
/// - 1 byte representing presence (1) or absence (0) of the value
/// - raw value bytes (with applied offset and length)
///
/// Input:
/// - compact encoded bytes len
/// - raw key bytes
/// - byte representing params: 0 - no additional params, 1 - offset, 2 - length, 3 - offset and length
/// - corresponding compact encoded offset, length or offset followed by length
///
/// # Example call
///
/// ```rust
///  # use codec::{Encode, Decode};
///  # use pallet_evm_precompile_storage_reader::raw_storage_reader::input::RawStorageReaderInput;
///  # use pallet_evm_precompile_storage_reader::params::Params;
///  # let input =
///  RawStorageReaderInput::new(
///		(0u8..10).collect::<Vec<_>>(),
///		Params::Offset(10),
///	 );
///  // will be encoded as the following sequence of bytes
///  # assert_eq!(
///  vec![
///     40, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, // - Key bytes: length(bytes) << 2 followed by bytes
///     1, // - Offset param will be used
///     40 // - Offset value (10 << 2)
///  ]
///  # , input.encode());
/// ```
///
pub struct RawStorageReader<T>(PhantomData<T>);

impl<T: pallet_evm::Config> Precompile for RawStorageReader<T> {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let mut input = handle.input();
		let target_gas = handle.gas_limit();

		debug!(
			"`RawStorageReader` input: {:?}, target gas: {:?}",
			input, target_gas
		);

		let RawStorageReaderInput { key, params } =
			RawStorageReaderInput::decode(&mut input).map_err(Error::Decoding)?;

		let base_gas_cost = Self::base_gas_cost();
		crate::ensure_enough_gas!(target_gas >= base_gas_cost);

		let raw_output = Self::read(&key);

		let total_gas_cost = base_gas_cost.saturating_add(Self::output_gas_cost(raw_output.len()));
		crate::ensure_enough_gas!(target_gas >= total_gas_cost);

		let output = raw_output.apply_params(&params);

		handle.record_cost(total_gas_cost)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: output.encode_to_bytes(),
		})
	}
}

impl<T: pallet_evm::Config> RawStorageReader<T> {
	/// Reads the value from the storage using supplied raw key.
	pub(super) fn read(raw_key: &[u8]) -> RawStorageValue {
		debug!("`RawStorageReader` read: {:?}", raw_key);

		sp_io::storage::get(&raw_key).into()
	}

	pub(super) fn base_gas_cost() -> u64 {
		T::GasWeightMapping::weight_to_gas(T::DbWeight::get().reads(1))
	}

	pub(super) fn output_gas_cost(output_len: usize) -> u64 {
		T::GasWeightMapping::weight_to_gas(
			T::ByteReadWeight::get().saturating_mul(output_len as u64),
		)
	}
}

pub enum Error {
	Decoding(codec::Error),
}

impl From<Error> for PrecompileFailure {
	fn from(err: Error) -> Self {
		match err {
			Error::Decoding(_) => PrecompileFailure::Error {
				exit_status: ExitError::Other(Cow::Borrowed("Failed to decode")),
			},
		}
	}
}
