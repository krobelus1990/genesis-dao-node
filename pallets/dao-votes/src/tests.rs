use crate::{mock::*, types::*, Config, Error, Proposals, Votes};
use frame_support::{assert_noop, assert_ok, traits::TypedGet, BoundedVec};
use frame_system::ensure_signed;
use pallet_dao_core::{CurrencyOf, Error as DaoError};

#[test]
fn can_create_a_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);
		let sender = ensure_signed(origin.clone()).unwrap();

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();

		// cannot create a proposal without a DAO
		assert_noop!(
			DaoVotes::create_proposal(
				origin.clone(),
				dao_id.clone(),
				prop_id.clone(),
				metadata.clone(),
				hash.clone()
			),
			DaoError::<Test>::DaoDoesNotExist
		);

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));

		// cannot create a proposal without DAO tokens existing (because they need to be reserved)
		assert_noop!(
			DaoVotes::create_proposal(
				origin.clone(),
				dao_id.clone(),
				prop_id.clone(),
				metadata.clone(),
				hash.clone()
			),
			Error::<Test>::DaoTokenNotYetIssued
		);

		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));

		let dao = pallet_dao_core::Pallet::<Test>::load_dao(dao_id.clone()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		// check that no DAO tokens are reserved yet
		assert_eq!(
			pallet_dao_assets::pallet::Pallet::<Test>::reserved(asset_id, sender),
			Default::default()
		);

		let reserved_currency = CurrencyOf::<Test>::reserved_balance(sender);

		// cannot create a proposal without a governance set
		assert_noop!(
			DaoVotes::create_proposal(
				origin.clone(),
				dao_id.clone(),
				prop_id.clone(),
				metadata.clone(),
				hash.clone()
			),
			Error::<Test>::GovernanceNotSet
		);

		// preparation: set governance
		let duration = 4200;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 10; // slightly less than 1 %
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));

		// test creating a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id.clone(),
			prop_id.clone(),
			metadata.clone(),
			hash.clone()
		));

		// check that a proposal exists with the given id
		let bounded_prop_id: BoundedVec<_, _> = prop_id.clone().try_into().unwrap();
		assert!(<Proposals<Test>>::contains_key(bounded_prop_id));

		// creating a proposal should reserve currency
		assert_eq!(
			CurrencyOf::<Test>::reserved_balance(sender),
			reserved_currency + <Test as Config>::ProposalDeposit::get()
		);

		// creating a proposal should reserve DAO tokens
		assert_eq!(
			pallet_dao_assets::pallet::Pallet::<Test>::reserved(asset_id, sender),
			token_deposit
		);

		// test that trying to overwrite a proposal fails
		assert_noop!(
			DaoVotes::create_proposal(origin, dao_id, prop_id, metadata, hash),
			Error::<Test>::ProposalAlreadyExists
		);
	});
}

#[test]
fn can_cast_and_remove_a_vote() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let voter = 1;
		let origin = RuntimeOrigin::signed(voter);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();

		// cannot create a vote without a proposal
		assert_noop!(
			DaoVotes::vote(origin.clone(), prop_id.clone(), None),
			Error::<Test>::ProposalDoesNotExist
		);

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));
		// preparation: set governance
		let duration = 4200;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 10; // slightly less than 1 %
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));

		let vote = true;
		let bounded_prop_id: BoundedVec<_, _> = prop_id.clone().try_into().unwrap();

		// test creating a vote
		assert!(!<Votes<Test>>::contains_key(&bounded_prop_id, voter));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id.clone(), Some(vote)));
		assert_eq!(<Votes<Test>>::get(&bounded_prop_id, voter), Some(vote));

		// test removing the same vote
		assert_ok!(DaoVotes::vote(origin, prop_id, None));
		assert!(!<Votes<Test>>::contains_key(&bounded_prop_id, voter));
	});
}

fn run_to_block(n: u64) {
	use frame_support::traits::{OnFinalize, OnInitialize};
	while System::block_number() < n {
		let mut block = System::block_number();
		Assets::on_finalize(block);
		System::on_finalize(block);
		System::reset_events();
		block += 1;
		System::set_block_number(block);
		System::on_initialize(block);
		Assets::on_initialize(block);
	}
}

#[test]
fn can_fault_a_proposal() {
	new_test_ext().execute_with(|| {
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);
		let reason = b"Bad".to_vec();
		assert_noop!(
			DaoVotes::fault_proposal(origin.clone(), prop_id.clone(), reason.clone()),
			Error::<Test>::ProposalDoesNotExist
		);
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));
		// preparation: set governance
		let duration = 4200;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 10; // slightly less than 1 %
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));

		let non_owner = RuntimeOrigin::signed(35);
		assert_noop!(
			DaoVotes::fault_proposal(non_owner, prop_id.clone(), reason.clone()),
			Error::<Test>::SenderIsNotDaoOwner,
		);

		assert_ok!(DaoVotes::fault_proposal(origin, prop_id, reason));
	})
}

#[test]
fn can_finalize_a_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);

		assert_noop!(
			DaoVotes::finalize_proposal(origin.clone(), prop_id.clone()),
			Error::<Test>::ProposalDoesNotExist
		);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));
		// preparation: set governance
		let duration = 1;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 10; // slightly less than 1 %
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));

		// cannot finalize proposal that is still running
		assert_noop!(
			DaoVotes::finalize_proposal(origin.clone(), prop_id.clone()),
			Error::<Test>::ProposalDurationHasNotPassed
		);

		let mut block = System::block_number();
		block += 1;
		run_to_block(block);
		// cannot finalize proposal that is still running
		assert_noop!(
			DaoVotes::finalize_proposal(origin.clone(), prop_id.clone()),
			Error::<Test>::ProposalDurationHasNotPassed
		);

		block += 1;
		run_to_block(block);
		assert_ok!(DaoVotes::finalize_proposal(origin, prop_id));
	})
}

#[test]
fn voting_outcome_unsuccessful_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));
		// preparation: set governance
		let duration = 0;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 0;
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));
		let voter = 2;
		assert_ok!(Assets::transfer(origin.clone(), 1, voter, 500));
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id.clone(), Some(true)));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id.clone(), Some(false)));

		let block = System::block_number() + 1 + duration as u64;
		run_to_block(block);
		assert_ok!(DaoVotes::finalize_proposal(origin, prop_id.clone()));
		let bounded_prop_id: BoundedVec<_, _> = prop_id.try_into().unwrap();
		let proposal = Proposals::<Test>::get(bounded_prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Rejected);
	})
}

#[test]
fn voting_outcome_successful_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1001));
		// preparation: set governance
		let duration = 0;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 0;
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));
		let voter = 2;
		assert_ok!(Assets::transfer(origin.clone(), 1, voter, 501));
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id.clone(), Some(true)));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id.clone(), Some(false)));

		let block = System::block_number() + 1 + duration as u64;
		run_to_block(block);
		assert_ok!(DaoVotes::finalize_proposal(origin, prop_id.clone()));

		let bounded_prop_id: BoundedVec<_, _> = prop_id.try_into().unwrap();
		let proposal = Proposals::<Test>::get(bounded_prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Accepted);
	})
}


#[test]
fn mark_accepted_proposal_as_implemented() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"PROP".to_vec();
		let origin = RuntimeOrigin::signed(1);

		assert_noop!(
			DaoVotes::mark_implemented(origin.clone(), prop_id.clone()),
			Error::<Test>::ProposalDoesNotExist
		);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1001));
		// preparation: set governance
		let duration = 0;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 0;
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));
		// preparation: create a proposal
		assert_ok!(DaoVotes::create_proposal(
			origin.clone(),
			dao_id,
			prop_id.clone(),
			metadata,
			hash
		));

		assert_noop!(
			DaoVotes::mark_implemented(origin.clone(), prop_id.clone()),
			Error::<Test>::ProposalStatusNotAccepted
		);

		let voter = 2;
		assert_ok!(Assets::transfer(origin.clone(), 1, voter, 501));
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id.clone(), Some(true)));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id.clone(), Some(false)));

		let block = System::block_number() + 1 + duration as u64;
		run_to_block(block);
		assert_ok!(DaoVotes::finalize_proposal(origin.clone(), prop_id.clone()));

		let bounded_prop_id: BoundedVec<_, _> = prop_id.clone().try_into().unwrap();
		let proposal = Proposals::<Test>::get(bounded_prop_id.clone()).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Accepted);

		assert_ok!(DaoVotes::mark_implemented(origin, prop_id));
		let proposal = Proposals::<Test>::get(bounded_prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Implemented);
	})
}
