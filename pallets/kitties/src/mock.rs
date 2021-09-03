use crate as pallet_kitties;
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use frame_support_test::TestRandomness;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		KittiesModule: pallet_kitties::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const ProofMaxLength: usize = 32;
	pub const ProofMinLength: usize = 4;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::AllowAll;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

// Refer https://github.com/paritytech/substrate/blob/49a4103f4bfef55be20a5c6d26e18ff3003c3353/frame/treasury/src/tests.rs#L81
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
    pub const MaxReserves: u32 = 50;

    pub const BalanceToReserve: u64 = 20;
    pub const KittiesPalletId: PalletId = PalletId(*b"py/kitty");
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
    type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_kitties::Config for Test {
	type Event = Event;
    type Currency = pallet_balances::Pallet<Test>;
    type KittyIndex = u32;
    type Randomness = TestRandomness<Self>;
    type BalanceToReserve = BalanceToReserve;
    type PalletId = KittiesPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Refer https://github.com/paritytech/substrate/blob/8a88403d3e0791bf2a3a45cfadc202fc85065b7b/frame/lottery/src/mock.rs#L123
    let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
