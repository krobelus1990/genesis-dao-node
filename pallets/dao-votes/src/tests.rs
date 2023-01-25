use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_dao_core::Error as DaoError;

#[test]
fn it_creates_a_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"TEST".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let prop_id = b"TEST_proposal".to_vec();

		// cannot create a proposal without a DAO
		assert_noop!(
			DaoVotes::create_proposal(RuntimeOrigin::signed(1), dao_id.clone(), prop_id.clone()),
			DaoError::<Test>::DaoDoesNotExist
		);

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), dao_id.clone(), dao_name));

		// cannot create a proposal without DAO tokens existing (because they need to be reserved)
		assert_noop!(
			DaoVotes::create_proposal(RuntimeOrigin::signed(1), dao_id.clone(), prop_id.clone()),
			Error::<Test>::DaoTokenNotYetIssued
		);

		// preparation: issue token
		assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), dao_id.clone(), 1000));

		// test creating a proposal
		assert_ok!(DaoVotes::create_proposal(RuntimeOrigin::signed(1), dao_id, prop_id));

		// creating a proposal should reserve DAO tokens
		//assert_eq!(Balances::reserved_balance(1), 2);
	});
}

#[test]
fn it_creates_a_vote() {
	new_test_ext().execute_with(|| {
		// assert_ok!(DaoVotes::create_vote(
		// 	RuntimeOrigin::signed(1),
		// 	b"TEST_proposal".to_vec(),
		// 	true
		// ));
	});
}
