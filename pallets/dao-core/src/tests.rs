use crate::{mock::*, Error};
use frame_support::{
    assert_ok, assert_noop
};
use pallet_balances::Error as BalancesError;


#[test]
fn it_creates_a_dao() {
	new_test_ext().execute_with(|| {

		assert_noop!(
				DaoCore::create_dao(RuntimeOrigin::signed(1), b"XX".to_vec(), b"Genesis DAO".to_vec()),
			Error::<Test>::DaoIdInvalidLengthTooShort
		);

		assert_noop!(
				DaoCore::create_dao(RuntimeOrigin::signed(1), b"XKETGUNIQHQKYYQ7JRQLC7SH02LY7WI27DXOZQURA0M4Z2MRI11L5UZ0DMVRDREQ9TFQI530UB3Z7ZOMMZ10HXAA9TKBMBC1ETFCXHP6HI9G0UXX8SAIYQ0JZI1R1CH9CU7FZHDN50SB3DWMQVD0DAG1BPD52COUUS8JBQSMYOKJDDXU5LMXHELVG5DNZFXKDWSI8XFY605ZLZZAV34OBVUL5770GKF5DS96E0UIPC80IW65E4F7VILBURIIO87CP".to_vec(), b"Genesis DAO".to_vec()),
			Error::<Test>::DaoIdInvalidLengthTooLong
		);

		assert_noop!(
				DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"GD".to_vec()),
			Error::<Test>::DaoNameInvalidLengthTooShort
		);

		assert_noop!(
				DaoCore::create_dao(RuntimeOrigin::signed(1),  b"GDAO".to_vec(), b"XKETGUNIQHQKYYQ7JRQLC7SH02LY7WI27DXOZQURA0M4Z2MRI11L5UZ0DMVRDREQ9TFQI530UB3Z7ZOMMZ10HXAA9TKBMBC1ETFCXHP6HI9G0UXX8SAIYQ0JZI1R1CH9CU7FZHDN50SB3DWMQVD0DAG1BPD52COUUS8JBQSMYOKJDDXU5LMXHELVG5DNZFXKDWSI8XFY605ZLZZAV34OBVUL5770GKF5DS96E0UIPC80IW65E4F7VILBURIIO87CP".to_vec()),
			Error::<Test>::DaoNameInvalidLengthTooLong
		);

		assert_eq!(Balances::free_balance(1), 100);
		assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"Genesis DAO".to_vec()));
		// reserve taken
		assert_eq!(Balances::free_balance(1), 90);

		assert_noop!(
			DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"Genesis DAO".to_vec()),
			Error::<Test>::DaoAlreadyExists
		);

		// reserve insufficient fails
		assert_noop!(
			DaoCore::create_dao(RuntimeOrigin::signed(2), b"GDAO2".to_vec(), b"Genesis DAO 2".to_vec()),
			BalancesError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn it_destroys_a_dao() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DaoCore::destroy_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec()),
			Error::<Test>::DaoDoesNotExist
		);

		assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"Genesis DAO".to_vec()));

		assert_noop!(
			DaoCore::destroy_dao(RuntimeOrigin::signed(2), b"GDAO".to_vec()),
			Error::<Test>::DaoSignerNotOwner
		);

		assert_ok!(DaoCore::destroy_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec()));

		assert_noop!(
			DaoCore::destroy_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec()),
			Error::<Test>::DaoDoesNotExist
		);
	});
}

#[test]
fn issues_a_token() {
    new_test_ext().execute_with(|| {
        assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"Genesis DAO".to_vec()));
        assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"GDAO".to_vec(), 1000));

        let dao = DaoCore::load_dao(b"GDAO".to_vec()).unwrap();
        let asset_id = dao.asset_id.unwrap();

        assert_eq!(asset_id, 1);

        use frame_support::traits::tokens::fungibles::metadata::Inspect;
        assert_eq!(Assets::name(asset_id), b"Genesis DAO".to_vec());
        assert_eq!(Assets::symbol(asset_id), b"GDAO".to_vec());
        assert_eq!(Assets::decimals(asset_id), 9);

		// payout
        assert_eq!(Assets::balance(asset_id.clone(), &dao.owner), 1000);

        // increment
        assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO2".to_vec(), b"Genesis DAO 2".to_vec()));
        assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"GDAO2".to_vec(), 1000));

        let dao2 = DaoCore::load_dao(b"GDAO2".to_vec()).unwrap();
        let asset_id2 = dao2.asset_id.unwrap();
        assert_eq!(asset_id2, 2);
	});
}