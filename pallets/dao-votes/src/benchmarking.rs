//! DAO Votes pallet benchmarking.

use super::*;
use crate::test_utils::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;

use crate::Pallet as Votes;
use pallet_dao_core::{Config as DaoConfig, Currency};

const SEED: u32 = 0;

/// Add voters to a proposal
/// - `proposal_id`: id of the proposal
fn add_voters<T: Config>(proposal_id: T::ProposalId, n: u32) {
	for i in 0..n {
		let voter = account("voter", i, SEED);
		assert!(Votes::<T>::vote(RawOrigin::Signed(voter).into(), proposal_id, Some(true)).is_ok());
	}
}

/// A whitelisted caller with enough funds
fn setup_caller<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let min_balance = <T as DaoConfig>::Currency::minimum_balance();
	let balance = min_balance * u32::MAX.into() * u32::MAX.into();
	<T as DaoConfig>::Currency::issue(balance);
	<T as DaoConfig>::Currency::make_free_balance_be(&caller, balance);
	assert_eq!(<T as DaoConfig>::Currency::free_balance(&caller), balance);
	caller
}

benchmarks! {
	create_proposal {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), dao_id)
	verify {
		assert_last_event::<T>(Event::ProposalCreated { proposal_id: Votes::<T>::get_current_proposal_id() }.into());
	}

	set_metadata {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = create_proposal_id::<T>(caller.clone(), dao_id);
		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), proposal_id, metadata, hash)
	verify {
		assert_last_event::<T>(Event::ProposalMetadataSet { proposal_id: Votes::<T>::get_current_proposal_id() }.into());
	}

	fault_proposal {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = setup_proposal::<T>(caller.clone(), dao_id);
		let reason = b"Bad".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), proposal_id.clone(), reason.clone())
	verify {
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

	mark_implemented {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao_with_governance::<T>(caller.clone());
		let proposal_id = setup_accepted_proposal::<T>(caller.clone(), dao_id);
	}: _(RawOrigin::Signed(caller.clone()), proposal_id.clone())
	verify {
		assert_last_event::<T>(Event::ProposalImplemented { proposal_id }.into());
	}

	impl_benchmark_test_suite!(Votes, crate::mock::new_test_ext(), crate::mock::Test)
}
