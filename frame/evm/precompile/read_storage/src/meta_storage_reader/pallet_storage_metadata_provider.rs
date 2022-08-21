use super::ToEither;
use evm::ExitError;
use frame_metadata::{StorageEntryMetadata, StorageMetadata};
use sp_std::borrow::Cow;

pub struct InvalidMetadata;

pub trait PalletStorageMetadataProvider {
    fn pallet_storage_metadata(pallet: &str) -> Option<StorageMetadata>;

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

impl From<InvalidMetadata> for ExitError {
    fn from(InvalidMetadata: InvalidMetadata) -> Self {
        ExitError::Other(Cow::Borrowed("Invalid metadata"))
    }
}
