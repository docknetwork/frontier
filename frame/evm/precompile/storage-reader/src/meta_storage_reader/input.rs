use codec::{Decode, Encode};
use sp_std::borrow::Cow;

pub use crate::common::params::Params;

use super::key::Key;

/// Input for `MetaStorageReader` precompile.
#[derive(Debug, Encode, Decode, Clone)]
pub struct MetaStorageReaderInput<'a> {
    /// Target pallet name (for ex. `System`)
    pub pallet: Cow<'a, str>,
    /// Target pallet storage entry (for ex. `Now`)
    pub entry: Cow<'a, str>,
    /// Key used to access storage entry member.
    pub key: Key,
    /// Additional params (offset and length).
    pub params: Params,
}

impl<'a> MetaStorageReaderInput<'a> {
    pub fn new(
        pallet: impl Into<Cow<'a, str>>,
        entry: impl Into<Cow<'a, str>>,
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

    pub fn with_replaced_key(&self, key: impl Into<Key>) -> Self {
        let mut new_self = self.clone();
        new_self.key = key.into();
        new_self
    }
}
