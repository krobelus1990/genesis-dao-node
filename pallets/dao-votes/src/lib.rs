#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use dao_core::{Config, Pallet as DaoCore};
pub use frame_support::{
	sp_runtime::traits::{One, Saturating},
	storage::bounded_vec::BoundedVec,
	traits::ReservableCurrency,
};
use pallet_dao_core::{self as dao_core};
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub use types::{Proposal, Vote};

type DaoId<T> = BoundedVec<u8, <T as Config>::MaxLength>;
type ProposalId<T> = BoundedVec<u8, <T as dao_core::Config>::MaxLength>;

type ProposalOf<T> = Proposal<ProposalId<T>, DaoId<T>, <T as frame_system::Config>::AccountId>;
type VoteOf<T> = Vote<<T as frame_system::Config>::AccountId>;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::storage]
	pub(super) type Proposals<T: Config> = StorageMap<_, Twox64Concat, DaoId<T>, ProposalOf<T>>;

	#[pallet::storage]
	pub(super) type Votes<T: Config> = StorageMap<_, Twox64Concat, ProposalId<T>, VoteOf<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + dao_core::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// #[pallet::constant]
		// type MaxIdLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProposalCreated {},
		//ProposalDestroyed,
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoTokenNotYetIssued,
		ProposalIdInvalidLengthTooLong,
		ProposalDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			proposal_id: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = DaoCore::<T>::load_dao(dao_id)?;
			ensure!(dao.asset_id.is_some(), Error::<T>::DaoTokenNotYetIssued);

			// want to reserve x amount of DAO Tokens for the creation of proposal
			//<T as dao_core::Config>::Currency::reserve(10);

			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			// store the proposal
			<Proposals<T>>::insert(
				dao.id.clone(),
				Proposal { id: proposal_id, dao_id: dao.id, creator: sender },
			);
			// emit an event
			Self::deposit_event(Event::<T>::ProposalCreated {});
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_vote(
			origin: OriginFor<T>,
			proposal_id: Vec<u8>,
			aye: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let proposal_id: BoundedVec<_, _> =
				proposal_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			// check that a proposal exists with the given id
			ensure!(<Proposals<T>>::contains_key(proposal_id.clone()), Error::<T>::ProposalDoesNotExist);

			// check if the proposal is still live (hardcoded duration in relation to the
			// created event) store the vote with in favour or not in favour and the voter

			// store the vote
			<Votes<T>>::insert(proposal_id, Vote { voter: sender, aye });
			Ok(())
		}
	}
}
