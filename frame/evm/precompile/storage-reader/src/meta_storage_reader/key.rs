use super::utils::{hash_bytes_with, hasher_weight};
use codec::{Decode, Encode};
use frame_metadata::{StorageEntryType, StorageHasher};
use frame_support::{weights::Weight, StorageHasher as _, Twox128};
use sp_std::prelude::*;

/// Key that can be hashed according to the provided metadata.
pub trait HashableKey {
    /// Attempts to hash given key using supplied metadata.
    /// Returns `None` if the key is incompatible with the given metadata.
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>>;

    /// Attempts to calculate hashing weight for the given key using supplied metadata.
    /// Returns `None` if the key is incompatible with the given metadata.
    fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight>;
}

/// Empty key used to access single values.
#[derive(Debug, Encode, Decode, Clone)]
pub struct NoKey;

/// Map key used to access single-key map entities.
#[derive(Debug, Encode, Decode, Clone)]
pub struct MapKey(Vec<u8>);

impl MapKey {
    pub fn new(key: impl Encode) -> Self {
        Self(key.encode())
    }
}

/// Double map key used to access double-key map entities.
#[derive(Debug, Encode, Decode, Clone)]
pub struct DoubleMapKey(Vec<u8>, Vec<u8>);

impl DoubleMapKey {
    pub fn new(key1: impl Encode, key2: impl Encode) -> Self {
        Self(key1.encode(), key2.encode())
    }
}

/// All kinds of allowed keys.
#[derive(Debug, Encode, Decode, Clone)]
pub enum Key {
    /// For a `Plain` entity.
    NoKey(NoKey),
    /// For a `Map` entity.
    MapKey(MapKey),
    /// For a `DoubleMap` entity.
    DoubleMapKey(DoubleMapKey),
}

impl Key {
    /// Converts self to the full-featured hashed storage key with prefix.
    /// Returns `None` if the key is incompatible with the provided metadata.
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

    /// Calculates hash for converting to the full-featured hashed storage key with prefix.
    /// Returns `None` if the key is incompatible with the provided metadata.
    pub fn full_hashing_weight(
        &self,
        pallet: &str,
        entry: &str,
        entry_type: &StorageEntryType,
    ) -> Option<Weight> {
        let res = self.hashing_weight(entry_type)?
            + hasher_weight(&StorageHasher::Twox128, pallet.as_bytes().len())
            + hasher_weight(&StorageHasher::Twox128, entry.as_bytes().len());

        Some(res)
    }
}

impl HashableKey for NoKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        match entry_type {
            StorageEntryType::Plain(_) => Some(vec![]),
            _ => None,
        }
    }

    fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
        match entry_type {
            StorageEntryType::Plain(_) => Some(0),
            _ => None,
        }
    }
}

impl HashableKey for MapKey {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        let Self(key) = self;

        match entry_type {
            StorageEntryType::Map { hasher, .. } => Some(hash_bytes_with(&key, hasher)),
            _ => None,
        }
    }

    fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
        let Self(key) = self;

        match entry_type {
            StorageEntryType::Map { hasher, .. } => Some(hasher_weight(hasher, key.len())),
            _ => None,
        }
    }
}

impl HashableKey for DoubleMapKey {
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

    fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
        let Self(key1, key2) = self;

        match entry_type {
            StorageEntryType::DoubleMap { hasher, .. } => {
                Some(hasher_weight(hasher, key1.len()) + hasher_weight(hasher, key2.len()))
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

impl HashableKey for Key {
    fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
        match self {
            Self::NoKey(key) => key.to_hashed_key(entry_type),
            Self::MapKey(map_key) => map_key.to_hashed_key(entry_type),
            Self::DoubleMapKey(double_map_key) => double_map_key.to_hashed_key(entry_type),
        }
    }

    fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
        match self {
            Self::NoKey(key) => key.hashing_weight(entry_type),
            Self::MapKey(map_key) => map_key.hashing_weight(entry_type),
            Self::DoubleMapKey(double_map_key) => double_map_key.hashing_weight(entry_type),
        }
    }
}
