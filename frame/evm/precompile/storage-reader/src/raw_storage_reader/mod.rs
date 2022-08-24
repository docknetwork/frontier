use core::marker::PhantomData;

use codec::Decode;
use evm::executor::PrecompileOutput;
use evm::{Context, ExitError, ExitSucceed};
use fp_evm::Precompile;
use pallet_evm::GasWeightMapping;

use frame_support::{log::debug, traits::Get};
pub use params::*;
use sp_std::{borrow::Cow, prelude::*};

pub mod input;
mod params;
#[cfg(test)]
mod tests;

use crate::output::RawStorageValue;
use input::RawStorageReaderInput;

/// Precompile allowing to read any storage data using provided raw key.
/// Unlike `MetaStorageReader`, default members won't be instantiated in case of absence.
/// If you need this behaviour, consider using `MetaStorageReader` instead.
/// Input:
/// - compact encoded bytes len
/// - raw key bytes
/// - byte representing params: 0 - no additional params, 1 - offset, 2 - length, 3 - offset and length
/// - corresponding compact encoded offset, length, or offset followed by length
pub struct RawStorageReader<T: pallet_evm::Config>(PhantomData<T>);

impl<T: pallet_evm::Config> Precompile for RawStorageReader<T> {
    fn execute(
        mut input: &[u8],
        target_gas: Option<u64>,
        _: &Context,
    ) -> core::result::Result<PrecompileOutput, ExitError> {
        debug!(
            "`RawStorageReader` input: {:?}, target gas: {:?}",
            input, target_gas
        );

        let RawStorageReaderInput { key, params } =
            RawStorageReaderInput::decode(&mut input).map_err(Error::Decoding)?;

        let base_gas_cost = Self::base_gas_cost();
        if let Some(target_gas) = target_gas {
            if target_gas < base_gas_cost {
                return Err(ExitError::OutOfGas);
            }
        }

        let raw_output = Self::read(&key)?;

        let total_gas_cost = base_gas_cost + Self::output_gas_cost(raw_output.len());
        if let Some(target_gas) = target_gas {
            if target_gas < total_gas_cost {
                return Err(ExitError::OutOfGas);
            }
        }

        let output = raw_output.apply_params(&params.into())?;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: total_gas_cost,
            output: output.encode_to_bytes(),
            logs: Default::default(),
        })
    }
}

impl<T: pallet_evm::Config> RawStorageReader<T> {
    pub(super) fn read(raw_key: &[u8]) -> Result<RawStorageValue, ExitError> {
        debug!("`RawStorageReader` read: {:?}", raw_key);

        Ok(sp_io::storage::get(&raw_key).into())
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
    InvalidParams,
    Decoding(codec::Error),
}

impl From<Error> for ExitError {
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidParams => ExitError::Other(Cow::Borrowed("Invalid params")),
            Error::Decoding(_) => ExitError::Other(Cow::Borrowed("Failed to decode")),
        }
    }
}
