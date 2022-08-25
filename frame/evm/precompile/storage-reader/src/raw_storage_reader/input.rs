use codec::{Decode, Encode};
use sp_std::prelude::*;

use crate::common::params::Params;

/// Input for the `RawStorageReader` precompile.
#[derive(Encode, Decode, Debug, Clone)]
pub struct RawStorageReaderInput {
    /// Raw bytes key.
    pub key: Vec<u8>,
    /// Additional params (offset and length).
    pub params: Params,
}
