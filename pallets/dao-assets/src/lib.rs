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

mod functions;
mod impl_fungibles;
mod types;

pub use types::*;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	sp_runtime::{
		traits::{
			AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
		},
		ArithmeticError, TokenError,
	},
	traits::{
		tokens::{DepositConsequence, WithdrawConsequence},
		Currency, EnsureOriginWithArg, ReservableCurrency,
	},
	BoundedBTreeMap,
};
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;

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
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

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
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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

		/// The amount of funds that must be reserved when creating a new approval.
		#[pallet::constant]
		type ApprovalDeposit: Get<DepositBalanceOf<Self>>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The age for which historical data may be removed.
		/// For example, if this is 100 and the current block is 150,
		/// then history for blocks 50 and older may be removed.
		#[pallet::constant]
		type HistoryHorizon: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Helper trait for benchmarks.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::AssetIdParameter>;
	}

	#[pallet::storage]
	/// Details of an asset.
	pub type Asset<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, AssetDetails<T::Balance, T::AccountId>>;

	#[pallet::storage]
	/// The holdings of a specific account for a specific asset.
	pub(super) type Account<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		Blake2_128Concat,
		T::AccountId,
		AssetAccountOf<T>,
	>;

	#[pallet::storage]
	/// Approved balance transfers. First balance is the amount approved for transfer. Second
	/// is the amount of `T::Currency` reserved for storing this.
	/// First key is the asset ID, second key is the owner and third key is the delegate.
	pub(super) type Approvals<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AssetId>,
			NMapKey<Blake2_128Concat, T::AccountId>, // owner
			NMapKey<Blake2_128Concat, T::AccountId>, // delegate
		),
		Approval<T::Balance, DepositBalanceOf<T>>,
	>;

	#[pallet::storage]
	/// Metadata of an asset.
	pub(super) type Metadata<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetMetadata<BoundedVec<u8, T::StringLimit>>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// History for the total supply across all accounts.
	pub(super) type SupplyHistory<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		BoundedBTreeMap<BlockNumberFor<T>, AssetBalanceOf<T>, T::HistoryHorizon>,
	>;

	#[pallet::storage]
	/// History for the total balance of each account for all assets.
	pub(super) type AccountHistory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		Blake2_128Concat,
		T::AccountId,
		BoundedBTreeMap<BlockNumberFor<T>, AssetBalanceOf<T>, T::HistoryHorizon>,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Genesis assets: id, owner, is_sufficient, min_balance
		#[allow(clippy::type_complexity)]
		pub assets: Vec<(T::AssetId, T::AccountId, T::Balance)>,
		/// Genesis metadata: id, name, symbol, decimals
		#[allow(clippy::type_complexity)]
		pub metadata: Vec<(T::AssetId, Vec<u8>, Vec<u8>, u8)>,
		/// Genesis accounts: id, account_id, balance
		pub accounts: Vec<(T::AssetId, T::AccountId, T::Balance)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				assets: Default::default(),
				metadata: Default::default(),
				accounts: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (id, owner, min_balance) in &self.assets {
				assert!(!Asset::<T>::contains_key(id), "Asset id already in use");
				assert!(!min_balance.is_zero(), "Min balance should not be zero");
				Asset::<T>::insert(
					id,
					AssetDetails {
						owner: owner.clone(),
						supply: Zero::zero(),
						min_balance: *min_balance,
						accounts: 0,
						approvals: 0,
						status: AssetStatus::Live,
					},
				);
			}

			for (id, name, symbol, decimals) in &self.metadata {
				assert!(Asset::<T>::contains_key(id), "Asset does not exist");

				let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().expect("asset name is too long");
				let bounded_symbol: BoundedVec<u8, T::StringLimit> =
					symbol.clone().try_into().expect("asset symbol is too long");

				let metadata = AssetMetadata {
					name: bounded_name,
					symbol: bounded_symbol,
					decimals: *decimals,
				};
				Metadata::<T>::insert(id, metadata);
			}

			for (id, account_id, amount) in &self.accounts {
				let result = <Pallet<T>>::increase_balance(
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
	pub enum Event<T: Config> {
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
		MetadataSet { asset_id: T::AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8 },
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
	pub enum Error<T> {
		/// Account balance must be greater than or equal to the transfer amount.
		BalanceLow,
		/// The account to alter does not exist.
		NoAccount,
		/// The signing account has no permission to do the operation.
		NoPermission,
		/// The given asset ID is unknown.
		Unknown,
		/// The asset ID is already taken.
		InUse,
		/// Invalid witness data given.
		BadWitness,
		/// Minimum balance should be non-zero.
		MinBalanceZero,
		/// Invalid metadata given.
		BadMetadata,
		/// No approval exists that would allow the transfer.
		Unapproved,
		/// The source account would not survive the transfer and it needs to stay alive.
		WouldDie,
		/// The asset-account already exists.
		AlreadyExists,
		/// The operation would result in funds being burned.
		WouldBurn,
		/// The asset is not live, and likely being destroyed.
		AssetNotLive,
		/// The asset status is not the expected status.
		IncorrectStatus,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Start the process of destroying a fungible asset class.
		///
		/// `start_destroy` is the first in a series of extrinsics that should be called, to allow
		/// destruction of an asset class.
		///
		/// The origin must conform to `ForceOrigin` or must be `Signed` by the asset's `owner`.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identify an existing
		///   asset.
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
			Self::do_transfer(id, &origin, &dest, amount, f).map(|_| ())
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
			Self::do_transfer(id, &source, &dest, amount, f).map(|_| ())
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
			let mut d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
			ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);

			let approval =
				Approvals::<T>::take((id, &owner, &delegate)).ok_or(Error::<T>::Unknown)?;
			T::Currency::unreserve(&owner, approval.deposit);

			d.approvals.saturating_dec();
			Asset::<T>::insert(id, d);

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
	}
}
