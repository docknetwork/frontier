use frame_metadata::DefaultByteGetter;
use sp_std::prelude::*;

use super::params::Params;

/// Represents raw value read from the storage.
#[derive(Debug, Clone)]
pub enum RawStorageValue {
    None,
    Item(Vec<u8>),
}

impl RawStorageValue {
    /// Returns underlying value bytes length.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Item(bytes) => bytes.len(),
        }
    }

    /// Attempts to convert `Self` into item.
    pub fn into_item(self) -> Option<Vec<u8>> {
        match self {
            Self::None => None,
            Self::Item(bytes) => Some(bytes),
        }
    }

    /// Encodes `Self` as bytes. The first byte is 0 or 1 depending on `self` being `None` or `Item`.
    pub fn encode_to_bytes(&self) -> Vec<u8> {
        let mut output = vec![0; self.len() + 1];
        if let RawStorageValue::Item(bytes) = self {
            output[0] = 1;
            output[1..].copy_from_slice(bytes)
        }

        output
    }

    /// Decodes `Self` from bytes.
    pub fn decode_from_bytes(bytes: &[u8]) -> Self {
        match bytes.get(0) {
            Some(1) => Self::Item(bytes[1..].to_vec()),
            _ => Self::None,
        }
    }

    /// If value is absent, attempts to replace it with default value provided by supplied getter.
    pub fn or_default(self, byte_getter: Option<&DefaultByteGetter>) -> Self {
        match self {
            Self::None => byte_getter
                .map(|byte_getter| byte_getter.0.default_byte())
                .into(),
            item @ Self::Item(_) => item,
        }
    }

    /// Applies offset and length params to the value.
    pub fn apply_params(self, params: &Params) -> Self {
        match self {
            Self::Item(bytes) => {
                let item = if let Some(range) = params.to_range(bytes.len()) {
                    bytes[range].to_vec()
                } else {
                    bytes
                };

                Self::Item(item)
            }
            Self::None => Self::None,
        }
    }
}

impl From<Option<Vec<u8>>> for RawStorageValue {
    fn from(bytes_opt: Option<Vec<u8>>) -> Self {
        bytes_opt.map_or(RawStorageValue::None, RawStorageValue::Item)
    }
}
