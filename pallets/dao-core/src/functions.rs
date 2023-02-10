//! Functions for the dao-core pallet.
use super::*;

impl<T: Config> Pallet<T> {
	/// Load a dao from storage by id.
	///
	/// - `dao_id`: the unique identifier for the DAO
	pub fn load_dao(dao_id: Vec<u8>) -> Result<DaoOf<T>, Error<T>> {
		let bounded_dao_id: BoundedVec<_, _> =
			dao_id.try_into().map_err(|_| Error::<T>::DaoIdInvalidLengthTooLong)?;
		<Daos<T>>::get(bounded_dao_id).ok_or(Error::<T>::DaoDoesNotExist)
	}

	/// Determine whether 'meta' is a valid HTTP or IPFS address
	///
	/// - `meta`: the address to be validated
	pub fn metadata_is_valid(_meta: &MetadataOf<T>) -> bool {
		// this is currently empty, but we will offer a hook to
		// ink! in future versions to add custom validations.
		true
	}
}
