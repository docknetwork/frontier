use core::{
    convert::{TryFrom, TryInto},
    num::NonZeroU32,
};

use codec::{Decode, Encode};
use evm::ExitError;
use sp_std::borrow::Cow;

use super::input::InputParams;

pub const MAX_BYTES_LEN: u32 = 0x100000;

#[derive(Debug, Encode, Decode, Clone, Default, Copy)]
pub struct Params {
    pub offset: Option<NonZeroU32>,
    pub len: Option<u32>,
}

impl Params {
    pub fn new(
        offset: impl Into<Option<u32>>,
        len: impl Into<Option<u32>>,
    ) -> Result<Self, ParamsError> {
        let len = len.into();
        if len.as_ref().map_or(false, |&val| val > MAX_BYTES_LEN) {
            return Err(ParamsError::LengthExceedsLimit);
        }

        Ok(Self {
            offset: offset.into().and_then(|val| val.try_into().ok()),
            len,
        })
    }

    pub fn lower_upper(&self) -> Result<(usize, Option<usize>), ParamsError> {
        let lower: usize = self
            .offset
            .map(NonZeroU32::get)
            .unwrap_or_default()
            .try_into()
            .map_err(|_| ParamsError::Overflow)?;
        let upper = self
            .len
            .map(|len| {
                let len: usize = len.try_into().map_err(|_| ParamsError::Overflow)?;

                len.checked_add(lower)
                    .ok_or(ParamsError::OffsetPlusLengthOverflow)
            })
            .transpose()?;

        Ok((lower, upper))
    }
}

pub enum ParamsError {
    LengthExceedsLimit,
    OffsetPlusLengthOverflow,
    Overflow,
}

impl From<ParamsError> for ExitError {
    fn from(err: ParamsError) -> Self {
        match err {
            ParamsError::Overflow => ExitError::Other(Cow::Borrowed("Params overflow")),
            ParamsError::LengthExceedsLimit => {
                ExitError::Other(Cow::Borrowed("Length exceeds limit"))
            }
            ParamsError::OffsetPlusLengthOverflow => {
                ExitError::Other(Cow::Borrowed("offset + length overflow"))
            }
        }
    }
}

impl TryFrom<InputParams> for Params {
    type Error = ParamsError;

    fn try_from(params: InputParams) -> Result<Self, Self::Error> {
        match params {
            InputParams::None => Ok(Self::default()),
            InputParams::Offset(offset) => Self::new(offset, None),
            InputParams::Len(len) => Self::new(None, len),
            InputParams::OffsetAndLen { offset, len } => Self::new(offset, len),
        }
    }
}
