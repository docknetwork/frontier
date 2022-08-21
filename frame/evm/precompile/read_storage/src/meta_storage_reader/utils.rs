use frame_metadata::DecodeDifferent;

pub(crate) trait ToEither<L, R> {
    fn to_left(&self) -> Option<&L>;

    fn to_right(&self) -> Option<&R>;
}

impl<B, O> ToEither<B, O> for DecodeDifferent<B, O> {
    fn to_left(&self) -> Option<&B> {
        match self {
            DecodeDifferent::Encode(value) => Some(value),
            _ => None,
        }
    }

    fn to_right(&self) -> Option<&O> {
        match self {
            DecodeDifferent::Decoded(value) => Some(value),
            _ => None,
        }
    }
}
