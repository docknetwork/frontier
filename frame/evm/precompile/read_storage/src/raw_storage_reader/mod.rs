use core::convert::TryInto;
use core::num::NonZeroU32;

use codec::Decode;
use evm::{ExitError, ExitSucceed};
use fp_evm::LinearCostPrecompile;

use sp_std::prelude::*;
use sp_std::{borrow::Cow, vec::Vec};

pub mod input;
mod params;

use params::*;

use self::input::Input;

pub struct RawStorageReader;

impl LinearCostPrecompile for RawStorageReader {
    const BASE: u64 = 3000;
    const WORD: u64 = 0;

    fn execute(mut input: &[u8], cost: u64) -> Result<(ExitSucceed, Vec<u8>), ExitError> {
        let Input { key, params } = Input::decode(&mut input).map_err(Error::Decoding)?;

        Self::read(&key, params.try_into()?, cost).map(|res| (ExitSucceed::Returned, res))
    }
}

impl RawStorageReader {
    pub(super) fn read(
        raw_key: &[u8],
        Params { offset, len }: Params,
        cost: u64,
    ) -> Result<Vec<u8>, ExitError> {
        frame_support::log::error!("{:?} {:?}", offset, len);

        let value = if let Some(len) = len {
            let mut value_out = vec![0; len as usize];
            let bytes_read = sp_io::storage::read(
                &raw_key,
                &mut value_out,
                offset.map(NonZeroU32::get).unwrap_or_default(),
            )
            .ok_or(Error::ItemNotFound)? as usize;
            if bytes_read < value_out.len() {
                value_out.truncate(bytes_read);
            }

            value_out
        } else {
            let bytes = sp_io::storage::get(&raw_key).ok_or(Error::ItemNotFound)?;

            offset
                .map(|offset| {
                    Some(&bytes[bytes.len().min(offset.get() as usize)..])
                        .ok_or(Error::InvalidParams)
                        .map(<[_]>::to_vec)
                })
                .transpose()?
                .unwrap_or(bytes)
        };

        Ok(value)
    }
}

pub enum Error {
    ItemNotFound,
    InvalidParams,
    Decoding(codec::Error),
}

impl From<Error> for ExitError {
    fn from(err: Error) -> Self {
        match err {
            Error::ItemNotFound => ExitError::Other(Cow::Borrowed("Item not found")),
            Error::InvalidParams => ExitError::Other(Cow::Borrowed("Invalid params")),
            Error::Decoding(_) => ExitError::Other(Cow::Borrowed("Failed to decode")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn encode() {
        let encoded = Input {
            key: vec![1],
            offset: None,
            len: None,
        };

        panic!("{:?}", encoded.encode());
    }*/
}
