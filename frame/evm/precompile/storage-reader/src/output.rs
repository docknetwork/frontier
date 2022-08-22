use evm::ExitError;
use frame_metadata::DefaultByteGetter;
use sp_std::prelude::*;

use crate::raw_storage_reader::Params;

#[derive(Debug, Clone)]
pub enum RawStorageValue {
    None,
    Item(Vec<u8>),
}

impl RawStorageValue {
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Item(bytes) => bytes.len(),
        }
    }

    pub fn into_item(self) -> Option<Vec<u8>> {
        match self {
            Self::None => None,
            Self::Item(bytes) => Some(bytes),
        }
    }

    pub fn encode_to_bytes(&self) -> Vec<u8> {
        let mut output = vec![0; self.len() + 1];
        if let RawStorageValue::Item(bytes) = self {
            output[0] = 1;
            output[1..].copy_from_slice(bytes)
        }

        output
    }

    pub fn decode_from_bytes(bytes: &[u8]) -> Self {
        match bytes.get(0) {
            Some(1) => Self::Item(bytes[1..].to_vec()),
            _ => Self::None,
        }
    }

    pub fn or_default(
        self,
        params: &Params,
        byte_getter: Option<&DefaultByteGetter>,
    ) -> Result<Self, ExitError> {
        match self {
            Self::None => byte_getter
                .map(|byte_getter| byte_getter.0.default_byte())
                .map(|default_bytes| {
                    let range = params.to_range(default_bytes.len())?;
                    let sliced_value = default_bytes[range].to_vec();

                    Ok(sliced_value)
                })
                .transpose()
                .map(Into::into),
            item @ Self::Item(_) => Ok(item),
        }
    }
}

impl From<Option<Vec<u8>>> for RawStorageValue {
    fn from(bytes_opt: Option<Vec<u8>>) -> Self {
        bytes_opt.map_or(RawStorageValue::None, RawStorageValue::Item)
    }
}
