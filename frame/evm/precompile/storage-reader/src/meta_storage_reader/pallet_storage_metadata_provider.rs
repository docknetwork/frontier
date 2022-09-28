use frame_metadata::{PalletStorageMetadata, StorageEntryMetadata};

/// Provides metadata for the pallet storage.
pub trait PalletStorageMetadataProvider {
	/// Provides metadata for the storage of the pallet with supplied name.
	fn pallet_storage_metadata(pallet: &str) -> Option<PalletStorageMetadata>;

	/// Provides metadata for the storage entry of the pallet with supplied name.
	/// Returns an error if metadata is invalid (i.e. encoded not properly).
	fn pallet_storage_entry_metadata(pallet: &str, entry: &str) -> Option<StorageEntryMetadata> {
		if let Some(pallet_meta) = Self::pallet_storage_metadata(pallet) {
			for item in pallet_meta.entries {
				if &item.name == &entry {
					return Some(item.clone());
				}
			}
		}

		None
	}
}

/// Implements `PalletStorageMetadataProvider` for the provided identifier.
#[macro_export]
macro_rules! impl_pallet_storage_metadata_provider {
    (for $name: ident: $($pallet_name: literal => $pallet: ident),+) => {
        impl $crate::meta_storage_reader::PalletStorageMetadataProvider for $name {
            fn pallet_storage_metadata(pallet: &str) -> Option<$crate::frame_metadata::PalletStorageMetadata> {
                match pallet {
                    $($pallet_name => Some($pallet::storage_metadata())),+
                    ,_ => None
                }
            }
        }
    }
}
