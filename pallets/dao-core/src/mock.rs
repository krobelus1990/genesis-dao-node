use crate as pallet_dao_core;
use frame_support::traits::{
	AsEnsureOriginWithArg, ConstU8, ConstU16, ConstU32, ConstU64, ConstU128
};
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;

use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup}
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;


pub(crate) type Balance = u128;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		DaoCore: pallet_dao_core,
		Balances: pallet_balances,
		Assets: pallet_dao_assets,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	// we're not really using this, as reservation is via DAO, but whatever
	pub const AssetDeposit: Balance = 1;
	pub const ApprovalDeposit: Balance = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1;
	pub const MetadataDepositPerByte: Balance = 1;
}

impl pallet_dao_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type RemoveItemsLimit = ConstU32<1000>;
	type StringLimit = AssetsStringLimit;
	type Extra = ();
	type WeightInfo = ();
}

impl pallet_dao_core::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MinLength = ConstU32<3>;
	type MaxLength = ConstU32<16>;
	type Currency = Balances;
	type DaoDeposit = ConstU128<10>;
    type TokenUnits = ConstU8<9>;
	type AssetId = u32;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {

	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 100),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
