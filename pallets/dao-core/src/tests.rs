use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_balances::Error as BalancesError;

#[test]
fn dao_id_valid_chars() {
	new_test_ext().execute_with(|| {
		for b in 0u8..255u8 {
			let mut id = b"GDAO".to_vec();
			id.push(b);
			match b {
				b'A'..=b'Z' | b'0'..=b'9' => {
					assert_ok!(DaoCore::create_dao(
						RuntimeOrigin::signed(1),
						id,
						b"Genesis DAO".to_vec()
					));
				},
				_ => {
					assert_noop!(
						DaoCore::create_dao(RuntimeOrigin::signed(1), id, b"Genesis DAO".to_vec()),
						Error::<Test>::DaoIdInvalidChar
					);
				},
			}
		}
	})
}

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

		assert_eq!(Balances::free_balance(1), 1000);
		assert_ok!(DaoCore::create_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec(), b"Genesis DAO".to_vec()));
		// reserve taken
		assert_eq!(Balances::free_balance(1), 990);

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

		assert_ok!(DaoCore::create_dao(
			RuntimeOrigin::signed(1),
			b"GDAO".to_vec(),
			b"Genesis DAO".to_vec()
		));

		assert_noop!(
			DaoCore::destroy_dao(RuntimeOrigin::signed(2), b"GDAO".to_vec()),
			Error::<Test>::DaoSignerNotOwner
		);

		assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"GDAO".to_vec(), 1000));

		assert_noop!(
			DaoCore::destroy_dao(RuntimeOrigin::signed(1), b"GDAO".to_vec()),
			Error::<Test>::DaoTokenAlreadyIssued
		);

		let dao = DaoCore::load_dao(b"GDAO".to_vec()).unwrap();
		let asset_id = dao.asset_id.unwrap();
		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), asset_id));

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
		assert_ok!(DaoCore::create_dao(
			RuntimeOrigin::signed(1),
			b"GDAO".to_vec(),
			b"Genesis DAO".to_vec()
		));
		assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"GDAO".to_vec(), 1000));

		let dao = DaoCore::load_dao(b"GDAO".to_vec()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		assert_eq!(asset_id, 1);

		use frame_support::traits::tokens::fungibles::metadata::Inspect;
		assert_eq!(Assets::name(asset_id), b"Genesis DAO".to_vec());
		assert_eq!(Assets::symbol(asset_id), b"GDAO".to_vec());
		assert_eq!(Assets::decimals(asset_id), 9);

		// payout
		assert_eq!(Assets::balance(asset_id, &dao.owner), 1000);

		// increment
		assert_ok!(DaoCore::create_dao(
			RuntimeOrigin::signed(1),
			b"GDAO2".to_vec(),
			b"Genesis DAO 2".to_vec()
		));
		assert_ok!(DaoCore::issue_token(RuntimeOrigin::signed(1), b"GDAO2".to_vec(), 1000));

		let dao2 = DaoCore::load_dao(b"GDAO2".to_vec()).unwrap();
		let asset_id2 = dao2.asset_id.unwrap();
		assert_eq!(asset_id2, 2);
	});
}
