use core::{convert::TryInto, num::NonZeroU32, ops::Range};

use codec::{Decode, Encode};

/// Input params allowing to specify offset and length for the raw value bytes.
#[derive(Debug, Encode, Decode, Clone)]
pub enum Params {
    /// No params specified.
    None,
    /// Apply offset to the value bytes.
    Offset(#[codec(compact)] u32),
    /// Specify max amount of bytes to be returned by the reader.
    Len(#[codec(compact)] u32),
    /// Apply offset and specify the max amount of bytes to be returned by the reader.
    OffsetAndLen {
        #[codec(compact)]
        offset: u32,
        #[codec(compact)]
        len: u32,
    },
}

impl Params {
    /// Attempts to convert given params to the range bounds.
    /// Result range is always valid, and has upper/lower bound less or equal to max.
    pub fn to_range(&self, max: usize) -> Option<Range<usize>> {
        let (offset, len): (Option<NonZeroU32>, _) = match *self {
            Self::None => (None, None),
            Self::Offset(offset) => (offset.try_into().ok(), None),
            Self::Len(len) => (None, Some(len)),
            Self::OffsetAndLen { offset, len } => (offset.try_into().ok(), Some(len)),
        };

        if offset.is_none() && len.is_none() {
            return None;
        }

        let lower = offset.map(NonZeroU32::get).unwrap_or_default() as usize;
        let upper = len
            .and_then(|len| (len as usize).checked_add(lower))
            .unwrap_or(max)
            .min(max);

        Some(lower.min(upper)..upper)
    }
}
