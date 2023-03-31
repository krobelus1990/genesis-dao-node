//! Various basic types for use in the assets pallet.

use super::*;
use frame_support::{pallet_prelude::*, traits::fungible};

// Type alias for `frame_system`'s account id.
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
// This pallet's asset id and balance type.
pub type AssetBalanceOf<T> = <T as Config>::Balance;
// Generic fungible balance type.
pub type BalanceOf<F, T> = <F as fungible::Inspect<AccountIdOf<T>>>::Balance;
// The deposit balance type
pub type DepositBalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
// The account data for an asset
pub type AssetAccountOf<T> = AssetAccount<AssetBalanceOf<T>>;
pub type AssetDetailsOf<T> = AssetDetails<AssetBalanceOf<T>, AccountIdOf<T>, DepositBalanceOf<T>>;

/// AssetStatus holds the current state of the asset. It could either be Live and available for use,
/// or in a Destroying state.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum AssetStatus {
	/// The asset is active and able to be used.
	Live,
	/// The asset is currently being destroyed, and all actions are no longer permitted on the
	/// asset. Once set to `Destroying`, the asset can never transition back to a `Live` state.
	Destroying,
	/// The asset has been destroyed
	Destroyed,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<Balance, AccountId, DepositBalance> {
	/// Can change `owner`, `issuer`, and `admin` accounts.
	pub(super) owner: AccountId,
	/// Can mint tokens.
	pub(super) issuer: AccountId,
	/// Can force transfers and burn tokens from any account.
	pub(super) admin: AccountId,
	/// The total supply across all accounts.
	pub(super) supply: Balance,
	/// The balance deposited for this asset. This pays for the data stored here.
	pub(super) deposit: DepositBalance,
	/// The ED for virtual accounts.
	pub(super) min_balance: Balance,
	/// The total number of accounts.
	pub(super) accounts: u32,
	/// The total number of approvals.
	pub(super) approvals: u32,
	/// The status of the asset
	pub status: AssetStatus,
}

/// Data concerning an approval.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo)]
pub struct Approval<Balance, DepositBalance> {
	/// The amount of funds approved for the balance transfer from the owner to some delegated
	/// target.
	pub(super) amount: Balance,
	/// The amount reserved on the owner's account to hold this item in storage.
	pub(super) deposit: DepositBalance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetAccount<Balance> {
	/// Free balance.
	pub(super) balance: Balance,
	/// Reserved balance.
	pub(super) reserved: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetMetadata<DepositBalance, BoundedString> {
	/// The balance deposited for this metadata.
	///
	/// This pays for the data stored in this struct.
	pub(super) deposit: DepositBalance,
	/// The user friendly name of this asset. Limited in length by `StringLimit`.
	pub(super) name: BoundedString,
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub(super) symbol: BoundedString,
	/// The number of decimals this asset uses to represent one unit.
	pub(super) decimals: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct TransferFlags {
	/// The debited account must stay alive at the end of the operation; an error is returned if
	/// this cannot be achieved legally.
	pub(super) keep_alive: bool,
	/// Less than the amount specified needs be debited by the operation for it to be considered
	/// successful. If `false`, then the amount debited will always be at least the amount
	/// specified.
	pub(super) best_effort: bool,
	/// Any additional funds debited (due to minimum balance requirements) should be burned rather
	/// than credited to the destination account.
	pub(super) burn_dust: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct DebitFlags {
	/// The debited account must stay alive at the end of the operation; an error is returned if
	/// this cannot be achieved legally.
	pub(super) keep_alive: bool,
	/// Less than the amount specified needs be debited by the operation for it to be considered
	/// successful. If `false`, then the amount debited will always be at least the amount
	/// specified.
	pub(super) best_effort: bool,
}

impl From<TransferFlags> for DebitFlags {
	fn from(f: TransferFlags) -> Self {
		Self { keep_alive: f.keep_alive, best_effort: f.best_effort }
	}
}
