#![cfg_attr(not(feature = "std"), no_std)]

pub mod meta_storage_reader;
pub mod raw_storage_reader;

pub use meta_storage_reader::MetaStorageReader;
pub use raw_storage_reader::RawStorageReader;

#[macro_export]
macro_rules! impl_pallet_storage_metadata_provider {
    (for $name: ident: $($pallet_name: literal => $pallet: ident),+) => {
        use frame_metadata::StorageMetadata;

        impl $crate::meta_storage_reader::PalletStorageMetadataProvider for $name {
            fn pallet_storage_metadata(pallet: &str) -> Option<StorageMetadata> {
                match pallet {
                    $($pallet_name => Some($pallet::storage_metadata())),+
                    ,_ => None
                }
            }
        }
    }
}
