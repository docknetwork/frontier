use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Debug, Clone)]
pub struct RawStorageReaderInput {
    pub key: Vec<u8>,
    pub params: InputParams,
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum InputParams {
    None,
    Offset(#[codec(compact)] u32),
    Len(#[codec(compact)] u32),
    OffsetAndLen {
        #[codec(compact)]
        offset: u32,
        #[codec(compact)]
        len: u32,
    },
}
