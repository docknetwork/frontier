use codec::{Decode, Encode};
use frame_metadata::{StorageEntryType, StorageHasher};
use frame_support::{StorageHasher as _, Twox128};
use sp_std::prelude::*;

pub trait ToHashedKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>>;
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct NoKey;

#[derive(Debug, Encode, Decode, Clone)]
pub struct MapKey(Vec<u8>);

impl MapKey {
    pub fn new(key: impl Encode) -> Self {
        Self(key.encode())
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct DoubleMapKey(Vec<u8>, Vec<u8>);

impl DoubleMapKey {
    pub fn new(key1: impl Encode, key2: impl Encode) -> Self {
        Self(key1.encode(), key2.encode())
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum Key {
    NoKey(NoKey),
    MapKey(MapKey),
    DoubleMapKey(DoubleMapKey),
}

impl Key {
    pub fn to_pallet_entry_storage_key(
        &self,
        pallet: &str,
        entry: &str,
        entry_type: &StorageEntryType,
    ) -> Option<Vec<u8>> {
        let storage_key = self.to_hashed_key(entry_type)?;

        let mut final_key = vec![0u8; 32 + storage_key.len()];
        final_key[0..16].copy_from_slice(&Twox128::hash(pallet.as_bytes()));
        final_key[16..32].copy_from_slice(&Twox128::hash(entry.as_bytes()));
        final_key[32..].copy_from_slice(&storage_key);

        Some(final_key)
    }
}

impl ToHashedKey for NoKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        match entry_type {
            StorageEntryType::Plain(_) => Some(vec![]),
            _ => None,
        }
    }
}

impl ToHashedKey for MapKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        let Self(key) = self;

        match entry_type {
            StorageEntryType::Map { hasher, .. } => Some(hash_bytes_with(&key, hasher)),
            _ => None,
        }
    }
}

impl ToHashedKey for DoubleMapKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        let Self(key1, key2) = self;

        match entry_type {
            StorageEntryType::DoubleMap {
                hasher,
                key2_hasher,
                ..
            } => {
                let hash1 = hash_bytes_with(key1, hasher);
                let hash2 = hash_bytes_with(key2, key2_hasher);
                let final_key = [hash1, hash2].concat();

                Some(final_key)
            }
            _ => None,
        }
    }
}

impl From<NoKey> for Key {
    fn from(NoKey: NoKey) -> Self {
        Self::NoKey(NoKey)
    }
}

impl From<MapKey> for Key {
    fn from(map_key: MapKey) -> Self {
        Self::MapKey(map_key)
    }
}

impl From<DoubleMapKey> for Key {
    fn from(double_map_key: DoubleMapKey) -> Self {
        Self::DoubleMapKey(double_map_key)
    }
}

impl ToHashedKey for Key {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        match self {
            Self::NoKey(key) => key.to_hashed_key(entry_type),
            Self::MapKey(map_key) => map_key.to_hashed_key(entry_type),
            Self::DoubleMapKey(double_map_key) => double_map_key.to_hashed_key(entry_type),
        }
    }
}

fn hash_bytes_with(bytes: &[u8], hasher: &StorageHasher) -> Vec<u8> {
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
