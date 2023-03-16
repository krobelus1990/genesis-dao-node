//! Functions for the Assets pallet.

use super::*;
use frame_support::{traits::Get, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::borrow::Borrow;

#[must_use]
pub(super) enum DeadConsequence {
	Remove,
	Keep,
}

use DeadConsequence::*;

// The main implementation block for the module.
impl<T: Config> Pallet<T> {
	// Public immutables

	/// Get the asset `id` free balance of `who`, or zero if the asset-account doesn't exist.
	pub fn balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_balance(id, who).unwrap_or_default()
	}

	/// Get the asset `id` total (including reserved) balance of `who`,
	/// or zero if the asset-account doesn't exist.
	pub fn total_balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_total_balance(id, who).unwrap_or_default()
	}

	/// Get the asset `id` reserved balance of `who`, or zero if the asset-account doesn't exist.
	pub fn reserved(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_reserved(id, who).unwrap_or_default()
	}

	/// Get the asset `id` free balance of `who` if the asset-account exists.
	pub fn maybe_balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.balance)
	}

	/// Get the asset `id` total (including reserved) balance of `who` if the asset-account exists.
	pub fn maybe_total_balance(
		id: T::AssetId,
		who: impl Borrow<T::AccountId>,
	) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.balance + a.reserved)
	}

	/// Get the asset `id` reserved balance of `who` if the asset-account exists.
	pub fn maybe_reserved(id: T::AssetId, who: impl Borrow<T::AccountId>) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.reserved)
	}

	/// Get the total supply of an asset `id`.
	pub fn total_supply(id: T::AssetId) -> T::Balance {
		Self::maybe_total_supply(id).unwrap_or_default()
	}

	/// Get the total supply of an asset `id` if the asset exists.
	pub fn maybe_total_supply(id: T::AssetId) -> Option<T::Balance> {
		Asset::<T>::get(id).map(|x| x.supply)
	}

	/// Get the total historical supply of an asset `id` at a certain `block`.
	/// Result may be None, if the age of the requested block is at or beyond
	/// the HistoryHorizon and history has been removed.
	pub fn total_historical_supply(id: T::AssetId, block: BlockNumberFor<T>) -> Option<T::Balance> {
		let default = || {
			let current_block = frame_system::Pallet::<T>::block_number();
			if current_block - block < T::HistoryHorizon::get().into() {
				Some(Zero::zero())
			} else {
				None
			}
		};

		SupplyHistory::<T>::get(id).map_or_else(default, |history| {
			history.range(..=block).next_back().map(|item| *item.1).or_else(default)
		})
	}

	pub(super) fn update_supply_history(id: T::AssetId, supply: T::Balance) {
		let mut history = SupplyHistory::<T>::get(id).unwrap_or_default();
		let current_block = frame_system::Pallet::<T>::block_number();

		// the oldest block for which we need to be able to retrieve history
		let inner_horizon_block =
			current_block.saturating_sub((T::HistoryHorizon::get() - 1).into());

		// if there is enough history, find block that has history for inner_horizon_block
		if let Some(block) = history.range(..=inner_horizon_block).next_back().map(|i| *i.0) {
			// and remove everything older than that block
			history = BoundedBTreeMap::try_from(history.into_inner().split_off(&block)).unwrap();
		}

		// insert new history item
		history.try_insert(current_block, supply).expect("Enough space has been freed");

		// record new history
		SupplyHistory::<T>::insert(id, history);
	}

	pub(super) fn new_account(
		who: &T::AccountId,
		d: &mut AssetDetailsOf<T>,
		maybe_deposit: Option<DepositBalanceOf<T>>,
	) -> Result<ExistenceReason<DepositBalanceOf<T>>, DispatchError> {
		let accounts = d.accounts.checked_add(1).ok_or(ArithmeticError::Overflow)?;
		let reason = if let Some(deposit) = maybe_deposit {
			ExistenceReason::DepositHeld(deposit)
		} else if d.is_sufficient {
			frame_system::Pallet::<T>::inc_sufficients(who);
			d.sufficients += 1;
			ExistenceReason::Sufficient
		} else {
			frame_system::Pallet::<T>::inc_consumers(who).map_err(|_| Error::<T>::NoProvider)?;
			ExistenceReason::Consumer
		};
		d.accounts = accounts;
		Ok(reason)
	}

	pub(super) fn dead_account(
		who: &T::AccountId,
		d: &mut AssetDetailsOf<T>,
		reason: &ExistenceReason<DepositBalanceOf<T>>,
		force: bool,
	) -> DeadConsequence {
		match *reason {
			ExistenceReason::Consumer => frame_system::Pallet::<T>::dec_consumers(who),
			ExistenceReason::Sufficient => {
				d.sufficients = d.sufficients.saturating_sub(1);
				frame_system::Pallet::<T>::dec_sufficients(who);
			},
			ExistenceReason::DepositRefunded => {},
			ExistenceReason::DepositHeld(_) if !force => return Keep,
			ExistenceReason::DepositHeld(_) => {},
		}
		d.accounts = d.accounts.saturating_sub(1);
		Remove
	}

	/// Returns `true` when the balance of `account` can be increased by `amount`.
	///
	/// - `id`: The id of the asset that should be increased.
	/// - `who`: The account of which the balance should be increased.
	/// - `amount`: The amount by which the balance should be increased.
	/// - `increase_supply`: Will the supply of the asset be increased by `amount` at the same time
	///   as crediting the `account`.
	pub(super) fn can_increase(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		increase_supply: bool,
	) -> DepositConsequence {
		let details = match Asset::<T>::get(id) {
			Some(details) => details,
			None => return DepositConsequence::UnknownAsset,
		};
		if increase_supply && details.supply.checked_add(&amount).is_none() {
			return DepositConsequence::Overflow
		}
		if let Some(balance) = Self::maybe_balance(id, who) {
			if balance.checked_add(&amount).is_none() {
				return DepositConsequence::Overflow
			}
		} else {
			if amount < details.min_balance {
				return DepositConsequence::BelowMinimum
			}
			if !details.is_sufficient && !frame_system::Pallet::<T>::can_inc_consumer(who) {
				return DepositConsequence::CannotCreate
			}
			if details.is_sufficient && details.sufficients.checked_add(1).is_none() {
				return DepositConsequence::Overflow
			}
		}

		DepositConsequence::Success
	}

	/// Return the consequence of a withdraw.
	pub(super) fn can_decrease(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		keep_alive: bool,
	) -> WithdrawConsequence<T::Balance> {
		use WithdrawConsequence::*;
		let details = match Asset::<T>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};
		if details.supply.checked_sub(&amount).is_none() {
			return Underflow
		}
		if amount.is_zero() {
			return Success
		}
		let account = match Account::<T>::get(id, who) {
			Some(a) => a,
			None => return NoFunds,
		};
		if let Some(rest) = account.balance.checked_sub(&amount) {
			let is_provider = false;
			let is_required = is_provider && !frame_system::Pallet::<T>::can_dec_provider(who);
			let must_keep_alive = keep_alive || is_required;

			if rest < details.min_balance {
				if must_keep_alive {
					WouldDie
				} else {
					ReducedToZero(rest)
				}
			} else {
				Success
			}
		} else {
			NoFunds
		}
	}

	// Maximum `amount` that can be passed into `can_withdraw` to result in a `WithdrawConsequence`
	// of `Success`.
	pub(super) fn reducible_balance(
		id: T::AssetId,
		who: &T::AccountId,
		keep_alive: bool,
	) -> Result<T::Balance, DispatchError> {
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let account = Account::<T>::get(id, who).ok_or(Error::<T>::NoAccount)?;
		let amount = {
			let is_provider = false;
			let is_required = is_provider && !frame_system::Pallet::<T>::can_dec_provider(who);
			if keep_alive || is_required {
				// We want to keep the account around.
				account.balance.saturating_sub(details.min_balance)
			} else {
				// Don't care if the account dies
				account.balance
			}
		};
		Ok(amount.min(details.supply))
	}

	/// Make preparatory checks for debiting some funds from an account. Flags indicate requirements
	/// of the debit.
	///
	/// - `amount`: The amount desired to be debited. The actual amount returned for debit may be
	///   less (in the case of `best_effort` being `true`) or greater by up to the minimum balance
	///   less one.
	/// - `keep_alive`: Require that `target` must stay alive.
	/// - `best_effort`: The debit amount may be less than `amount`.
	///
	/// On success, the amount which should be debited (this will always be at least `amount` unless
	/// `best_effort` is `true`).
	///
	/// If no valid debit can be made then return an `Err`.
	pub(super) fn prep_debit(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
	) -> Result<T::Balance, DispatchError> {
		let actual = Self::reducible_balance(id, target, f.keep_alive)?.min(amount);
		ensure!(f.best_effort || actual >= amount, Error::<T>::BalanceLow);

		let conseq = Self::can_decrease(id, target, actual, f.keep_alive);
		let actual = match conseq.into_result() {
			Ok(dust) => actual.saturating_add(dust), //< guaranteed by reducible_balance
			Err(e) => {
				debug_assert!(false, "passed from reducible_balance; qed");
				return Err(e)
			},
		};

		Ok(actual)
	}

	/// Make preparatory checks for crediting some funds from an account. Flags indicate
	/// requirements of the credit.
	///
	/// - `amount`: The amount desired to be credited.
	/// - `debit`: The amount by which some other account has been debited. If this is greater than
	///   `amount`, then the `burn_dust` parameter takes effect.
	/// - `burn_dust`: Indicates that in the case of debit being greater than amount, the additional
	///   (dust) value should be burned, rather than credited.
	///
	/// On success, the amount which should be credited (this will always be at least `amount`)
	/// together with an optional value indicating the value which should be burned. The latter
	/// will always be `None` as long as `burn_dust` is `false` or `debit` is no greater than
	/// `amount`.
	///
	/// If no valid credit can be made then return an `Err`.
	pub(super) fn prep_credit(
		id: T::AssetId,
		dest: &T::AccountId,
		amount: T::Balance,
		debit: T::Balance,
		burn_dust: bool,
	) -> Result<(T::Balance, Option<T::Balance>), DispatchError> {
		let (credit, maybe_burn) = match (burn_dust, debit.checked_sub(&amount)) {
			(true, Some(dust)) => (amount, Some(dust)),
			_ => (debit, None),
		};
		Self::can_increase(id, dest, credit, false).into_result()?;
		Ok((credit, maybe_burn))
	}

	/// Creates a account for `who` to hold asset `id` with a zero balance and takes a deposit.
	pub(super) fn do_touch(id: T::AssetId, who: T::AccountId) -> DispatchResult {
		ensure!(!Account::<T>::contains_key(id, &who), Error::<T>::AlreadyExists);
		let deposit = T::AssetAccountDeposit::get();
		let mut details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		let reason = Self::new_account(&who, &mut details, Some(deposit))?;
		T::Currency::reserve(&who, deposit)?;
		Asset::<T>::insert(id, details);
		Account::<T>::insert(
			id,
			&who,
			AssetAccountOf::<T> { balance: Zero::zero(), reserved: Zero::zero(), reason },
		);
		Ok(())
	}

	/// Returns a deposit, destroying an asset-account.
	pub(super) fn do_refund(id: T::AssetId, who: T::AccountId) -> DispatchResult {
		let mut account = Account::<T>::get(id, &who).ok_or(Error::<T>::NoDeposit)?;
		let deposit = account.reason.take_deposit().ok_or(Error::<T>::NoDeposit)?;
		let mut details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		ensure!(account.balance.is_zero(), Error::<T>::WouldBurn);

		T::Currency::unreserve(&who, deposit);

		details.accounts.saturating_dec();
		Account::<T>::remove(id, &who);

		Asset::<T>::insert(id, details);
		Ok(())
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub(super) fn do_mint(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		maybe_check_issuer: Option<T::AccountId>,
	) -> DispatchResult {
		Self::increase_balance(id, beneficiary, amount, |details| -> DispatchResult {
			if let Some(check_issuer) = maybe_check_issuer {
				ensure!(check_issuer == details.issuer, Error::<T>::NoPermission);
			}
			debug_assert!(
				T::Balance::max_value() - details.supply >= amount,
				"checked in prep; qed"
			);
			details.supply.saturating_accrue(amount);

			Self::update_supply_history(id, details.supply);

			Ok(())
		})?;
		Self::deposit_event(Event::Issued {
			asset_id: id,
			owner: beneficiary.clone(),
			total_supply: amount,
		});
		Ok(())
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_mint` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub(super) fn increase_balance(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		check: impl FnOnce(&mut AssetDetailsOf<T>) -> DispatchResult,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}

		Self::can_increase(id, beneficiary, amount, true).into_result()?;
		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);
			check(details)?;

			Account::<T>::try_mutate(id, beneficiary, |maybe_account| -> DispatchResult {
				match maybe_account {
					Some(ref mut account) => {
						account.balance.saturating_accrue(amount);
					},
					maybe_account @ None => {
						// Note this should never fail as it's already checked by `can_increase`.
						ensure!(amount >= details.min_balance, TokenError::BelowMinimum);
						*maybe_account = Some(AssetAccountOf::<T> {
							balance: amount,
							reserved: Zero::zero(),
							reason: Self::new_account(beneficiary, details, None)?,
						});
					},
				}
				Ok(())
			})?;
			Ok(())
		})?;
		Ok(())
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	pub(super) fn do_burn(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		maybe_check_admin: Option<T::AccountId>,
		f: DebitFlags,
	) -> Result<T::Balance, DispatchError> {
		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let actual = Self::decrease_balance(id, target, amount, f, |actual, details| {
			// Check admin rights.
			if let Some(check_admin) = maybe_check_admin {
				ensure!(check_admin == details.admin, Error::<T>::NoPermission);
			}

			debug_assert!(details.supply >= actual, "checked in prep; qed");
			details.supply.saturating_reduce(actual);

			Self::update_supply_history(id, details.supply);

			Ok(())
		})?;
		Self::deposit_event(Event::Burned { asset_id: id, owner: target.clone(), balance: actual });
		Ok(actual)
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_burn` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	pub(super) fn decrease_balance(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
		check: impl FnOnce(T::Balance, &mut AssetDetailsOf<T>) -> DispatchResult,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let actual = Self::prep_debit(id, target, amount, f)?;
		let mut target_died: Option<DeadConsequence> = None;

		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			check(actual, details)?;

			Account::<T>::try_mutate(id, target, |maybe_account| -> DispatchResult {
				let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;
				debug_assert!(account.balance >= actual, "checked in prep; qed");

				// Make the debit.
				account.balance = account.balance.saturating_sub(actual);
				if account.balance < details.min_balance {
					debug_assert!(account.balance.is_zero(), "checked in prep; qed");
					target_died = Some(Self::dead_account(target, details, &account.reason, false));
					if let Some(Remove) = target_died {
						return Ok(())
					}
				};
				*maybe_account = Some(account);
				Ok(())
			})?;

			Ok(())
		})?;

		Ok(actual)
	}

	/// Reserves some `amount` of asset `id` balance of `target`.
	pub fn do_reserve(
		id: T::AssetId,
		target: impl Borrow<T::AccountId>,
		amount: T::Balance,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let f = DebitFlags { keep_alive: true, best_effort: false };

		let actual = Self::prep_debit(id, target.borrow(), amount, f)?;

		Account::<T>::try_mutate(id, target.borrow(), |maybe_account| -> DispatchResult {
			let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;
			debug_assert!(account.balance >= actual, "checked in prep; qed");

			// Make the reservation.
			account.balance = account.balance.saturating_sub(actual);
			account.reserved = account.reserved.saturating_add(actual);
			*maybe_account = Some(account);
			Ok(())
		})?;

		Ok(actual)
	}

	/// Unreserves some `amount` of asset `id` balance of `target`.
	/// If `amount` is greater than reserved balance, then the whole reserved balance is unreserved.
	pub fn do_unreserve(
		id: T::AssetId,
		target: impl Borrow<T::AccountId>,
		mut amount: T::Balance,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		// check asset is live
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		Account::<T>::try_mutate(id, target.borrow(), |maybe_account| -> DispatchResult {
			let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;

			// Unreserve the minimum of amount and reserved balance
			amount = amount.min(account.reserved);
			account.balance = account.balance.saturating_add(amount);
			account.reserved = account.reserved.saturating_sub(amount);
			*maybe_account = Some(account);
			Ok(())
		})?;

		Ok(amount)
	}

	/// Reduces the asset `id` balance of `source` by some `amount` and increases the balance of
	/// `dest` by (similar) amount.
	///
	/// Returns the actual amount placed into `dest`. Exact semantics are determined by the flags
	/// `f`.
	///
	/// Will fail if the amount transferred is so small that it cannot create the destination due
	/// to minimum balance requirements.
	pub(super) fn do_transfer(
		id: T::AssetId,
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: T::Balance,
		maybe_need_admin: Option<T::AccountId>,
		f: TransferFlags,
	) -> Result<T::Balance, DispatchError> {
		// Early exit if no-op.
		if amount.is_zero() {
			return Ok(amount)
		}
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		// Figure out the debit and credit, together with side-effects.
		let debit = Self::prep_debit(id, source, amount, f.into())?;
		let (credit, maybe_burn) = Self::prep_credit(id, dest, amount, debit, f.burn_dust)?;

		let mut source_account = Account::<T>::get(id, source).ok_or(Error::<T>::NoAccount)?;

		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Check admin rights.
			if let Some(need_admin) = maybe_need_admin {
				ensure!(need_admin == details.admin, Error::<T>::NoPermission);
			}

			// Skip if source == dest
			if source == dest {
				return Ok(())
			}

			// Burn any dust if needed.
			if let Some(burn) = maybe_burn {
				// Debit dust from supply; this will not saturate since it's already checked in
				// prep.
				debug_assert!(details.supply >= burn, "checked in prep; qed");
				details.supply = details.supply.saturating_sub(burn);
			}

			// Debit balance from source; this will not saturate since it's already checked in prep.
			debug_assert!(source_account.balance >= debit, "checked in prep; qed");
			source_account.balance = source_account.balance.saturating_sub(debit);

			Account::<T>::try_mutate(id, dest, |maybe_account| -> DispatchResult {
				match maybe_account {
					Some(ref mut account) => {
						// Calculate new balance; this will not saturate since it's already checked
						// in prep.
						debug_assert!(
							account.balance.checked_add(&credit).is_some(),
							"checked in prep; qed"
						);
						account.balance.saturating_accrue(credit);
					},
					maybe_account @ None => {
						*maybe_account = Some(AssetAccountOf::<T> {
							balance: credit,
							reserved: Zero::zero(),
							reason: Self::new_account(dest, details, None)?,
						});
					},
				}
				Ok(())
			})?;

			// Remove source account if it's now dead.
			if source_account.balance < details.min_balance {
				debug_assert!(source_account.balance.is_zero(), "checked in prep; qed");
				if let Some(Remove) =
					Some(Self::dead_account(source, details, &source_account.reason, false))
				{
					Account::<T>::remove(id, source);
					return Ok(())
				}
			}
			Account::<T>::insert(id, source, &source_account);
			Ok(())
		})?;

		Self::deposit_event(Event::Transferred {
			asset_id: id,
			from: source.clone(),
			to: dest.clone(),
			amount: credit,
		});
		Ok(credit)
	}

	/// Create a new asset without taking a deposit.
	///
	/// * `id`: The `AssetId` you want the new asset to have. Must not already be in use.
	/// * `owner`: The owner, issuer, and admin of this asset upon creation.
	/// * `is_sufficient`: Whether this asset needs users to have an existential deposit to hold
	///   this asset.
	/// * `min_balance`: The minimum balance a user is allowed to have of this asset before they are
	///   considered dust and cleaned up.
	pub(super) fn do_force_create(
		id: T::AssetId,
		owner: T::AccountId,
		is_sufficient: bool,
		min_balance: T::Balance,
	) -> DispatchResult {
		ensure!(!Asset::<T>::contains_key(id), Error::<T>::InUse);
		ensure!(!min_balance.is_zero(), Error::<T>::MinBalanceZero);

		Asset::<T>::insert(
			id,
			AssetDetails {
				owner: owner.clone(),
				issuer: owner.clone(),
				admin: owner.clone(),
				supply: Zero::zero(), // no need to record a supply of zero in the SupplyHistory
				deposit: Zero::zero(),
				min_balance,
				is_sufficient,
				accounts: 0,
				sufficients: 0,
				approvals: 0,
				status: AssetStatus::Live,
			},
		);

		Self::deposit_event(Event::ForceCreated { asset_id: id, owner });
		Ok(())
	}

	/// Start the process of destroying an asset, by setting the asset status to `Destroying`, and
	/// emitting the `DestructionStarted` event.
	pub(super) fn do_start_destroy(
		id: T::AssetId,
		maybe_check_owner: Option<T::AccountId>,
	) -> DispatchResult {
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let mut details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			if let Some(check_owner) = maybe_check_owner {
				ensure!(details.owner == check_owner, Error::<T>::NoPermission);
			}
			details.status = AssetStatus::Destroying;
			SupplyHistory::<T>::remove(id);

			Self::deposit_event(Event::DestructionStarted { asset_id: id });
			Ok(())
		})
	}

	/// Destroy accounts associated with a given asset up to the max (T::RemoveItemsLimit).
	///
	/// Each call emits the `Event::AccountsDestroyed` event.
	/// Returns the number of destroyed accounts.
	pub(super) fn do_destroy_accounts(
		id: T::AssetId,
		max_items: u32,
	) -> Result<u32, DispatchError> {
		let mut dead_accounts = 0;
		let mut remaining_accounts = 0;
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Should only destroy accounts while the asset is in a destroying state
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);

			for (who, v) in Account::<T>::drain_prefix(id) {
				let _ = Self::dead_account(&who, details, &v.reason, true);
				dead_accounts += 1;
				if dead_accounts >= max_items {
					break
				}
			}
			remaining_accounts = details.accounts;
			Ok(())
		})?;

		Self::deposit_event(Event::AccountsDestroyed {
			asset_id: id,
			accounts_destroyed: dead_accounts,
			accounts_remaining: remaining_accounts,
		});
		Ok(dead_accounts)
	}

	/// Destroy approvals associated with a given asset up to the max (T::RemoveItemsLimit).
	///
	/// Each call emits the `Event::ApprovalsDestroyed` event
	/// Returns the number of destroyed approvals.
	pub(super) fn do_destroy_approvals(
		id: T::AssetId,
		max_items: u32,
	) -> Result<u32, DispatchError> {
		let mut removed_approvals = 0;
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let mut details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Should only destroy accounts while the asset is in a destroying state.
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);

			for ((owner, _), approval) in Approvals::<T>::drain_prefix((id,)) {
				T::Currency::unreserve(&owner, approval.deposit);
				removed_approvals = removed_approvals.saturating_add(1);
				details.approvals = details.approvals.saturating_sub(1);
				if removed_approvals >= max_items {
					break
				}
			}
			Self::deposit_event(Event::ApprovalsDestroyed {
				asset_id: id,
				approvals_destroyed: removed_approvals,
				approvals_remaining: details.approvals,
			});
			Ok(())
		})?;
		Ok(removed_approvals)
	}

	/// Complete destroying an asset and unreserve the deposit.
	///
	/// On success, the `Event::Destroyed` event is emitted.
	pub(super) fn do_finish_destroy(id: T::AssetId) -> DispatchResult {
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let mut details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);
			ensure!(details.accounts == 0, Error::<T>::InUse);
			ensure!(details.approvals == 0, Error::<T>::InUse);

			let metadata = Metadata::<T>::take(id);
			T::Currency::unreserve(
				&details.owner,
				details.deposit.saturating_add(metadata.deposit),
			);
			details.status = AssetStatus::Destroyed;

			Self::deposit_event(Event::Destroyed { asset_id: id });

			Ok(())
		})
	}

	/// Creates an approval from `owner` to spend `amount` of asset `id` tokens by 'delegate'
	/// while reserving `T::ApprovalDeposit` from owner
	///
	/// If an approval already exists, the new amount is added to such existing approval
	pub(super) fn do_approve_transfer(
		id: T::AssetId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		let mut d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		Approvals::<T>::try_mutate((id, &owner, &delegate), |maybe_approved| -> DispatchResult {
			let mut approved = match maybe_approved.take() {
				// an approval already exists and is being updated
				Some(a) => a,
				// a new approval is created
				None => {
					d.approvals.saturating_inc();
					Default::default()
				},
			};
			let deposit_required = T::ApprovalDeposit::get();
			if approved.deposit < deposit_required {
				T::Currency::reserve(owner, deposit_required - approved.deposit)?;
				approved.deposit = deposit_required;
			}
			approved.amount = approved.amount.saturating_add(amount);
			*maybe_approved = Some(approved);
			Ok(())
		})?;
		Asset::<T>::insert(id, d);
		Self::deposit_event(Event::ApprovedTransfer {
			asset_id: id,
			source: owner.clone(),
			delegate: delegate.clone(),
			amount,
		});

		Ok(())
	}

	/// Reduces the asset `id` balance of `owner` by some `amount` and increases the balance of
	/// `dest` by (similar) amount, checking that 'delegate' has an existing approval from `owner`
	/// to spend`amount`.
	///
	/// Will fail if `amount` is greater than the approval from `owner` to 'delegate'
	/// Will unreserve the deposit from `owner` if the entire approved `amount` is spent by
	/// 'delegate'
	pub(super) fn do_transfer_approved(
		id: T::AssetId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
		destination: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		Approvals::<T>::try_mutate_exists(
			(id, &owner, delegate),
			|maybe_approved| -> DispatchResult {
				let mut approved = maybe_approved.take().ok_or(Error::<T>::Unapproved)?;
				let remaining =
					approved.amount.checked_sub(&amount).ok_or(Error::<T>::Unapproved)?;

				let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
				Self::do_transfer(id, owner, destination, amount, None, f)?;

				if remaining.is_zero() {
					T::Currency::unreserve(owner, approved.deposit);
					Asset::<T>::mutate(id, |maybe_details| {
						if let Some(details) = maybe_details {
							details.approvals.saturating_dec();
						}
					});
				} else {
					approved.amount = remaining;
					*maybe_approved = Some(approved);
				}
				Ok(())
			},
		)?;

		Ok(())
	}

	/// Do set metadata
	pub(super) fn do_set_metadata(
		id: T::AssetId,
		from: &T::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		ensure!(from == &d.owner, Error::<T>::NoPermission);

		Metadata::<T>::try_mutate_exists(id, |metadata| {
			let old_deposit = metadata.take().map_or(Zero::zero(), |m| m.deposit);
			let new_deposit = T::MetadataDepositPerByte::get()
				.saturating_mul(((name.len() + symbol.len()) as u32).into())
				.saturating_add(T::MetadataDepositBase::get());

			if new_deposit > old_deposit {
				T::Currency::reserve(from, new_deposit - old_deposit)?;
			} else {
				T::Currency::unreserve(from, old_deposit - new_deposit);
			}

			*metadata = Some(AssetMetadata {
				deposit: new_deposit,
				name: bounded_name,
				symbol: bounded_symbol,
				decimals,
			});

			Self::deposit_event(Event::MetadataSet { asset_id: id, name, symbol, decimals });
			Ok(())
		})
	}
}
