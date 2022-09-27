use codec::{Decode, Encode};
use sp_std::prelude::*;

use crate::common::params::Params;

/// Input for the `RawStorageReader` precompile.
#[derive(Encode, Decode, Debug, Clone)]
pub struct RawStorageReaderInput {
	/// Raw key bytes.
	pub key: Vec<u8>,
	/// Additional params (offset and length).
	pub params: Params,
}

impl RawStorageReaderInput {
	/// Constructs `RawStorageReaderInput` with supplied arguments.
	///
	/// - raw key bytes used to access storage entry
	/// - optional offset and length to be applied to the value bytes
	pub fn new(key: impl Into<Vec<u8>>, params: impl Into<Params>) -> Self {
		Self {
			key: key.into(),
			params: params.into(),
		}
	}
}
