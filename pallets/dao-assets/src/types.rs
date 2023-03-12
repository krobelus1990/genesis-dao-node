//! Various basic types for use in the assets pallet.

use super::*;
use frame_support::{
	pallet_prelude::*,
	traits::{fungible, tokens::BalanceConversion},
};
use sp_runtime::{traits::Convert, FixedPointNumber, FixedPointOperand, FixedU128};

// Type alias for `frame_system`'s account id.
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
// This pallet's asset id and balance type.
type AssetIdOf<T> = <T as Config>::AssetId;
pub type AssetBalanceOf<T> = <T as Config>::Balance;
// Generic fungible balance type.
pub type BalanceOf<F, T> = <F as fungible::Inspect<AccountIdOf<T>>>::Balance;
// The deposit balance type
pub type DepositBalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
// The account data for an asset
pub type AssetAccountOf<T> = AssetAccount<AssetBalanceOf<T>, DepositBalanceOf<T>>;
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
	/// If `true`, then any account with this asset is given a provider reference. Otherwise, it
	/// requires a consumer reference.
	pub(super) is_sufficient: bool,
	/// The total number of accounts.
	pub(super) accounts: u32,
	/// The total number of accounts for which we have placed a self-sufficient reference.
	pub(super) sufficients: u32,
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
pub enum ExistenceReason<Balance> {
	#[codec(index = 0)]
	Consumer,
	#[codec(index = 1)]
	Sufficient,
	#[codec(index = 2)]
	DepositHeld(Balance),
	#[codec(index = 3)]
	DepositRefunded,
}

impl<Balance> ExistenceReason<Balance> {
	pub(crate) fn take_deposit(&mut self) -> Option<Balance> {
		if !matches!(self, ExistenceReason::DepositHeld(_)) {
			return None
		}
		if let ExistenceReason::DepositHeld(deposit) =
			sp_std::mem::replace(self, ExistenceReason::DepositRefunded)
		{
			Some(deposit)
		} else {
			None
		}
	}
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetAccount<Balance, DepositBalance> {
	/// Free balance.
	pub(super) balance: Balance,
	/// Reserved balance.
	pub(super) reserved: Balance,
	/// The reason for the existence of the account.
	pub(super) reason: ExistenceReason<DepositBalance>,
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

/// Possible errors when converting between external and asset balances.
#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode)]
pub enum ConversionError {
	/// The external minimum balance must not be zero.
	MinBalanceZero,
	/// The asset is not present in storage.
	AssetMissing,
	/// The asset is not sufficient and thus does not have a reliable `min_balance` so it cannot be
	/// converted.
	AssetNotSufficient,
}

/// Converts a balance value into an asset balance based on the ratio between the fungible's
/// minimum balance and the minimum asset balance.
pub struct BalanceToAssetBalance<F, T, CON>(PhantomData<(F, T, CON)>);
impl<F, T, CON> BalanceConversion<BalanceOf<F, T>, AssetIdOf<T>, AssetBalanceOf<T>>
	for BalanceToAssetBalance<F, T, CON>
where
	F: fungible::Inspect<AccountIdOf<T>>,
	T: Config,
	CON: Convert<BalanceOf<F, T>, AssetBalanceOf<T>>,
	BalanceOf<F, T>: FixedPointOperand + Zero,
	AssetBalanceOf<T>: FixedPointOperand + Zero,
{
	type Error = ConversionError;

	/// Convert the given balance value into an asset balance based on the ratio between the
	/// fungible's minimum balance and the minimum asset balance.
	///
	/// Will return `Err` if the asset is not found, not sufficient or the fungible's minimum
	/// balance is zero.
	fn to_asset_balance(
		balance: BalanceOf<F, T>,
		asset_id: AssetIdOf<T>,
	) -> Result<AssetBalanceOf<T>, ConversionError> {
		let asset = Asset::<T>::get(asset_id).ok_or(ConversionError::AssetMissing)?;
		// only sufficient assets have a min balance with reliable value
		ensure!(asset.is_sufficient, ConversionError::AssetNotSufficient);
		let min_balance = CON::convert(F::minimum_balance());
		// make sure we don't divide by zero
		ensure!(!min_balance.is_zero(), ConversionError::MinBalanceZero);
		let balance = CON::convert(balance);
		// balance * asset.min_balance / min_balance
		Ok(FixedU128::saturating_from_rational(asset.min_balance, min_balance)
			.saturating_mul_int(balance))
	}
}
