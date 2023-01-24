use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_creates_a_proposal() {
	new_test_ext().execute_with(|| {
		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(
			RuntimeOrigin::signed(1),
			b"TEST".to_vec(),
			b"TEST DAO".to_vec()
		));
		// preparation: issue token
		assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"TEST".to_vec(), 1000));

		// actual test
		assert_ok!(DaoVotes::create_proposal(
			RuntimeOrigin::signed(1),
			b"TEST".to_vec(),
			b"TEST_proposal".to_vec()
		));

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
