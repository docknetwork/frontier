use core::marker::PhantomData;

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use fp_evm::Precompile;
use frame_support::log::debug;
use pallet_evm::GasWeightMapping;
use sp_std::{borrow::Cow, prelude::*};

use codec::Decode;
use frame_metadata::{StorageEntryModifier, StorageEntryType};
use input::MetaStorageReaderInput;

use crate::raw_storage_reader::RawStorageReader;
pub use pallet_storage_metadata_provider::*;
use utils::*;

use key::Key;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks;
pub mod input;
pub mod key;
mod pallet_storage_metadata_provider;
#[cfg(test)]
mod tests;
mod utils;
mod weights;

/// Precompile allows reading any pallet storage member data using provided key hashed according to the corresponding metadata.
/// Unlike with `RawStorageReader`, default members will be instantiated in case of absence.
///
/// Output:
/// - 1 byte representing presence (1) or absence (0) of the value
/// - raw value bytes (with applied offset and length)
///
/// Input:
/// - compact encoded length of the pallet name (UTF-8)
/// - pallet name (UTF-8)
/// - compact encoded length of the pallet's storage member name (UTF-8)
/// - pallet's storage member name (UTF-8)
/// - byte representing key type: 0 - NoKey, 1 - MapKey, 2 - DoubleMapKey
/// - encoded key
///   - nothing for NoKey
///   - compact encoded length of the underlying key bytes followed by bytes for MapKey
///   - two compact encoded lengths of the underlying key bytes followed by bytes for DoubleMapKey
/// - byte representing params: 0 - no additional params, 1 - offset, 2 - length, 3 - offset and length
/// - corresponding compact encoded offset, length or offset followed by length
#[derive(Default, Debug)]
pub struct MetaStorageReader<T>(PhantomData<T>);

impl<T: pallet_evm::Config + PalletStorageMetadataProvider> Precompile for MetaStorageReader<T> {
    fn execute(
        mut input: &[u8],
        target_gas: Option<u64>,
        _: &Context,
    ) -> core::result::Result<PrecompileOutput, ExitError> {
        debug!(
            "`MetaStorageReader` input: {:?}, target gas: {:?}",
            input, target_gas
        );

        let MetaStorageReaderInput {
            pallet,
            entry,
            key,
            params,
        } = MetaStorageReaderInput::decode(&mut input).map_err(Error::Decoding)?;

        debug!(
            "`MetaStorageReader` decoded input: pallet = {:?}, entry = {:?}, key = {:?}, params = {:?}",
            pallet,
            entry,
            key,
            params
        );

        let entry_meta = T::pallet_storage_entry_metadata(&pallet, &entry)?
            .ok_or(Error::PalletStorageEntryNotFound)?;
        let default_byte_getter = (entry_meta.modifier == StorageEntryModifier::Default)
            .then(|| entry_meta.default.to_left().ok_or(Error::InvalidMetadata))
            .transpose()?;

        let base_gas_cost = Self::base_gas_cost(&pallet, &entry, &key, &entry_meta.ty)?;
        crate::ensure_enough_gas!(target_gas >= base_gas_cost);

        let storage_key = key
            .to_pallet_entry_storage_key(&pallet, &entry, &entry_meta.ty)
            .ok_or(Error::InvalidKey)?;

        let raw_output = RawStorageReader::<T>::read(&storage_key).or_default(default_byte_getter);

        let total_gas_cost = base_gas_cost.saturating_add(Self::output_gas_cost(raw_output.len()));
        crate::ensure_enough_gas!(target_gas >= total_gas_cost);

        let output = raw_output.apply_params(&params);

        Ok(PrecompileOutput {
            cost: total_gas_cost,
            output: output.encode_to_bytes(),
            exit_status: ExitSucceed::Returned,
            logs: Default::default(),
        })
    }
}

impl<T: pallet_evm::Config> MetaStorageReader<T> {
    fn base_gas_cost(
        pallet: &str,
        entry: &str,
        key: &Key,
        entry_type: &StorageEntryType,
    ) -> Result<u64, ExitError> {
        let key_hashing_weight = key
            .full_hashing_weight(pallet, entry, entry_type)
            .ok_or(Error::InvalidKey)?;

        Ok(T::GasWeightMapping::weight_to_gas(key_hashing_weight)
            .saturating_add(RawStorageReader::<T>::base_gas_cost()))
    }

    fn output_gas_cost(output_len: usize) -> u64 {
        RawStorageReader::<T>::output_gas_cost(output_len)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    PalletStorageEntryNotFound,
    InvalidMetadata,
    InvalidKey,
    Decoding(codec::Error),
}

impl From<Error> for ExitError {
    fn from(err: Error) -> Self {
        let msg = match err {
            Error::InvalidMetadata => "Invalid metadata",
            Error::PalletStorageEntryNotFound => "Pallet storage entry not found",
            Error::InvalidKey => "Invalid key",
            Error::Decoding(_) => "Failed to decode input",
        };

        ExitError::Other(Cow::Borrowed(msg))
    }
}
