//! DAO Votes pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;

use crate::Pallet as Votes;
use pallet_dao_core::{Config as DaoConfig, Currency, Pallet as DaoCore};

const SEED: u32 = 0;

/// Add voters to a proposal
/// - `proposal_id`: id of the proposal
fn add_voters<T: Config>(proposal_id: Vec<u8>, n: u32) {
	for i in 0..n {
		let voter = account("voter", i, SEED);
		assert!(Votes::<T>::vote(RawOrigin::Signed(voter).into(), proposal_id.clone(), Some(true))
			.is_ok());
	}
}

/// A whitelisted caller with enough funds
fn setup_caller<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let balance = u32::MAX.into();
	<T as DaoConfig>::Currency::issue(balance);
	<T as DaoConfig>::Currency::make_free_balance_be(&caller, balance);
	caller
}

/// Creates a DAO for the given caller
/// - `caller`: AccountId of the dao creator
fn setup_dao<T: Config>(caller: T::AccountId) -> Vec<u8> {
	let dao_id: Vec<u8> = b"GDAO".to_vec();
	let dao_name = b"Genesis DAO".to_vec();
	let origin = RawOrigin::Signed(caller);
	assert!(DaoCore::<T>::create_dao(origin.clone().into(), dao_id.clone(), dao_name,).is_ok());
	assert!(DaoCore::<T>::issue_token(origin.into(), dao_id.clone(), 1000_u32.into()).is_ok());
	dao_id
}

/// Creates a DAO for the given caller with a governance set
/// - `caller`: AccountId of the dao creator
fn setup_dao_with_governance<T: Config>(caller: T::AccountId) -> Vec<u8> {
	let dao_id = setup_dao::<T>(caller.clone());
	let proposal_duration = 1_u32;
	let proposal_token_deposit = 1_u32.into();
	let minimum_majority_per_1024 = 10;
	assert!(Votes::<T>::set_governance_majority_vote(
		RawOrigin::Signed(caller).into(),
		dao_id.clone(),
		proposal_duration,
		proposal_token_deposit,
		minimum_majority_per_1024
	)
	.is_ok());
	dao_id
}

/// Creates a DAO for the given caller with a governance set and a proposal created
/// - `caller`: AccountId of the dao creator
/// - `dao_id`: id of the dao
fn setup_proposal<T: Config>(caller: T::AccountId, dao_id: Vec<u8>) -> Vec<u8> {
	let prop_id = b"PROP".to_vec();
	let metadata = b"http://my.cool.proposal".to_vec();
	// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	assert!(Votes::<T>::create_proposal(
		RawOrigin::Signed(caller).into(),
		dao_id,
		prop_id.clone(),
		metadata,
		hash
	)
	.is_ok());
	prop_id
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	create_proposal {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = b"PROP".to_vec();
		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), dao_id, proposal_id.clone(), metadata, hash)
	verify {
		let proposal_id: BoundedVec<_, _> = proposal_id.try_into().expect("fits");
		assert_last_event::<T>(Event::ProposalCreated { proposal_id }.into());
	}

	fault_proposal {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = setup_proposal::<T>(caller.clone(), dao_id);
		let reason = b"Bad".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), proposal_id.clone(), reason.clone())
	verify {
		let proposal_id: BoundedVec<_, _> = proposal_id.try_into().expect("fits");
		assert_last_event::<T>(Event::ProposalFaulted { proposal_id, reason }.into());
	}

	finalize_proposal {
		let v in 0 .. T::FinalizeVotesLimit::get();
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = setup_proposal::<T>(caller.clone(), dao_id);
		add_voters::<T>(proposal_id.clone(), v);
		frame_system::Pallet::<T>::set_block_number(5_u32.into());
	}: _(RawOrigin::Signed(caller.clone()), proposal_id.clone())
	verify {
		let proposal_id: BoundedVec<_, _> = proposal_id.try_into().expect("fits");
		assert_last_event::<T>(Event::ProposalRejected { proposal_id }.into());
	}

	vote {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = setup_proposal::<T>(caller.clone(), dao_id);
		let voter = caller;
		let in_favor = Some(true);
	}: _(RawOrigin::Signed(voter.clone()), proposal_id.clone(), in_favor)
	verify {
		let proposal_id: BoundedVec<_, _> = proposal_id.try_into().expect("fits");
		assert_last_event::<T>(Event::VoteCast { proposal_id, voter, in_favor }.into());
	}

	set_governance_majority_vote {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao::<T>(caller.clone());
		let proposal_duration = 1_u32;
		let proposal_token_deposit = 1_u32.into();
		let minimum_majority_per_1024 = 10;
	}: _(RawOrigin::Signed(caller.clone()), dao_id.clone(), proposal_duration, proposal_token_deposit, minimum_majority_per_1024)
	verify {
		let dao_id: BoundedVec<_, _> = dao_id.try_into().expect("fits");
		assert_last_event::<T>(Event::SetGovernanceMajorityVote { dao_id, proposal_duration, proposal_token_deposit, minimum_majority_per_1024 }.into());
	}

	impl_benchmark_test_suite!(Votes, crate::mock::new_test_ext(), crate::mock::Test)
}
