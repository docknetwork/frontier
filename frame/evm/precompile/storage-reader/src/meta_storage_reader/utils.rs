use frame_metadata::{DecodeDifferent, StorageHasher};
use frame_support::weights::Weight;
use sp_std::prelude::*;

use super::weights::WeightInfo;

/// Used to convert type to one of two underlying types.
pub(crate) trait ToEither<L, R> {
    /// Attempts to convert `Self` to the left type.
    fn to_left(&self) -> Option<&L>;

    /// Attempts to convert `Self` to the right type.
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

/// Hashes given bytes using supplied hasher.
pub(crate) fn hash_bytes_with(bytes: &[u8], hasher: &StorageHasher) -> Vec<u8> {
    use frame_support::StorageHasher as _;

    match hasher {
        StorageHasher::Blake2_128 => frame_support::Blake2_128::hash(bytes).to_vec(),
        StorageHasher::Blake2_256 => frame_support::Blake2_256::hash(bytes).to_vec(),
        StorageHasher::Blake2_128Concat => frame_support::Blake2_128Concat::hash(bytes),
        StorageHasher::Twox128 => frame_support::Twox128::hash(bytes).to_vec(),
        StorageHasher::Twox256 => frame_support::Twox256::hash(bytes).to_vec(),
        StorageHasher::Twox64Concat => frame_support::Twox64Concat::hash(bytes),
        StorageHasher::Identity => frame_support::Identity::hash(bytes),
    }
}

/// Returns corresponding hasher weight.
pub(crate) fn hasher_weight(hasher: &StorageHasher, input_len: usize) -> Weight {
    let len = input_len as u32;

    match hasher {
        StorageHasher::Blake2_128 => <()>::blake2_128(len),
        StorageHasher::Blake2_256 => <()>::blake2_256(len),
        StorageHasher::Blake2_128Concat => <()>::blake2_128_concat(len),
        StorageHasher::Twox128 => <()>::twox_128(len),
        StorageHasher::Twox256 => <()>::twox_256(len),
        StorageHasher::Twox64Concat => <()>::twox_64_concat(len),
        StorageHasher::Identity => <()>::identity(len),
    }
}
