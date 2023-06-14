//! DAO Votes utility functions for testing and benchmarking.

use super::*;
use frame_system::RawOrigin;

use crate::Pallet as Votes;
use frame_system::{Config as SystemConfig, Pallet as System};
use pallet_dao_core::Pallet as DaoCore;

/// Creates a DAO for the given caller
/// - `caller`: AccountId of the dao creator
pub fn setup_dao<T: Config>(caller: T::AccountId) -> Vec<u8> {
	let dao_id: Vec<u8> = b"GDAO".to_vec();
	let dao_name = b"Genesis DAO".to_vec();
	let origin = RawOrigin::Signed(caller);
	assert_eq!(DaoCore::<T>::create_dao(origin.clone().into(), dao_id.clone(), dao_name), Ok(()));
	assert_eq!(DaoCore::<T>::issue_token(origin.into(), dao_id.clone(), 1000_u32.into()), Ok(()));
	dao_id
}

/// Creates a DAO for the given caller with a governance set
/// - `caller`: AccountId of the dao creator
pub fn setup_dao_with_governance<T: Config>(caller: T::AccountId) -> Vec<u8> {
	let dao_id = setup_dao::<T>(caller.clone());
	let proposal_duration = 0_u32;
	let proposal_token_deposit = 1_u32.into();
	let minimum_majority_per_1024 = 10;
	assert_eq!(
		Votes::<T>::set_governance_majority_vote(
			RawOrigin::Signed(caller).into(),
			dao_id.clone(),
			proposal_duration,
			proposal_token_deposit,
			minimum_majority_per_1024
		),
		Ok(())
	);
	dao_id
}

/// Creates a proposal id for the given dao_id and caller
/// - `caller`: AccountId of the dao creator
/// - `dao_id`: id of the dao
pub fn create_proposal_id<T: Config>(caller: T::AccountId, dao_id: Vec<u8>) -> T::ProposalId {
	assert_eq!(Votes::<T>::create_proposal(RawOrigin::Signed(caller).into(), dao_id,), Ok(()));
	Votes::<T>::get_current_proposal_id()
}

/// Creates a proposal for the given proposal_id and caller
/// - `caller`: AccountId of the dao creator
/// - `proposal_id`: id of the proposal
pub fn setup_proposal_with_id<T: Config>(caller: T::AccountId, proposal_id: T::ProposalId) {
	let metadata = b"http://my.cool.proposal".to_vec();
	// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	assert_eq!(
		Votes::<T>::set_metadata(RawOrigin::Signed(caller).into(), proposal_id, metadata, hash),
		Ok(())
	);
}

/// Creates a proposal for the given dao_id and caller
/// - `caller`: AccountId of the dao creator
/// - `dao_id`: id of the dao
pub fn setup_proposal<T: Config>(caller: T::AccountId, dao_id: Vec<u8>) -> T::ProposalId {
	let proposal_id = create_proposal_id::<T>(caller.clone(), dao_id);
	setup_proposal_with_id::<T>(caller, proposal_id);
	proposal_id
}

pub fn run_to_block<T: Config>(n: <T as SystemConfig>::BlockNumber) {
	use frame_support::traits::{OnFinalize, OnInitialize};
	while System::<T>::block_number() < n {
		let mut block = System::<T>::block_number();
		Assets::<T>::on_finalize(block);
		System::<T>::on_finalize(block);
		System::<T>::reset_events();
		block += 1_u32.into();
		System::<T>::set_block_number(block);
		System::<T>::on_initialize(block);
		Assets::<T>::on_initialize(block);
	}
}
