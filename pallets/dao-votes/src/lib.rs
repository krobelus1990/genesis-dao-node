#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use frame_support::{
	storage::bounded_vec::BoundedVec,
	sp_runtime::traits::{Saturating, One},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub use types::{Proposal, Vote};


#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*};
	use frame_system::pallet_prelude::*;


	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// #[pallet::constant]
		// type MaxIdLength: Get<u32>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {

	}

	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	{
		#[pallet::weight(0)]
		pub fn create_proposal(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// want to reserve x amount of DAO Tokens for the creation of proposal
			// add the proposal to the storage
			// emit an event
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_vote(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// would check if a proposal exists with the given id
			// would check if the proposal is still live (hardcoded duration in relation to the created event)
			// store the vote with in favour or not in favour and the voter
			Ok(())
		}
	}
}