use codec::{Decode, Encode};
use frame_support::{
    construct_runtime, decl_module, decl_storage,
    pallet_prelude::Weight,
    parameter_types,
    sp_runtime::{
        self, generic,
        traits::{BlakeTwo256, IdentityLookup},
        AccountId32,
    },
    weights::constants::RocksDbWeight,
};
use frame_system::{Config, RawOrigin};
use pallet_evm::EnsureAddressOrigin;
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
}

impl Default for Bytes {
    fn default() -> Self {
        Self::with_len(100)
    }
}

#[derive(Clone, Debug, Encode, Decode, Eq, Ord, PartialEq, PartialOrd)]
pub struct LargeBytes(pub Bytes);

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

pub type AccountId = AccountId32;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<u32, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
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

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        EVM: pallet_evm::{Module, Config, Call, Storage, Event<T>},
        TestStorage: test_storage::{Module, Call, Storage},
    }
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ();
    type Balance = u64;
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ();
    type AccountStore = System;
    type WeightInfo = ();
}

/// Ensure that the address is truncated hash of the origin. Only works if the account id is
/// `AccountId32`.
pub struct EnsureAddressTruncated;

impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressTruncated
where
    OuterOrigin: Into<Result<RawOrigin<AccountId32>, OuterOrigin>> + From<RawOrigin<AccountId32>>,
{
    type Success = AccountId32;

    fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId32, OuterOrigin> {
        origin.into().and_then(|o| match o {
            RawOrigin::Signed(who) if AsRef::<[u8; 32]>::as_ref(&who)[0..20] == address[0..20] => {
                Ok(who)
            }
            r => Err(OuterOrigin::from(r)),
        })
    }
}

parameter_types! {
    pub const ByteReadWeight: Weight = 10;
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = ();
    type GasWeightMapping = ();
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping;
    type CallOrigin = pallet_evm::EnsureAddressRoot<H160>;
    type WithdrawOrigin = pallet_evm::EnsureAddressNever<H160>;
    type AddressMapping = pallet_evm::IdentityAddressMapping;
    type Currency = Balances;
    type Event = ();
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type ByteReadWeight = ByteReadWeight;
    type Precompiles = ();
    type ChainId = ();
    type BlockGasLimit = ();
    type OnChargeTransaction = ();
    type FindAuthor = ();
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
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
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

crate::impl_pallet_storage_metadata_provider!(
    for Runtime:
        "TestStorage" => TestStorage
);
