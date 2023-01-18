// This recursion limit is needed because we have too many benchmarks and benchmarking will fail if
// we add more without this limit.
#![recursion_limit = "1024"]
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

mod extra_mutator;
pub use extra_mutator::*;
mod functions;
mod impl_fungibles;
mod impl_stored_map;
mod types;

pub use types::*;

use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
	},
	ArithmeticError, TokenError,
};
use sp_std::{borrow::Borrow};

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	storage::KeyPrefixIterator,
	traits::{
		tokens::{fungibles, DepositConsequence, WithdrawConsequence},
		BalanceStatus::Reserved,
		Currency, EnsureOriginWithArg, ReservableCurrency, StoredMap,
	},
};
use frame_system::Config as SystemConfig;

pub use weights::WeightInfo;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<AssetIdParameter> {
		fn create_asset_id_parameter(id: u32) -> AssetIdParameter;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<AssetIdParameter: From<u32>> BenchmarkHelper<AssetIdParameter> for () {
		fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
			id.into()
		}
	}

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The units in which we record balances.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Max number of items to destroy per `destroy_accounts` and `destroy_approvals` call.
		///
		/// Must be configured to result in a weight that makes each call fit in a block.
		#[pallet::constant]
		type RemoveItemsLimit: Get<u32>;

		/// Identifier for the class of asset.
		type AssetId: Member + Parameter + Copy + MaybeSerializeDeserialize + MaxEncodedLen;

		/// Wrapper around `Self::AssetId` to use in dispatchable call signatures. Allows the use
		/// of compact encoding in instances of the pallet, which will prevent breaking changes
		/// resulting from the removal of `HasCompact` from `Self::AssetId`.
		///
		/// This type includes the `From<Self::AssetId>` bound, since tightly coupled pallets may
		/// want to convert an `AssetId` into a parameter for calling dispatchable functions
		/// directly.
		type AssetIdParameter: Parameter
			+ Copy
			+ From<Self::AssetId>
			+ Into<Self::AssetId>
			+ MaxEncodedLen;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Standard asset class creation is only allowed if the origin attempting it and the
		/// asset class are in this set.
		type CreateOrigin: EnsureOriginWithArg<
			Self::RuntimeOrigin,
			Self::AssetId,
			Success = Self::AccountId,
		>;

		/// The origin which may forcibly create or destroy an asset or otherwise alter privileged
		/// attributes.
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The basic amount of funds that must be reserved for an asset.
		#[pallet::constant]
		type AssetDeposit: Get<DepositBalanceOf<Self, I>>;

		/// The amount of funds that must be reserved for a non-provider asset account to be
		/// maintained.
		#[pallet::constant]
		type AssetAccountDeposit: Get<DepositBalanceOf<Self, I>>;

		/// The basic amount of funds that must be reserved when adding metadata to your asset.
		#[pallet::constant]
		type MetadataDepositBase: Get<DepositBalanceOf<Self, I>>;

		/// The additional funds that must be reserved for the number of bytes you store in your
		/// metadata.
		#[pallet::constant]
		type MetadataDepositPerByte: Get<DepositBalanceOf<Self, I>>;

		/// The amount of funds that must be reserved when creating a new approval.
		#[pallet::constant]
		type ApprovalDeposit: Get<DepositBalanceOf<Self, I>>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// A hook to allow a per-asset, per-account minimum balance to be enforced. This must be
		/// respected in all permissionless operations.
		type Freezer: FrozenBalance<Self::AssetId, Self::AccountId, Self::Balance>;

		/// Additional data to be stored with an account's asset balance.
		type Extra: Member + Parameter + Default + MaxEncodedLen;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Helper trait for benchmarks.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::AssetIdParameter>;
	}

	#[pallet::storage]
	/// Details of an asset.
	pub(super) type Asset<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
	>;

	#[pallet::storage]
	/// The holdings of a specific account for a specific asset.
	pub(super) type Account<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		Blake2_128Concat,
		T::AccountId,
		AssetAccountOf<T, I>,
	>;

	#[pallet::storage]
	/// Approved balance transfers. First balance is the amount approved for transfer. Second
	/// is the amount of `T::Currency` reserved for storing this.
	/// First key is the asset ID, second key is the owner and third key is the delegate.
	pub(super) type Approvals<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AssetId>,
			NMapKey<Blake2_128Concat, T::AccountId>, // owner
			NMapKey<Blake2_128Concat, T::AccountId>, // delegate
		),
		Approval<T::Balance, DepositBalanceOf<T, I>>,
	>;

	#[pallet::storage]
	/// Metadata of an asset.
	pub(super) type Metadata<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetMetadata<DepositBalanceOf<T, I>, BoundedVec<u8, T::StringLimit>>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Genesis assets: id, owner, is_sufficient, min_balance
		pub assets: Vec<(T::AssetId, T::AccountId, bool, T::Balance)>,
		/// Genesis metadata: id, name, symbol, decimals
		pub metadata: Vec<(T::AssetId, Vec<u8>, Vec<u8>, u8)>,
		/// Genesis accounts: id, account_id, balance
		pub accounts: Vec<(T::AssetId, T::AccountId, T::Balance)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				assets: Default::default(),
				metadata: Default::default(),
				accounts: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			for (id, owner, is_sufficient, min_balance) in &self.assets {
				assert!(!Asset::<T, I>::contains_key(id), "Asset id already in use");
				assert!(!min_balance.is_zero(), "Min balance should not be zero");
				Asset::<T, I>::insert(
					id,
					AssetDetails {
						owner: owner.clone(),
						issuer: owner.clone(),
						admin: owner.clone(),
						freezer: owner.clone(),
						supply: Zero::zero(),
						deposit: Zero::zero(),
						min_balance: *min_balance,
						is_sufficient: *is_sufficient,
						accounts: 0,
						sufficients: 0,
						approvals: 0,
						status: AssetStatus::Live,
					},
				);
			}

			for (id, name, symbol, decimals) in &self.metadata {
				assert!(Asset::<T, I>::contains_key(id), "Asset does not exist");

				let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().expect("asset name is too long");
				let bounded_symbol: BoundedVec<u8, T::StringLimit> =
					symbol.clone().try_into().expect("asset symbol is too long");

				let metadata = AssetMetadata {
					deposit: Zero::zero(),
					name: bounded_name,
					symbol: bounded_symbol,
					decimals: *decimals,
					is_frozen: false,
				};
				Metadata::<T, I>::insert(id, metadata);
			}

			for (id, account_id, amount) in &self.accounts {
				let result = <Pallet<T, I>>::increase_balance(
					*id,
					account_id,
					*amount,
					|details| -> DispatchResult {
						debug_assert!(
							T::Balance::max_value() - details.supply >= *amount,
							"checked in prep; qed"
						);
						details.supply = details.supply.saturating_add(*amount);
						Ok(())
					},
				);
				assert!(result.is_ok());
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Some asset class was created.
		Created { asset_id: T::AssetId, creator: T::AccountId, owner: T::AccountId },
		/// Some assets were issued.
		Issued { asset_id: T::AssetId, owner: T::AccountId, total_supply: T::Balance },
		/// Some assets were transferred.
		Transferred {
			asset_id: T::AssetId,
			from: T::AccountId,
			to: T::AccountId,
			amount: T::Balance,
		},
		/// Some assets were destroyed.
		Burned { asset_id: T::AssetId, owner: T::AccountId, balance: T::Balance },
		/// The management team changed.
		TeamChanged {
			asset_id: T::AssetId,
			issuer: T::AccountId,
			admin: T::AccountId,
			freezer: T::AccountId,
		},
		/// The owner changed.
		OwnerChanged { asset_id: T::AssetId, owner: T::AccountId },
		/// Some account `who` was frozen.
		Frozen { asset_id: T::AssetId, who: T::AccountId },
		/// Some account `who` was thawed.
		Thawed { asset_id: T::AssetId, who: T::AccountId },
		/// Some asset `asset_id` was frozen.
		AssetFrozen { asset_id: T::AssetId },
		/// Some asset `asset_id` was thawed.
		AssetThawed { asset_id: T::AssetId },
		/// Accounts were destroyed for given asset.
		AccountsDestroyed { asset_id: T::AssetId, accounts_destroyed: u32, accounts_remaining: u32 },
		/// Approvals were destroyed for given asset.
		ApprovalsDestroyed {
			asset_id: T::AssetId,
			approvals_destroyed: u32,
			approvals_remaining: u32,
		},
		/// An asset class is in the process of being destroyed.
		DestructionStarted { asset_id: T::AssetId },
		/// An asset class was destroyed.
		Destroyed { asset_id: T::AssetId },
		/// Some asset class was force-created.
		ForceCreated { asset_id: T::AssetId, owner: T::AccountId },
		/// New metadata has been set for an asset.
		MetadataSet {
			asset_id: T::AssetId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			is_frozen: bool,
		},
		/// Metadata has been cleared for an asset.
		MetadataCleared { asset_id: T::AssetId },
		/// (Additional) funds have been approved for transfer to a destination account.
		ApprovedTransfer {
			asset_id: T::AssetId,
			source: T::AccountId,
			delegate: T::AccountId,
			amount: T::Balance,
		},
		/// An approval for account `delegate` was cancelled by `owner`.
		ApprovalCancelled { asset_id: T::AssetId, owner: T::AccountId, delegate: T::AccountId },
		/// An `amount` was transferred in its entirety from `owner` to `destination` by
		/// the approved `delegate`.
		TransferredApproved {
			asset_id: T::AssetId,
			owner: T::AccountId,
			delegate: T::AccountId,
			destination: T::AccountId,
			amount: T::Balance,
		},
		/// An asset has had its attributes changed by the `Force` origin.
		AssetStatusChanged { asset_id: T::AssetId },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Account balance must be greater than or equal to the transfer amount.
		BalanceLow,
		/// The account to alter does not exist.
		NoAccount,
		/// The signing account has no permission to do the operation.
		NoPermission,
		/// The given asset ID is unknown.
		Unknown,
		/// The origin account is frozen.
		Frozen,
		/// The asset ID is already taken.
		InUse,
		/// Invalid witness data given.
		BadWitness,
		/// Minimum balance should be non-zero.
		MinBalanceZero,
		/// Unable to increment the consumer reference counters on the account. Either no provider
		/// reference exists to allow a non-zero balance of a non-self-sufficient asset, or the
		/// maximum number of consumers has been reached.
		NoProvider,
		/// Invalid metadata given.
		BadMetadata,
		/// No approval exists that would allow the transfer.
		Unapproved,
		/// The source account would not survive the transfer and it needs to stay alive.
		WouldDie,
		/// The asset-account already exists.
		AlreadyExists,
		/// The asset-account doesn't have an associated deposit.
		NoDeposit,
		/// The operation would result in funds being burned.
		WouldBurn,
		/// The asset is a live asset and is actively being used. Usually emit for operations such
		/// as `start_destroy` which require the asset to be in a destroying state.
		LiveAsset,
		/// The asset is not live, and likely being destroyed.
		AssetNotLive,
		/// The asset status is not the expected status.
		IncorrectStatus,
		/// The asset should be frozen before the given operation.
		NotFrozen,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Start the process of destroying a fungible asset class.
		///
		/// `start_destroy` is the first in a series of extrinsics that should be called, to allow
		/// destruction of an asset class.
		///
		/// The origin must conform to `ForceOrigin` or must be `Signed` by the asset's `owner`.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identify an existing
		///   asset.
		///
		/// The asset class must be frozen before calling `start_destroy`.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let maybe_check_owner = match T::ForceOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};
			let id: T::AssetId = id.into();
			Self::do_start_destroy(id, maybe_check_owner)
		}

		/// Destroy all accounts associated with a given asset.
		///
		/// `destroy_accounts` should only be called after `start_destroy` has been called, and the
		/// asset is in a `Destroying` state.
		///
		/// Due to weight restrictions, this function may need to be called multiple times to fully
		/// destroy all accounts. It will destroy `RemoveItemsLimit` accounts at a time.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identify an existing
		///   asset.
		///
		/// Each call emits the `Event::DestroyedAccounts` event.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::destroy_accounts(T::RemoveItemsLimit::get()))]
		pub fn destroy_accounts(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let id: T::AssetId = id.into();
			let removed_accounts = Self::do_destroy_accounts(id, T::RemoveItemsLimit::get())?;
			Ok(Some(T::WeightInfo::destroy_accounts(removed_accounts)).into())
		}

		/// Destroy all approvals associated with a given asset up to the max (T::RemoveItemsLimit).
		///
		/// `destroy_approvals` should only be called after `start_destroy` has been called, and the
		/// asset is in a `Destroying` state.
		///
		/// Due to weight restrictions, this function may need to be called multiple times to fully
		/// destroy all approvals. It will destroy `RemoveItemsLimit` approvals at a time.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identify an existing
		///   asset.
		///
		/// Each call emits the `Event::DestroyedApprovals` event.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::destroy_approvals(T::RemoveItemsLimit::get()))]
		pub fn destroy_approvals(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let id: T::AssetId = id.into();
			let removed_approvals = Self::do_destroy_approvals(id, T::RemoveItemsLimit::get())?;
			Ok(Some(T::WeightInfo::destroy_approvals(removed_approvals)).into())
		}

		/// Complete destroying asset and unreserve currency.
		///
		/// `finish_destroy` should only be called after `start_destroy` has been called, and the
		/// asset is in a `Destroying` state. All accounts or approvals should be destroyed before
		/// hand.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identify an existing
		///   asset.
		///
		/// Each successful call emits the `Event::Destroyed` event.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::finish_destroy())]
		pub fn finish_destroy(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let id: T::AssetId = id.into();
			Self::do_finish_destroy(id)
		}

		/// Move some assets from the sender account to another.
		///
		/// Origin must be Signed.
		///
		/// - `id`: The identifier of the asset to have some amount transferred.
		/// - `target`: The account to be credited.
		/// - `amount`: The amount by which the sender's balance of assets should be reduced and
		/// `target`'s balance increased. The amount actually transferred may be slightly greater in
		/// the case that the transfer would otherwise take the sender balance above zero but below
		/// the minimum balance. Must be greater than zero.
		///
		/// Emits `Transferred` with the actual amount transferred. If this takes the source balance
		/// to below the minimum for the asset, then the amount transferred is increased to take it
		/// to zero.
		///
		/// Weight: `O(1)`
		/// Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of
		/// `target`.
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			target: AccountIdLookupOf<T>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(target)?;
			let id: T::AssetId = id.into();

			let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
			Self::do_transfer(id, &origin, &dest, amount, None, f).map(|_| ())
		}

		/// Move some assets from the sender account to another, keeping the sender account alive.
		///
		/// Origin must be Signed.
		///
		/// - `id`: The identifier of the asset to have some amount transferred.
		/// - `target`: The account to be credited.
		/// - `amount`: The amount by which the sender's balance of assets should be reduced and
		/// `target`'s balance increased. The amount actually transferred may be slightly greater in
		/// the case that the transfer would otherwise take the sender balance above zero but below
		/// the minimum balance. Must be greater than zero.
		///
		/// Emits `Transferred` with the actual amount transferred. If this takes the source balance
		/// to below the minimum for the asset, then the amount transferred is increased to take it
		/// to zero.
		///
		/// Weight: `O(1)`
		/// Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of
		/// `target`.
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::transfer_keep_alive())]
		pub fn transfer_keep_alive(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			target: AccountIdLookupOf<T>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(target)?;
			let id: T::AssetId = id.into();

			let f = TransferFlags { keep_alive: true, best_effort: false, burn_dust: false };
			Self::do_transfer(id, &source, &dest, amount, None, f).map(|_| ())
		}

		/// Move some assets from one account to another.
		///
		/// Origin must be Signed and the sender should be the Admin of the asset `id`.
		///
		/// - `id`: The identifier of the asset to have some amount transferred.
		/// - `source`: The account to be debited.
		/// - `dest`: The account to be credited.
		/// - `amount`: The amount by which the `source`'s balance of assets should be reduced and
		/// `dest`'s balance increased. The amount actually transferred may be slightly greater in
		/// the case that the transfer would otherwise take the `source` balance above zero but
		/// below the minimum balance. Must be greater than zero.
		///
		/// Emits `Transferred` with the actual amount transferred. If this takes the source balance
		/// to below the minimum for the asset, then the amount transferred is increased to take it
		/// to zero.
		///
		/// Weight: `O(1)`
		/// Modes: Pre-existence of `dest`; Post-existence of `source`; Account pre-existence of
		/// `dest`.
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::force_transfer())]
		pub fn force_transfer(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			source: AccountIdLookupOf<T>,
			dest: AccountIdLookupOf<T>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let source = T::Lookup::lookup(source)?;
			let dest = T::Lookup::lookup(dest)?;
			let id: T::AssetId = id.into();

			let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
			Self::do_transfer(id, &source, &dest, amount, Some(origin), f).map(|_| ())
		}

		/// Disallow further unprivileged transfers from an account.
		///
		/// Origin must be Signed and the sender should be the Freezer of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		/// - `who`: The account to be frozen.
		///
		/// Emits `Frozen`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::freeze())]
		pub fn freeze(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			who: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(
				d.status == AssetStatus::Live || d.status == AssetStatus::Frozen,
				Error::<T, I>::AssetNotLive
			);
			ensure!(origin == d.freezer, Error::<T, I>::NoPermission);
			let who = T::Lookup::lookup(who)?;

			Account::<T, I>::try_mutate(id, &who, |maybe_account| -> DispatchResult {
				maybe_account.as_mut().ok_or(Error::<T, I>::NoAccount)?.is_frozen = true;
				Ok(())
			})?;

			Self::deposit_event(Event::<T, I>::Frozen { asset_id: id, who });
			Ok(())
		}

		/// Allow unprivileged transfers from an account again.
		///
		/// Origin must be Signed and the sender should be the Admin of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		/// - `who`: The account to be unfrozen.
		///
		/// Emits `Thawed`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::thaw())]
		pub fn thaw(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			who: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();

			let details = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(
				details.status == AssetStatus::Live || details.status == AssetStatus::Frozen,
				Error::<T, I>::AssetNotLive
			);
			ensure!(origin == details.admin, Error::<T, I>::NoPermission);
			let who = T::Lookup::lookup(who)?;

			Account::<T, I>::try_mutate(id, &who, |maybe_account| -> DispatchResult {
				maybe_account.as_mut().ok_or(Error::<T, I>::NoAccount)?.is_frozen = false;
				Ok(())
			})?;

			Self::deposit_event(Event::<T, I>::Thawed { asset_id: id, who });
			Ok(())
		}

		/// Disallow further unprivileged transfers for the asset class.
		///
		/// Origin must be Signed and the sender should be the Freezer of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		///
		/// Emits `Frozen`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::freeze_asset())]
		pub fn freeze_asset(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let d = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(d.status == AssetStatus::Live, Error::<T, I>::AssetNotLive);
				ensure!(origin == d.freezer, Error::<T, I>::NoPermission);

				d.status = AssetStatus::Frozen;

				Self::deposit_event(Event::<T, I>::AssetFrozen { asset_id: id });
				Ok(())
			})
		}

		/// Allow unprivileged transfers for the asset again.
		///
		/// Origin must be Signed and the sender should be the Admin of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be thawed.
		///
		/// Emits `Thawed`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(14)]
		#[pallet::weight(T::WeightInfo::thaw_asset())]
		pub fn thaw_asset(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let d = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(origin == d.admin, Error::<T, I>::NoPermission);
				ensure!(d.status == AssetStatus::Frozen, Error::<T, I>::NotFrozen);

				d.status = AssetStatus::Live;

				Self::deposit_event(Event::<T, I>::AssetThawed { asset_id: id });
				Ok(())
			})
		}

		/// Change the Owner of an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// - `id`: The identifier of the asset.
		/// - `owner`: The new Owner of this asset.
		///
		/// Emits `OwnerChanged`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::transfer_ownership())]
		pub fn transfer_ownership(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			owner: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let owner = T::Lookup::lookup(owner)?;
			let id: T::AssetId = id.into();

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(details.status == AssetStatus::Live, Error::<T, I>::LiveAsset);
				ensure!(origin == details.owner, Error::<T, I>::NoPermission);
				if details.owner == owner {
					return Ok(())
				}

				let metadata_deposit = Metadata::<T, I>::get(id).deposit;
				let deposit = details.deposit + metadata_deposit;

				// Move the deposit to the new owner.
				T::Currency::repatriate_reserved(&details.owner, &owner, deposit, Reserved)?;

				details.owner = owner.clone();

				Self::deposit_event(Event::OwnerChanged { asset_id: id, owner });
				Ok(())
			})
		}

		/// Change the Issuer, Admin and Freezer of an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		/// - `issuer`: The new Issuer of this asset.
		/// - `admin`: The new Admin of this asset.
		/// - `freezer`: The new Freezer of this asset.
		///
		/// Emits `TeamChanged`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::set_team())]
		pub fn set_team(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			issuer: AccountIdLookupOf<T>,
			admin: AccountIdLookupOf<T>,
			freezer: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let issuer = T::Lookup::lookup(issuer)?;
			let admin = T::Lookup::lookup(admin)?;
			let freezer = T::Lookup::lookup(freezer)?;
			let id: T::AssetId = id.into();

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(details.status == AssetStatus::Live, Error::<T, I>::AssetNotLive);
				ensure!(origin == details.owner, Error::<T, I>::NoPermission);

				details.issuer = issuer.clone();
				details.admin = admin.clone();
				details.freezer = freezer.clone();

				Self::deposit_event(Event::TeamChanged { asset_id: id, issuer, admin, freezer });
				Ok(())
			})
		}

		/// Set the metadata for an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// Funds of sender are reserved according to the formula:
		/// `MetadataDepositBase + MetadataDepositPerByte * (name.len + symbol.len)` taking into
		/// account any already reserved funds.
		///
		/// - `id`: The identifier of the asset to update.
		/// - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
		/// - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
		/// - `decimals`: The number of decimals this asset uses to represent one unit.
		///
		/// Emits `MetadataSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(17)]
		#[pallet::weight(T::WeightInfo::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();
			Self::do_set_metadata(id, &origin, name, symbol, decimals)
		}

		/// Clear the metadata for an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// Any deposit is freed for the asset owner.
		///
		/// - `id`: The identifier of the asset to clear.
		///
		/// Emits `MetadataCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let id: T::AssetId = id.into();

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(d.status == AssetStatus::Live, Error::<T, I>::AssetNotLive);
			ensure!(origin == d.owner, Error::<T, I>::NoPermission);

			Metadata::<T, I>::try_mutate_exists(id, |metadata| {
				let deposit = metadata.take().ok_or(Error::<T, I>::Unknown)?.deposit;
				T::Currency::unreserve(&d.owner, deposit);
				Self::deposit_event(Event::MetadataCleared { asset_id: id });
				Ok(())
			})
		}

		/// Force the metadata for an asset to some value.
		///
		/// Origin must be ForceOrigin.
		///
		/// Any deposit is left alone.
		///
		/// - `id`: The identifier of the asset to update.
		/// - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
		/// - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
		/// - `decimals`: The number of decimals this asset uses to represent one unit.
		///
		/// Emits `MetadataSet`.
		///
		/// Weight: `O(N + S)` where N and S are the length of the name and symbol respectively.
		#[pallet::call_index(19)]
		#[pallet::weight(T::WeightInfo::force_set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn force_set_metadata(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			is_frozen: bool,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let id: T::AssetId = id.into();

			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;

			let bounded_symbol: BoundedVec<u8, T::StringLimit> =
				symbol.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;

			ensure!(Asset::<T, I>::contains_key(id), Error::<T, I>::Unknown);
			Metadata::<T, I>::try_mutate_exists(id, |metadata| {
				let deposit = metadata.take().map_or(Zero::zero(), |m| m.deposit);
				*metadata = Some(AssetMetadata {
					deposit,
					name: bounded_name,
					symbol: bounded_symbol,
					decimals,
					is_frozen,
				});

				Self::deposit_event(Event::MetadataSet {
					asset_id: id,
					name,
					symbol,
					decimals,
					is_frozen,
				});
				Ok(())
			})
		}

		/// Clear the metadata for an asset.
		///
		/// Origin must be ForceOrigin.
		///
		/// Any deposit is returned.
		///
		/// - `id`: The identifier of the asset to clear.
		///
		/// Emits `MetadataCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(20)]
		#[pallet::weight(T::WeightInfo::force_clear_metadata())]
		pub fn force_clear_metadata(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let id: T::AssetId = id.into();

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			Metadata::<T, I>::try_mutate_exists(id, |metadata| {
				let deposit = metadata.take().ok_or(Error::<T, I>::Unknown)?.deposit;
				T::Currency::unreserve(&d.owner, deposit);
				Self::deposit_event(Event::MetadataCleared { asset_id: id });
				Ok(())
			})
		}

		/// Alter the attributes of a given asset.
		///
		/// Origin must be `ForceOrigin`.
		///
		/// - `id`: The identifier of the asset.
		/// - `owner`: The new Owner of this asset.
		/// - `issuer`: The new Issuer of this asset.
		/// - `admin`: The new Admin of this asset.
		/// - `freezer`: The new Freezer of this asset.
		/// - `min_balance`: The minimum balance of this new asset that any single account must
		/// have. If an account's balance is reduced below this, then it collapses to zero.
		/// - `is_sufficient`: Whether a non-zero balance of this asset is deposit of sufficient
		/// value to account for the state bloat associated with its balance storage. If set to
		/// `true`, then non-zero balances may be stored without a `consumer` reference (and thus
		/// an ED in the Balances pallet or whatever else is used to control user-account state
		/// growth).
		/// - `is_frozen`: Whether this asset class is frozen except for permissioned/admin
		/// instructions.
		///
		/// Emits `AssetStatusChanged` with the identity of the asset.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(21)]
		#[pallet::weight(T::WeightInfo::force_asset_status())]
		pub fn force_asset_status(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			owner: AccountIdLookupOf<T>,
			issuer: AccountIdLookupOf<T>,
			admin: AccountIdLookupOf<T>,
			freezer: AccountIdLookupOf<T>,
			#[pallet::compact] min_balance: T::Balance,
			is_sufficient: bool,
			is_frozen: bool,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let id: T::AssetId = id.into();

			Asset::<T, I>::try_mutate(id, |maybe_asset| {
				let mut asset = maybe_asset.take().ok_or(Error::<T, I>::Unknown)?;
				ensure!(asset.status != AssetStatus::Destroying, Error::<T, I>::AssetNotLive);
				asset.owner = T::Lookup::lookup(owner)?;
				asset.issuer = T::Lookup::lookup(issuer)?;
				asset.admin = T::Lookup::lookup(admin)?;
				asset.freezer = T::Lookup::lookup(freezer)?;
				asset.min_balance = min_balance;
				asset.is_sufficient = is_sufficient;
				if is_frozen {
					asset.status = AssetStatus::Frozen;
				} else {
					asset.status = AssetStatus::Live;
				}
				*maybe_asset = Some(asset);

				Self::deposit_event(Event::AssetStatusChanged { asset_id: id });
				Ok(())
			})
		}

		/// Approve an amount of asset for transfer by a delegated third-party account.
		///
		/// Origin must be Signed.
		///
		/// Ensures that `ApprovalDeposit` worth of `Currency` is reserved from signing account
		/// for the purpose of holding the approval. If some non-zero amount of assets is already
		/// approved from signing account to `delegate`, then it is topped up or unreserved to
		/// meet the right value.
		///
		/// NOTE: The signing account does not need to own `amount` of assets at the point of
		/// making this call.
		///
		/// - `id`: The identifier of the asset.
		/// - `delegate`: The account to delegate permission to transfer asset.
		/// - `amount`: The amount of asset that may be transferred by `delegate`. If there is
		/// already an approval in place, then this acts additively.
		///
		/// Emits `ApprovedTransfer` on success.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(22)]
		#[pallet::weight(T::WeightInfo::approve_transfer())]
		pub fn approve_transfer(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			delegate: AccountIdLookupOf<T>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let delegate = T::Lookup::lookup(delegate)?;
			let id: T::AssetId = id.into();
			Self::do_approve_transfer(id, &owner, &delegate, amount)
		}

		/// Cancel all of some asset approved for delegated transfer by a third-party account.
		///
		/// Origin must be Signed and there must be an approval in place between signer and
		/// `delegate`.
		///
		/// Unreserves any deposit previously reserved by `approve_transfer` for the approval.
		///
		/// - `id`: The identifier of the asset.
		/// - `delegate`: The account delegated permission to transfer asset.
		///
		/// Emits `ApprovalCancelled` on success.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(23)]
		#[pallet::weight(T::WeightInfo::cancel_approval())]
		pub fn cancel_approval(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			delegate: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let delegate = T::Lookup::lookup(delegate)?;
			let id: T::AssetId = id.into();
			let mut d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(d.status == AssetStatus::Live, Error::<T, I>::AssetNotLive);

			let approval =
				Approvals::<T, I>::take((id, &owner, &delegate)).ok_or(Error::<T, I>::Unknown)?;
			T::Currency::unreserve(&owner, approval.deposit);

			d.approvals.saturating_dec();
			Asset::<T, I>::insert(id, d);

			Self::deposit_event(Event::ApprovalCancelled { asset_id: id, owner, delegate });
			Ok(())
		}

		/// Cancel all of some asset approved for delegated transfer by a third-party account.
		///
		/// Origin must be either ForceOrigin or Signed origin with the signer being the Admin
		/// account of the asset `id`.
		///
		/// Unreserves any deposit previously reserved by `approve_transfer` for the approval.
		///
		/// - `id`: The identifier of the asset.
		/// - `delegate`: The account delegated permission to transfer asset.
		///
		/// Emits `ApprovalCancelled` on success.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(24)]
		#[pallet::weight(T::WeightInfo::force_cancel_approval())]
		pub fn force_cancel_approval(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			owner: AccountIdLookupOf<T>,
			delegate: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let id: T::AssetId = id.into();
			let mut d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(d.status == AssetStatus::Live, Error::<T, I>::AssetNotLive);
			T::ForceOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(|origin| -> DispatchResult {
					let origin = ensure_signed(origin)?;
					ensure!(origin == d.admin, Error::<T, I>::NoPermission);
					Ok(())
				})?;

			let owner = T::Lookup::lookup(owner)?;
			let delegate = T::Lookup::lookup(delegate)?;

			let approval =
				Approvals::<T, I>::take((id, &owner, &delegate)).ok_or(Error::<T, I>::Unknown)?;
			T::Currency::unreserve(&owner, approval.deposit);
			d.approvals.saturating_dec();
			Asset::<T, I>::insert(id, d);

			Self::deposit_event(Event::ApprovalCancelled { asset_id: id, owner, delegate });
			Ok(())
		}

		/// Transfer some asset balance from a previously delegated account to some third-party
		/// account.
		///
		/// Origin must be Signed and there must be an approval in place by the `owner` to the
		/// signer.
		///
		/// If the entire amount approved for transfer is transferred, then any deposit previously
		/// reserved by `approve_transfer` is unreserved.
		///
		/// - `id`: The identifier of the asset.
		/// - `owner`: The account which previously approved for a transfer of at least `amount` and
		/// from which the asset balance will be withdrawn.
		/// - `destination`: The account to which the asset balance of `amount` will be transferred.
		/// - `amount`: The amount of assets to transfer.
		///
		/// Emits `TransferredApproved` on success.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(25)]
		#[pallet::weight(T::WeightInfo::transfer_approved())]
		pub fn transfer_approved(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			owner: AccountIdLookupOf<T>,
			destination: AccountIdLookupOf<T>,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let delegate = ensure_signed(origin)?;
			let owner = T::Lookup::lookup(owner)?;
			let destination = T::Lookup::lookup(destination)?;
			let id: T::AssetId = id.into();
			Self::do_transfer_approved(id, &owner, &delegate, &destination, amount)
		}

		/// Create an asset account for non-provider assets.
		///
		/// A deposit will be taken from the signer account.
		///
		/// - `origin`: Must be Signed; the signer account must have sufficient funds for a deposit
		///   to be taken.
		/// - `id`: The identifier of the asset for the account to be created.
		///
		/// Emits `Touched` event when successful.
		#[pallet::call_index(26)]
		#[pallet::weight(T::WeightInfo::mint())]
		pub fn touch(origin: OriginFor<T>, id: T::AssetIdParameter) -> DispatchResult {
			let id: T::AssetId = id.into();
			Self::do_touch(id, ensure_signed(origin)?)
		}

		/// Return the deposit (if any) of an asset account.
		///
		/// The origin must be Signed.
		///
		/// - `id`: The identifier of the asset for the account to be created.
		/// - `allow_burn`: If `true` then assets may be destroyed in order to complete the refund.
		///
		/// Emits `Refunded` event when successful.
		#[pallet::call_index(27)]
		#[pallet::weight(T::WeightInfo::mint())]
		pub fn refund(
			origin: OriginFor<T>,
			id: T::AssetIdParameter,
			allow_burn: bool,
		) -> DispatchResult {
			let id: T::AssetId = id.into();
			Self::do_refund(id, ensure_signed(origin)?, allow_burn)
		}
	}
}
