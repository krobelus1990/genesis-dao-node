use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_creates_a_proposal() {
	new_test_ext().execute_with(|| {
		// assert_ok!(DaoVotes::create_proposal(
		// 	RuntimeOrigin::signed(1),
		// 	b"TEST_DAO".to_vec(),
		// 	b"TEST_proposal".to_vec()
		// ));
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
