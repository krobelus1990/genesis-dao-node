#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::prelude::*;

pub use frame_support::{
	storage::bounded_vec::BoundedVec,
	traits::{
		tokens::fungibles::{
            Create, Mutate,
            metadata::Mutate as MetadataMutate
		},
		Currency,
	},
	sp_runtime::traits::{Saturating, One},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;

mod types;
pub use types::Dao;
use pallet_assets;

type DepositBalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> = <T as Config>::AssetId;
type DaoOf<T> = Dao<<T as frame_system::Config>::AccountId, BoundedVec<u8, <T as Config>::MaxLength>, AssetIdOf<T>>;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type AssetId: IsType<<Self as pallet_assets::Config>::AssetId>
			+ Parameter
			+ Default
			+ MaxEncodedLen
			+ One
			+ Saturating;

		#[pallet::constant]
		type DaoDeposit: Get<DepositBalanceOf<Self>>;

		#[pallet::constant]
		type MinLength: Get<u32>;

		#[pallet::constant]
		type MaxLength: Get<u32>;

        #[pallet::constant]
        type TokenUnits: Get<u8>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		DaoCreated { owner: T::AccountId, dao_id: BoundedVec<u8, T::MaxLength>},
		DaoDestroyed { dao_id: BoundedVec<u8, T::MaxLength> },
        DaoTokenIssued { dao_id: BoundedVec<u8, T::MaxLength>, supply: <T as pallet_assets::Config>::Balance },
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoIdInvalidLengthTooShort,
		DaoIdInvalidLengthTooLong,
		DaoNameInvalidLengthTooShort,
		DaoNameInvalidLengthTooLong,
		DaoAlreadyExists,
		DaoDoesNotExist,
		DaoSignerNotOwner,
		DaoTokenAlreadyIssued
	}

	/// Key-Value Store of all _DAOs_, with the key being the `dao_id`.
	#[pallet::storage]
	#[pallet::getter(fn get_dao)]
	pub(super) type Daos<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxLength>,
		DaoOf<T>
	>;

	/// Internal incrementor of all assets issued by this module.
	/// The first asset starts with _1_ (sic!, not 0) and then the id is assigned by order of creation.
	#[pallet::storage]
	#[pallet::getter(fn get_current_asset_id)]
	pub type CurrentAssetId<T> = StorageValue<_, AssetIdOf<T>, ValueQuery>;


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Create a fresh DAO.
		///
		/// - `dao_id`: A unique identifier for the DAO, bounded by _MinLength_ & _MaxLength_ in the config
		/// - `dao_name`: The name of the to-be-created DAO.
		///
		/// A DAO must reserve the _DaoDeposit_ fee.
		#[pallet::weight(0)]
		pub fn create_dao(origin: OriginFor<T>, dao_id: Vec<u8>, dao_name: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_dao_id: BoundedVec<_, _> = dao_id.try_into().map_err(|_| Error::<T>::DaoIdInvalidLengthTooLong)?;
			ensure!(bounded_dao_id.len() >= T::MinLength::get() as usize, Error::<T>::DaoIdInvalidLengthTooShort);
			ensure!(!<Daos<T>>::contains_key(&bounded_dao_id), Error::<T>::DaoAlreadyExists);

			let bounded_name: BoundedVec<_, _> = dao_name.try_into().map_err(|_| Error::<T>::DaoNameInvalidLengthTooLong)?;
			ensure!(bounded_name.len() >= T::MinLength::get() as usize, Error::<T>::DaoNameInvalidLengthTooShort);

			<T as Config>::Currency::reserve(&sender, <T as Config>::DaoDeposit::get())?;

			Self::deposit_event(Event::DaoCreated { owner: sender.clone(), dao_id: bounded_dao_id.clone() });
			<Daos<T>>::insert(bounded_dao_id.clone(), Dao {
				id: bounded_dao_id,
				name: bounded_name,
				owner: sender.clone(),
				asset_id: None
			});
			Ok(())
		}

		/// Destroy a DAO.
		///
		/// - `dao_id`: The DAO to destroy
		///
		/// Signer of this TX needs to be the owner of the DAO.
		#[pallet::weight(0)]
		pub fn destroy_dao(origin: OriginFor<T>, dao_id: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = Self::load_dao(dao_id)?;
			ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);

			<T as Config>::Currency::unreserve(&sender, <T as Config>::DaoDeposit::get());

			Self::deposit_event(Event::DaoDestroyed { dao_id: dao.id.clone() });
			<Daos<T>>::remove(&dao.id);
			Ok(())
		}

		/// Issue the DAO token
		///
		/// - `dao_id`: The DAO that wants to issue a token
		/// - `supply`: The total supply by the DAO
		///
		/// Tokens can only be issued once and the signer of this TX needs to be the owner
		/// of the DAO.
		#[pallet::weight(0)]
		pub fn issue_token(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			supply: <T as pallet_assets::Config>::Balance
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = Self::load_dao(dao_id)?;
			ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);
			ensure!(dao.asset_id.is_none(), Error::<T>::DaoTokenAlreadyIssued);

			// // create a fresh asset
			<CurrentAssetId<T>>::mutate(|asset_id| asset_id.saturating_inc());
			<pallet_assets::pallet::Pallet<T> as Create<T::AccountId>>::create(
				<CurrentAssetId<T>>::get().into(),
        	    dao.owner.clone(),
    	        true,
	            One::one(),
            )?;

			// and distribute it to the owner
			<pallet_assets::pallet::Pallet<T> as Mutate<T::AccountId>>::mint_into(
                <CurrentAssetId<T>>::get().into(),
                &dao.owner,
                supply
            )?;

			// set the token metadata to the dao metadata
            <pallet_assets::pallet::Pallet<T> as MetadataMutate<T::AccountId>>::set(
                <CurrentAssetId<T>>::get().into(),
                &dao.owner,
                dao.name.into(),
                dao.id.clone().into(),
                <T as Config>::TokenUnits::get()
            )?;

            Self::deposit_event(Event::DaoTokenIssued { dao_id: dao.id.clone(), supply });
			// ... and link the dao to the asset
            <Daos<T>>::try_mutate(dao.id, |maybe_dao| {
                let d = maybe_dao.as_mut().ok_or(Error::<T>::DaoDoesNotExist)?;
                d.asset_id = Some(<CurrentAssetId<T>>::get());
                Ok(())
            })
		}
	}
}