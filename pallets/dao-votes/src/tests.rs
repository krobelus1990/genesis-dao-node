use crate::{mock::*};

#[test]
fn it_creates_a_vote() {
	new_test_ext().execute_with(|| {

		DaoVotes::create_vote(RuntimeOrigin::signed(1));
	});
}
