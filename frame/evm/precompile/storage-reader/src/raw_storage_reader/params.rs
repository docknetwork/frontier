use core::{convert::TryInto, num::NonZeroU32, ops::Range};

use codec::{Decode, Encode};

use super::input::InputParams;

#[derive(Debug, Encode, Decode, Clone, Default, Copy)]
pub struct Params {
    pub(super) offset: Option<NonZeroU32>,
    pub(super) len: Option<u32>,
}

impl Params {
    /// Constructs new `Params`.
    pub fn new(offset: impl Into<Option<u32>>, len: impl Into<Option<u32>>) -> Self {
        Self {
            offset: offset.into().and_then(|val| val.try_into().ok()),
            len: len.into(),
        }
    }

    /// Attempts to convert given params to the range bounds.
    /// Result range is always valid, and has upper/lower bound less or equal to max.
    pub fn to_range(&self, max: usize) -> Option<Range<usize>> {
        if self.offset.is_none() && self.len.is_none() {
            return None;
        }

        let lower: usize = self.offset.map(NonZeroU32::get).unwrap_or_default() as usize;
        let upper = self
            .len
            .and_then(|len| (len as usize).checked_add(lower))
            .unwrap_or(max)
            .min(max);

        Some(lower.min(upper)..upper)
    }
}

impl From<InputParams> for Params {
    fn from(params: InputParams) -> Self {
        match params {
            InputParams::None => Self::default(),
            InputParams::Offset(offset) => Self::new(offset, None),
            InputParams::Len(len) => Self::new(None, len),
            InputParams::OffsetAndLen { offset, len } => Self::new(offset, len),
        }
    }
}
