#![cfg_attr(not(feature = "std"), no_std)]

pub use frame_support::{
	sp_runtime::traits::{One, Saturating},
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

use pallet_dao_assets::AssetBalanceOf;
use pallet_dao_core::{CurrencyOf, DaoIdOf, DepositBalanceOf, Error as DaoError};

type ProposalIdOf<T> = BoundedVec<u8, <T as pallet_dao_core::Config>::MaxLengthId>;
type ProposalOf<T> = Proposal<
	ProposalIdOf<T>,
	DaoIdOf<T>,
	<T as frame_system::Config>::AccountId,
	<T as frame_system::Config>::BlockNumber,
	pallet_dao_core::MetadataOf<T>,
>;
type VoteOf<T> = Vote<<T as frame_system::Config>::AccountId>;
type GovernanceOf<T, I = ()> = Governance<AssetBalanceOf<T, I>>;

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
	pub(super) type Votes<T: Config> = StorageMap<_, Twox64Concat, ProposalIdOf<T>, VoteOf<T>>;

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
		ProposalCreated {},
		//ProposalDestroyed,
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoTokenNotYetIssued,
		GovernanceNotSet,
		ProposalIdInvalidLengthTooLong,
		ProposalDoesNotExist,
		ProposalIsNotActive,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			prop_id: Vec<u8>,
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

			let prop_id: BoundedVec<_, _> =
				prop_id.try_into().map_err(|_| Error::<T>::ProposalIdInvalidLengthTooLong)?;

			let deposit = <T as Config>::ProposalDeposit::get();

			// reserve currency
			CurrencyOf::<T>::reserve(&sender, deposit)?;

			// reserve DAO token, but unreserve currency if that fails
			if let Err(error) = pallet_dao_assets::Pallet::<T>::do_reserve(
				asset_id.clone().into(),
				&sender,
				governance.proposal_token_deposit,
			) {
				CurrencyOf::<T>::unreserve(&sender, deposit);
				Err(error)?;
			};

			let birth_block = <frame_system::Pallet<T>>::block_number();
			// store the proposal
			<Proposals<T>>::insert(
				prop_id.clone(),
				Proposal {
					id: prop_id,
					dao_id,
					creator: sender,
					birth_block,
					status: ProposalStatus::Active,
					meta,
					meta_hash: hash,
				},
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
			let proposal = <Proposals<T>>::try_get(proposal_id.clone())
				.map_err(|_| Error::<T>::ProposalDoesNotExist)?;

			// check that the proposal is active
			ensure!(proposal.status == ProposalStatus::Active, Error::<T>::ProposalIsNotActive);

			// check if the proposal is still live (hardcoded duration in relation to the
			// created event)

			// store the vote
			<Votes<T>>::insert(proposal_id, Vote { voter: sender, aye });
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
			Ok(())
		}
	}
}
