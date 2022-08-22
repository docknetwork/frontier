use super::ToEither;
use evm::ExitError;
use frame_metadata::{StorageEntryMetadata, StorageMetadata};
use sp_std::borrow::Cow;

#[derive(Debug, Clone, Copy, Default)]
pub struct InvalidMetadata;

/// Provides metadata for the pallet storage.
pub trait PalletStorageMetadataProvider {
    /// Provides metadata for the storage of the pallet with supplied name.
    fn pallet_storage_metadata(pallet: &str) -> Option<StorageMetadata>;

    /// Provides metadata for the storage entry of the pallet with supplied name.
    /// Returns an error if metadata is invalid (i.e. encoded not properly).
    fn pallet_storage_entry_metadata(
        pallet: &str,
        entry: &str,
    ) -> Result<Option<StorageEntryMetadata>, InvalidMetadata> {
        if let Some(pallet_meta) = Self::pallet_storage_metadata(pallet) {
            for item in *pallet_meta.entries.to_left().ok_or(InvalidMetadata)? {
                if item.name.to_left().ok_or(InvalidMetadata)? == &entry {
                    return Ok(Some(item.clone()));
                }
            }
        }

        Ok(None)
    }
}

#[macro_export]
macro_rules! impl_pallet_storage_metadata_provider {
    (for $name: ident: $($pallet_name: literal => $pallet: ident),+) => {
        use $crate::frame_metadata::StorageMetadata;

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

impl From<InvalidMetadata> for ExitError {
    fn from(InvalidMetadata: InvalidMetadata) -> Self {
        ExitError::Other(Cow::Borrowed("Invalid metadata"))
    }
}
