use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Debug, Clone)]
pub struct Input {
    pub key: Vec<u8>,
    pub params: InputParams,
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum InputParams {
    None,
    Offset(u32),
    Len(u32),
    OffsetAndLen { offset: u32, len: u32 },
}
