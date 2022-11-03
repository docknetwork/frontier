use core::iter::once;

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

/// Empty key used to access `Plain` values.
#[derive(Debug, Encode, Decode, Clone)]
pub struct NoKey;

/// Map key used to access keyed map entities.
#[derive(Debug, Encode, Decode, Clone)]
pub struct MapKey(Vec<Vec<u8>>);

impl MapKey {
	pub fn new_single(key: impl Encode) -> Self {
		Self(vec![key.encode()])
	}

	pub fn new_double(key1: impl Encode, key2: impl Encode) -> Self {
		Self(vec![key1.encode(), key2.encode()])
	}
}

/// All kinds of allowed keys.
#[derive(Debug, Clone)]
pub enum Key {
	/// For a `Plain` entity.
	NoKey(NoKey),
	/// For a `Map` entity.
	MapKey(MapKey),
}

impl Encode for Key {
	fn encode(&self) -> Vec<u8> {
		match self {
			Key::NoKey(_) => vec![0],
			Key::MapKey(MapKey(keys)) => once(keys.len() as u8)
				.chain(keys.iter().flat_map(Encode::encode))
				.collect(),
		}
	}
}

impl Decode for Key {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let len = input.read_byte()?;
		let mut res = Vec::with_capacity(len as usize);

		for _ in 0..len {
			res.push(Vec::decode(input)?);
		}

		let val = match len {
			0 => Self::NoKey(NoKey),
			_ => Self::MapKey(MapKey(res)),
		};

		Ok(val)
	}
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
			StorageEntryType::Plain(_) => Some(Weight::zero()),
			_ => None,
		}
	}
}

impl HashableKey for MapKey {
	fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
		let Self(keys) = self;

		match entry_type {
			StorageEntryType::Map { hashers, .. } => (hashers.len() == self.0.len()).then(|| {
				keys.iter()
					.zip(hashers)
					.flat_map(|(bytes, hasher)| hash_bytes_with(bytes, hasher))
					.collect()
			}),
			_ => None,
		}
	}

	fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
		let Self(keys) = self;

		match entry_type {
			StorageEntryType::Map { hashers, .. } => (hashers.len() == self.0.len()).then(|| {
				keys.iter()
					.map(Vec::len)
					.zip(hashers)
					.map(|(len, hasher)| hasher_weight(hasher, len))
					.fold(Weight::zero(), |acc, cur| acc.saturating_add(cur))
			}),
			_ => None,
		}
	}
}

impl From<NoKey> for Key {
	fn from(_: NoKey) -> Self {
		Self::NoKey(NoKey)
	}
}

impl From<MapKey> for Key {
	fn from(map_key: MapKey) -> Self {
		Self::MapKey(map_key)
	}
}

impl HashableKey for Key {
	fn to_hashed_key(&self, entry_type: &StorageEntryType) -> Option<Vec<u8>> {
		match self {
			Self::NoKey(key) => key.to_hashed_key(entry_type),
			Self::MapKey(map_key) => map_key.to_hashed_key(entry_type),
		}
	}

	fn hashing_weight(&self, entry_type: &StorageEntryType) -> Option<Weight> {
		match self {
			Self::NoKey(key) => key.hashing_weight(entry_type),
			Self::MapKey(map_key) => map_key.hashing_weight(entry_type),
		}
	}
}
