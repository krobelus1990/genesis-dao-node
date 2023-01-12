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
			ensure_signed(origin)?;
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_vote(origin: OriginFor<T>, in_favour: bool) -> DispatchResult {
			ensure_signed(origin)?;
			Ok(())
		}
	}
}