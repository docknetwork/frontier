use bytes::Bytes;
use sp_std::prelude::*;

use super::params::Params;

/// Represents raw value read from the storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawStorageValue {
	/// Item isn't found.
	None,
	/// Item is present.
	Item(Bytes),
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
	/// If you need to encode the value to bytes, use `apply_params` and `encode_to_bytes` instead.
	pub fn into_item(self) -> Option<Bytes> {
		match self {
			Self::None => None,
			Self::Item(bytes) => Some(bytes),
		}
	}

	/// If value is absent, attempts to replace it with provided bytes.
	pub fn or_default(self, default_bytes: Option<Vec<u8>>) -> Self {
		match self {
			Self::None => default_bytes.map(Into::into).into(),
			item @ Self::Item(_) => item,
		}
	}

	/// Applies offset and length params to the value.
	pub fn apply_params(self, params: &Params) -> OutputStorageValue {
		let value = match self {
			Self::Item(bytes) => {
				let item = if let Some(range) = params.to_range(bytes.len()) {
					bytes[range].to_vec().into()
				} else {
					bytes
				};

				Self::Item(item)
			}
			Self::None => Self::None,
		};

		OutputStorageValue(value)
	}

	/// Decodes `Self` from bytes.
	pub fn decode_from_bytes(bytes: &[u8]) -> Self {
		match bytes.get(0) {
			Some(1) => RawStorageValue::Item(bytes[1..].to_vec().into()),
			_ => RawStorageValue::None,
		}
	}
}

impl From<Option<Bytes>> for RawStorageValue {
	fn from(bytes_opt: Option<Bytes>) -> Self {
		bytes_opt.map_or(RawStorageValue::None, RawStorageValue::Item)
	}
}

/// Represents value read from the storage with applied params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputStorageValue(RawStorageValue);

impl OutputStorageValue {
	/// Encodes `Self` as bytes. The first byte is 0 or 1 depending on `self` being `None` or `Item`.
	pub fn encode_to_bytes(&self) -> Vec<u8> {
		let Self(raw_value) = self;

		let mut output = vec![0; self.0.len() + 1];
		if let RawStorageValue::Item(bytes) = raw_value {
			output[0] = 1;
			output[1..].copy_from_slice(bytes)
		}

		output
	}
}
