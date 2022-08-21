use codec::{Decode, Encode};
use frame_support::{
    construct_runtime, decl_module, decl_storage,
    sp_runtime::{
        generic,
        traits::{BlakeTwo256, IdentityLookup},
    },
};
use frame_system::Config;
use sp_core::{ecdsa::Signature, hexdisplay::AsBytesRef, H160, H256};
use sp_std::prelude::*;

#[derive(Clone, Debug, Encode, Decode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    pub fn with_len(len: usize) -> Self {
        Self::from_to(0, len)
    }

    pub fn from_to(from: usize, to: usize) -> Self {
        Self((from..to).map(|i| i as u8).collect())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RawBytes(pub Bytes);

impl Encode for RawBytes {
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0 .0.as_bytes_ref())
    }
}

impl Decode for RawBytes {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let mut data = Vec::new();
        while input.remaining_len()?.unwrap_or_default() != 0 {
            data.push(input.read_byte()?)
        }

        Ok(Self(Bytes(data)))
    }

    // const TYPE_INFO: TypeInfo = TypeInfo::Unknown;
}

impl Default for Bytes {
    fn default() -> Self {
        Self::with_len(100)
    }
}

#[derive(Clone, Debug, Encode, Decode, Eq, Ord, PartialEq, PartialOrd)]
pub struct LargeBytes(Bytes);

impl Default for LargeBytes {
    fn default() -> Self {
        Self(Bytes::with_len(1_000_000))
    }
}

pub mod test_storage {
    use super::*;

    decl_storage! {
        trait Store for Module<T: Config> as TestStorage {
            pub Single: Option<Bytes>;
            pub SingleDefault: Bytes;

            pub LargeSingleDefault: LargeBytes;

            pub MapWithBlake2_128: map hasher(opaque_blake2_128) Bytes => Option<Bytes>;
            pub MapWithBlake2_256: map hasher(opaque_blake2_256) Bytes => Option<Bytes>;
            pub MapWithBlake2_128Concat: map hasher(blake2_128_concat) Bytes => Option<Bytes>;
            pub MapWithTwox128: map hasher(opaque_twox_128) Bytes => Option<Bytes>;
            pub MapWithTwox256: map hasher(opaque_twox_256) Bytes => Option<Bytes>;
            pub MapWithTwox64Concat: map hasher(twox_64_concat) Bytes => Option<Bytes>;
            pub MapWithIdentity: map hasher(identity) Bytes => Option<Bytes>;

            pub MapDefault: map hasher(opaque_blake2_128) Bytes => Bytes;

            pub DoubleMap: double_map hasher(opaque_blake2_128) Bytes, hasher(opaque_blake2_128) Bytes => Option<Bytes>;
            pub DoubleMapDefault: double_map hasher(opaque_blake2_128) Bytes, hasher(opaque_blake2_128) Bytes => Bytes;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin {}
    }
}

/// The address format for describing accounts.
pub type Address = u32;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<u32, BlakeTwo256>;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<u32, Call, SignedExtra>;

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        TestStorage: test_storage::{Module, Call, Storage}
    }
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = H160;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = generic::Header<u64, BlakeTwo256>;
    type Event = ();
    type BlockHashCount = ();
    type Version = ();
    type PalletInfo = PalletInfo1;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

pub struct PalletInfo1;

impl frame_support::traits::PalletInfo for PalletInfo1 {
    fn index<P: 'static>() -> Option<usize> {
        return Some(0);
    }

    fn name<P: 'static>() -> Option<&'static str> {
        return Some("TestName");
    }
}

crate::impl_pallet_storage_metadata_provider!(
    for Runtime:
        "TestStorage" => TestStorage
);
