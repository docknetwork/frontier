use core::convert::TryInto;
use core::marker::PhantomData;
use core::num::NonZeroU32;

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

        Self::read(&key, params.try_into()?)
            .and_then(|output| {
                let total_gas_cost = base_gas_cost + Self::output_gas_cost(output.len());
                if let Some(target_gas) = target_gas {
                    if target_gas < total_gas_cost {
                        return Err(ExitError::OutOfGas);
                    }
                }

                Ok((output, total_gas_cost))
            })
            .map(|(output, cost)| PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost,
                output: output.encode_to_bytes(),
                logs: Default::default(),
            })
    }
}

impl<T: pallet_evm::Config> RawStorageReader<T> {
    pub(super) fn read(
        raw_key: &[u8],
        Params { offset, len }: Params,
    ) -> Result<RawStorageValue, ExitError> {
        debug!(
            "`RawStorageReader` read: {:?} with offset = {:?}, len = {:?}",
            raw_key, offset, len
        );

        let value = if let Some(len) = len {
            let mut bytes = vec![0; len as usize];
            sp_io::storage::read(
                &raw_key,
                &mut bytes,
                offset.map(NonZeroU32::get).unwrap_or_default(),
            )
            .map(|bytes_read| {
                let bytes_read = bytes_read as usize;
                if bytes_read < bytes.len() {
                    bytes.truncate(bytes_read);
                }

                bytes
            })
        } else {
            let bytes = sp_io::storage::get(&raw_key);

            offset
                .zip(bytes.as_ref())
                .map(|(offset, bytes)| bytes[bytes.len().min(offset.get() as usize)..].to_vec())
                .or(bytes)
        };

        Ok(value.into())
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
