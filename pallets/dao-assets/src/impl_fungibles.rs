//! Implementations for fungibles trait.

use frame_support::{
	defensive,
	sp_runtime::{traits::Zero, DispatchError, DispatchResult},
	storage::KeyPrefixIterator,
	traits::tokens::{
		fungibles, DepositConsequence, Fortitude, Precision, Preservation, Provenance,
		WithdrawConsequence,
	},
};
use sp_std::vec::Vec;

use crate::{Approvals, Asset, Config, DebitFlags, Metadata, Pallet, SystemConfig};

impl<T: Config> fungibles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	type AssetId = T::AssetId;
	type Balance = T::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		Asset::<T>::get(asset).map(|x| x.supply).unwrap_or_else(Zero::zero)
	}

	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		Asset::<T>::get(asset).map(|x| x.min_balance).unwrap_or_else(Zero::zero)
	}

	fn total_balance(asset: Self::AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
		Pallet::<T>::total_balance(asset, who)
	}

	fn balance(asset: Self::AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
		Pallet::<T>::balance(asset, who)
	}

	fn reducible_balance(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		preservation: Preservation,
		_force: Fortitude,
	) -> Self::Balance {
		Pallet::<T>::reducible_balance(asset, who, preservation != Preservation::Expendable)
			.unwrap_or(Zero::zero())
	}

	fn can_deposit(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> DepositConsequence {
		Pallet::<T>::can_increase(asset, who, amount, provenance == Provenance::Minted)
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		Pallet::<T>::can_decrease(asset, who, amount, false)
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		Asset::<T>::contains_key(asset)
	}
}

impl<T: Config> fungibles::Balanced<T::AccountId> for Pallet<T> {
	type OnDropCredit = fungibles::DecreaseIssuance<T::AccountId, Self>;
	type OnDropDebt = fungibles::IncreaseIssuance<T::AccountId, Self>;
}

impl<T: Config> fungibles::Unbalanced<T::AccountId> for Pallet<T> {
	fn handle_raw_dust(_: Self::AssetId, _: Self::Balance) {}
	fn handle_dust(_dust: fungibles::Dust<T::AccountId, Self>) {
		defensive!("`decrease_balance` and `increase_balance` have non-default impls; nothing else calls this; qed");
	}

	fn write_balance(
		_asset: Self::AssetId,
		_who: &T::AccountId,
		_amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		defensive!("write_balance is not used if other functions are impl'd");
		Err(DispatchError::Unavailable)
	}

	fn set_total_issuance(id: T::AssetId, amount: Self::Balance) {
		Asset::<T>::mutate_exists(id, |maybe_asset| {
			if let Some(ref mut asset) = maybe_asset {
				asset.supply = amount
			}
		});
	}

	fn decrease_balance(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		_force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		let f = DebitFlags {
			keep_alive: preservation != Preservation::Expendable,
			best_effort: precision == Precision::BestEffort,
		};
		Self::decrease_balance(asset, who, amount, f, |_, _| Ok(()))
	}

	fn increase_balance(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
		_precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		Self::increase_balance(asset, who, amount, |_| Ok(()))?;
		Ok(amount)
	}
}

impl<T: Config> fungibles::metadata::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn name(asset: T::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).name.to_vec()
	}

	fn symbol(asset: T::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).symbol.to_vec()
	}

	fn decimals(asset: T::AssetId) -> u8 {
		Metadata::<T>::get(asset).decimals
	}
}

impl<T: Config> fungibles::metadata::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn set(
		asset: T::AssetId,
		from: &<T as SystemConfig>::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		Self::do_set_metadata(asset, from, name, symbol, decimals)
	}
}

impl<T: Config> fungibles::approvals::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	// Check the amount approved to be spent by an owner to a delegate
	fn allowance(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
	) -> T::Balance {
		Approvals::<T>::get((asset, &owner, &delegate))
			.map(|x| x.amount)
			.unwrap_or_else(Zero::zero)
	}
}

impl<T: Config> fungibles::approvals::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn approve(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::do_approve_transfer(asset, owner, delegate, amount)
	}

	// Aprove spending tokens from a given account
	fn transfer_from(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
		dest: &<T as SystemConfig>::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::do_transfer_approved(asset, owner, delegate, dest, amount)
	}
}

impl<T: Config> fungibles::InspectEnumerable<T::AccountId> for Pallet<T> {
	type AssetsIterator = KeyPrefixIterator<<T as Config>::AssetId>;

	/// Returns an iterator of the assets in existence.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn asset_ids() -> Self::AssetsIterator {
		Asset::<T>::iter_keys()
	}
}
