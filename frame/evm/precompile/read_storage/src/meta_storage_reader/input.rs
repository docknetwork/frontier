use codec::{Decode, Encode};
use sp_std::borrow::Cow;

use crate::raw_storage_reader::input::InputParams;

use super::key::Key;

#[derive(Debug, Encode, Decode, Clone)]
pub struct Input<'a> {
    pub pallet: Cow<'a, str>,
    pub entry: Cow<'a, str>,
    pub key: Key,
    pub params: InputParams,
}

impl<'a> Input<'a> {
    pub fn new(
        pallet: impl Into<Cow<'a, str>>,
        entry: impl Into<Cow<'a, str>>,
        key: impl Into<Key>,
        params: impl Into<InputParams>,
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
