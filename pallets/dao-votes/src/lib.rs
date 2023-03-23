#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

pub use frame_support::{
	sp_runtime::traits::{One, Saturating, Zero},
	storage::bounded_vec::BoundedVec,
	traits::ReservableCurrency,
};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub use types::*;

mod governance_types;
use governance_types::*;

use pallet_dao_assets::{AssetBalanceOf, Pallet as Assets};
use pallet_dao_core::{
	AccountIdOf, CurrencyOf, DaoIdOf, DepositBalanceOf, Error as DaoError, Pallet as Core,
};

type ProposalIdOf<T> = BoundedVec<u8, <T as pallet_dao_core::Config>::MaxLengthId>;
type ProposalOf<T> = Proposal<
	ProposalIdOf<T>,
	DaoIdOf<T>,
	<T as frame_system::Config>::AccountId,
	<T as frame_system::Config>::BlockNumber,
	pallet_dao_core::MetadataOf<T>,
>;

type GovernanceOf<T> = Governance<AssetBalanceOf<T>>;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::storage]
	pub(super) type Governances<T: Config> =
		StorageMap<_, Twox64Concat, DaoIdOf<T>, GovernanceOf<T>>;

	#[pallet::storage]
	pub(super) type Proposals<T: Config> =
		StorageMap<_, Twox64Concat, ProposalIdOf<T>, ProposalOf<T>>;

	#[pallet::storage]
	pub(super) type Votes<T: Config> =
		StorageDoubleMap<_, Twox64Concat, ProposalIdOf<T>, Twox64Concat, AccountIdOf<T>, bool>;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_dao_core::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type ProposalDeposit: Get<DepositBalanceOf<Self>>;

		// #[pallet::constant]
		// type MaxIdLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProposalCreated {
			proposal_id: ProposalIdOf<T>,
		},
		ProposalFaulted {
			proposal_id: ProposalIdOf<T>,
			reason: Vec<u8>,
		},
		ProposalAccepted {
			proposal_id: ProposalIdOf<T>,
		},
		ProposalRejected {
			proposal_id: ProposalIdOf<T>,
		},
		VoteCast {
			proposal_id: ProposalIdOf<T>,
			voter: AccountIdOf<T>,
		},
		SetGovernanceMajorityVote {
			dao_id: DaoIdOf<T>,
			proposal_duration: u32,
			proposal_token_deposit: T::Balance,
			minimum_majority_per_256: u8,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoTokenNotYetIssued,
		GovernanceNotSet,
		ProposalIdInvalidLengthTooLong,
		ProposalDoesNotExist,
		ProposalIsNotActive,
		ProposalDurationHasNotPassed,
		ProposalDurationHasPassed,
		SenderIsNotDaoOwner,
		HistoryHorizonHasPassed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			proposal_id: Vec<u8>,
			meta: Vec<u8>,
			hash: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let dao = pallet_dao_core::Pallet::<T>::load_dao(dao_id)?;
			let dao_id = dao.id;
			let asset_id = dao.asset_id.ok_or(Error::<T>::DaoTokenNotYetIssued)?;
			let governance =
				<Governances<T>>::get(dao_id.clone()).ok_or(Error::<T>::GovernanceNotSet)?;

			let meta: BoundedVec<_, _> =
				meta.try_into().map_err(|_| DaoError::<T>::MetadataInvalidLengthTooLong)?;
			let hash: BoundedVec<_, _> =
				hash.try_into().map_err(|_| DaoError::<T>::HashInvalidWrongLength)?;

			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			let deposit = <T as Config>::ProposalDeposit::get();

			// reserve currency
			CurrencyOf::<T>::reserve(&sender, deposit)?;

			// reserve DAO token, but unreserve currency if that fails
			if let Err(error) = pallet_dao_assets::Pallet::<T>::do_reserve(
				asset_id.into(),
				&sender,
				governance.proposal_token_deposit,
			) {
				CurrencyOf::<T>::unreserve(&sender, deposit);
				Err(error)?;
			};

			let birth_block = <frame_system::Pallet<T>>::block_number();
			// store the proposal
			<Proposals<T>>::insert(
				proposal_id.clone(),
				Proposal {
					id: proposal_id.clone(),
					dao_id,
					creator: sender,
					birth_block,
					status: ProposalStatus::Active,
					meta,
					meta_hash: hash,
				},
			);
			// emit an event
			Self::deposit_event(Event::<T>::ProposalCreated { proposal_id });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn fault_proposal(
			origin: OriginFor<T>,
			proposal_id: Vec<u8>,
			reason: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			// check that a proposal exists with the given id
			let mut proposal = <Proposals<T>>::try_get(proposal_id.clone())
				.map_err(|_| Error::<T>::ProposalDoesNotExist)?;

			// check that sender is owner of the DAO
			ensure!(
				sender == Core::<T>::get_dao(&proposal.dao_id).expect("DAO exists").owner,
				Error::<T>::SenderIsNotDaoOwner
			);

			proposal.status = ProposalStatus::Faulty;
			<Proposals<T>>::insert(proposal_id.clone(), proposal.clone());

			// unreserve currency
			CurrencyOf::<T>::unreserve(&proposal.creator, <T as Config>::ProposalDeposit::get());

			Self::deposit_event(Event::<T>::ProposalFaulted { proposal_id, reason });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn finalize_proposal(origin: OriginFor<T>, proposal_id: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			// check that a proposal exists with the given id
			let mut proposal = <Proposals<T>>::try_get(proposal_id.clone())
				.map_err(|_| Error::<T>::ProposalDoesNotExist)?;

			// check that the proposal is currently active
			ensure!(proposal.status == ProposalStatus::Active, Error::<T>::ProposalIsNotActive);

			let governance =
				<Governances<T>>::get(&proposal.dao_id).ok_or(Error::<T>::GovernanceNotSet)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			// check that the proposal has run for its entire duration
			ensure!(
				current_block - proposal.birth_block > governance.proposal_duration.into(),
				Error::<T>::ProposalDurationHasNotPassed
			);

			// check that there is definitely enough history
			ensure!(
				current_block - proposal.birth_block < T::HistoryHorizon::get().into(),
				Error::<T>::HistoryHorizonHasPassed
			);

			let asset_id = Core::<T>::get_dao(&proposal.dao_id)
				.expect("DAO exists")
				.asset_id
				.expect("asset has been issued");

			// count votes
			let mut votes_for: AssetBalanceOf<T> = Zero::zero();
			let mut votes_against: AssetBalanceOf<T> = Zero::zero();
			for (account_id, in_favor) in <Votes<T>>::iter_prefix(&proposal_id) {
				let token_balance = Assets::<T>::total_historical_balance(
					asset_id.into(),
					account_id,
					proposal.birth_block,
				)
				.expect("History exists (horizon checked above)");
				if in_favor {
					votes_for += token_balance;
				} else {
					votes_against += token_balance;
				}
			}

			// determine whether proposal has required votes and set status accordingly
			match governance.voting {
				Voting::Majority { minimum_majority_per_256 } => {
					if votes_for > votes_against && {
						let token_supply = Assets::<T>::total_historical_supply(
							asset_id.into(),
							proposal.birth_block,
						)
						.expect("History exists (horizon checked above)");
						let required_majority = token_supply /
							Into::<AssetBalanceOf<T>>::into(256_u32) *
							minimum_majority_per_256.into();
						// check for the required majority
						votes_for - votes_against >= required_majority
					} {
						proposal.status = ProposalStatus::Accepted;
					} else {
						proposal.status = ProposalStatus::Rejected;
					}
				},
			}

			// record updated proposal status
			<Proposals<T>>::insert(proposal_id.clone(), proposal.clone());

			// unreserve currency
			CurrencyOf::<T>::unreserve(&sender, <T as Config>::ProposalDeposit::get());

			// emit event
			Self::deposit_event(match proposal.status {
				ProposalStatus::Accepted => Event::ProposalAccepted { proposal_id },
				ProposalStatus::Rejected => Event::ProposalRejected { proposal_id },
				_ => unreachable!(),
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn vote(
			origin: OriginFor<T>,
			proposal_id: Vec<u8>,
			in_favor: Option<bool>,
		) -> DispatchResult {
			let voter = ensure_signed(origin)?;
			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			// check that a proposal exists with the given id
			let proposal = <Proposals<T>>::try_get(proposal_id.clone())
				.map_err(|_| Error::<T>::ProposalDoesNotExist)?;

			// check that the proposal is active
			ensure!(proposal.status == ProposalStatus::Active, Error::<T>::ProposalIsNotActive);

			let governance =
				<Governances<T>>::get(&proposal.dao_id).ok_or(Error::<T>::GovernanceNotSet)?;

			// check that the proposal has not yet run for its entire duration
			ensure!(
				<frame_system::Pallet<T>>::block_number() - proposal.birth_block <=
					governance.proposal_duration.into(),
				Error::<T>::ProposalDurationHasPassed
			);

			<Votes<T>>::set(proposal_id.clone(), voter.clone(), in_favor);
			Self::deposit_event(Event::<T>::VoteCast { proposal_id, voter });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_governance_majority_vote(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			proposal_duration: u32,
			proposal_token_deposit: T::Balance,
			minimum_majority_per_256: u8,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = pallet_dao_core::Pallet::<T>::load_dao(dao_id)?;
			let dao_id = dao.id;
			ensure!(dao.owner == sender, DaoError::<T>::DaoSignerNotOwner);
			let voting = Voting::Majority { minimum_majority_per_256 };
			let gov = GovernanceOf::<T> { proposal_duration, proposal_token_deposit, voting };
			<Governances<T>>::set(dao_id.clone(), Some(gov));
			Self::deposit_event(Event::<T>::SetGovernanceMajorityVote {
				dao_id,
				proposal_duration,
				proposal_token_deposit,
				minimum_majority_per_256,
			});
			Ok(())
		}
	}
}
