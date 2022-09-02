use codec::{Decode, Encode};

use super::key::Key;
pub use crate::common::params::Params;

/// Input for `MetaStorageReader` precompile.
#[derive(Debug, Encode, Decode, Clone)]
pub struct MetaStorageReaderInput {
    /// Target pallet name (for ex. `System`)
    pub pallet: String,
    /// Target pallet storage entry (for ex. `Now`)
    pub entry: String,
    /// Key used to access storage entry member.
    pub key: Key,
    /// Additional params (offset and length).
    pub params: Params,
}

impl MetaStorageReaderInput {
    /// Constructs new `MetaStorageReaderInput` with given arguments.
    /// 
    /// - pallet name (UTF-8)
    /// - pallet storage entry name (UTF-8)
    /// - key for the storage entry
    /// - optional offset and length to be applied to the value bytes
    pub fn new(
        pallet: impl Into<String>,
        entry: impl Into<String>,
        key: impl Into<Key>,
        params: impl Into<Params>,
    ) -> Self {
        Self {
            pallet: pallet.into(),
            entry: entry.into(),
            key: key.into(),
            params: params.into(),
        }
    }

    /// Clones given input and replaces old key with the supplied key.
    pub fn with_replaced_key(&self, key: impl Into<Key>) -> Self {
        let mut new_self = self.clone();
        new_self.key = key.into();
        new_self
    }
}
