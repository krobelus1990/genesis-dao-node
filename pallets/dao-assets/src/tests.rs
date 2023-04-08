//! Tests for Assets pallet.

use super::*;
use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungibles::InspectEnumerable, Currency},
};
use pallet_balances::Error as BalancesError;
use sp_runtime::TokenError;

fn asset_ids() -> Vec<u32> {
	let mut s: Vec<_> = Assets::asset_ids().collect();
	s.sort();
	s
}

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::do_mint(0, &2, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(200));
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn approval_lifecycle_works() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 40));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 60);
		assert_eq!(Assets::balance(0, 3), 40);
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn transfer_approved_all_funds() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);

		// transfer the full amount, which should trigger auto-cleanup
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 3), 50);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn approval_deposits_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		let e = BalancesError::<Test>::InsufficientBalance;
		assert_noop!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50), e);

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Balances::reserved_balance(&1), 1);

		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn cannot_transfer_more_than_approved() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		let e = Error::<Test>::Unapproved;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 51), e);
	});
}

#[test]
fn cannot_transfer_more_than_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 101));
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 101), e);
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 1, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(2), 0, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 3),
			Error::<Test>::Unknown
		);
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn lifecycle_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		let asset_id = 37;
		assert_ok!(Assets::do_force_create(asset_id, 1, 1));
		assert!(Asset::<Test>::contains_key(asset_id));

		assert_ok!(Assets::set_metadata(RuntimeOrigin::signed(1), asset_id, vec![0], vec![0], 12));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert!(Metadata::<Test>::contains_key(asset_id));

		Balances::make_free_balance_be(&10, 100);
		assert_ok!(Assets::do_mint(asset_id, &10, 100));
		Balances::make_free_balance_be(&20, 100);
		assert_ok!(Assets::do_mint(asset_id, &20, 100));
		assert_eq!(Account::<Test>::iter_prefix(asset_id).count(), 2);

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), asset_id));

		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(Asset::<Test>::get(asset_id).unwrap().status == AssetStatus::Destroyed);
		assert!(!Metadata::<Test>::contains_key(asset_id));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);
	});
}

#[test]
fn destroy_should_refund_approvals() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &10, 100));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 3, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 4, 50));
		assert_eq!(Balances::reserved_balance(&1), 3);
		assert_eq!(asset_ids(), vec![0, 999]);

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		assert_eq!(Balances::reserved_balance(&1), 0);

		// all approvals are removed
		assert!(Approvals::<Test>::iter().count().is_zero())
	});
}

#[test]
fn partial_destroy_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_ok!(Assets::do_mint(0, &2, 100));
		assert_ok!(Assets::do_mint(0, &3, 100));
		assert_ok!(Assets::do_mint(0, &4, 100));
		assert_ok!(Assets::do_mint(0, &5, 100));
		assert_ok!(Assets::do_mint(0, &6, 100));
		assert_ok!(Assets::do_mint(0, &7, 100));

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		// Asset is in use, as all the accounts have not yet been destroyed.
		// We need to call destroy_accounts or destroy_approvals again until asset is completely
		// cleaned up.
		assert_noop!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0), Error::<Test>::InUse);

		System::assert_has_event(RuntimeEvent::Assets(crate::Event::AccountsDestroyed {
			asset_id: 0,
			accounts_destroyed: 5,
			accounts_remaining: 2,
		}));
		System::assert_has_event(RuntimeEvent::Assets(crate::Event::ApprovalsDestroyed {
			asset_id: 0,
			approvals_destroyed: 0,
			approvals_remaining: 0,
		}));
		// Partially destroyed Asset should continue to exist
		assert!(Asset::<Test>::contains_key(0));

		// Second call to destroy on PartiallyDestroyed asset
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		System::assert_has_event(RuntimeEvent::Assets(crate::Event::AccountsDestroyed {
			asset_id: 0,
			accounts_destroyed: 2,
			accounts_remaining: 0,
		}));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		System::assert_has_event(RuntimeEvent::Assets(crate::Event::Destroyed { asset_id: 0 }));

		// Destroyed Asset should not exist
		assert!(Asset::<Test>::get(0).unwrap().status == AssetStatus::Destroyed);
	})
}

#[test]
fn min_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 10));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Cannot create a new account with a balance that is below minimum...
		assert_noop!(Assets::do_mint(0, &2, 9), TokenError::BelowMinimum);
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 9), TokenError::BelowMinimum);

		// When deducting from an account to below minimum, it should be reaped.
		// Death by `transfer`.
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 91));
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `force_transfer`
		let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
		let _ = Assets::do_transfer(0, &2, &1, 91, f).map(|_| ());
		assert!(Assets::maybe_balance(0, 2).is_none());
		assert_eq!(Assets::balance(0, 1), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `burn`.
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, 91, flags);
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 0);

		// Death by `transfer_approved`.
		assert_ok!(Assets::do_mint(0, &1, 100));

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 91));
	});
}

#[test]
fn querying_total_supply_should_work() {
	let asset_id = 7;
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(asset_id, 1, 1));
		assert_ok!(Assets::do_mint(asset_id, &1, 100));
		assert_eq!(Assets::balance(asset_id, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), asset_id, 2, 50));
		assert_eq!(Assets::balance(asset_id, 1), 50);
		assert_eq!(Assets::balance(asset_id, 2), 50);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), asset_id, 3, 31));
		assert_eq!(Assets::balance(asset_id, 1), 50);
		assert_eq!(Assets::balance(asset_id, 2), 19);
		assert_eq!(Assets::balance(asset_id, 3), 31);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(asset_id, &3, u64::MAX, flags);
		assert_eq!(Assets::total_supply(asset_id), 69);
	});
}

#[test]
fn transferring_amount_below_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
	});
}

#[test]
fn transferring_enough_to_kill_source_when_keep_alive_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 10));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 91),
			Error::<Test>::BalanceLow
		);
		assert_ok!(Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 90));
		assert_eq!(Assets::balance(0, 1), 10);
		assert_eq!(Assets::balance(0, 2), 90);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn origin_guards_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_noop!(
			Assets::start_destroy(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, flags);
		assert_eq!(Assets::balance(0, 1), 0);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 1, 50),
			Error::<Test>::NoAccount
		);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(2), 0, 1, 51),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn transferring_less_than_one_unit_is_fine() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 0));
		// `ForceCreated` and `Issued` but no `Transferred` event.
		assert_eq!(System::events().len(), 2);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 101),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn burning_asset_balance_with_positive_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, flags);
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_eq!(Assets::balance(0, 1), 0);
	});
}

#[test]
fn burning_asset_balance_with_zero_balance_does_nothing() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 2), 0);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		assert_noop!(Assets::do_burn(0, &2, u64::MAX, flags), Error::<Test>::NoAccount);
		assert_eq!(Assets::balance(0, 2), 0);
		assert_eq!(Assets::total_supply(0), 100);
	});
}

#[test]
fn set_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// Cannot add metadata to unknown asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::Unknown,
		);
		assert_ok!(Assets::do_force_create(0, 1, 1));
		// Cannot add metadata to unowned asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(2), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::NoPermission,
		);

		// Cannot add oversized metadata
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 100], vec![0u8; 10], 12),
			Error::<Test>::BadMetadata,
		);
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 100], 12),
			Error::<Test>::BadMetadata,
		);

		// Successfully add metadata
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			12
		));

		// Clear Metadata
		assert!(Metadata::<Test>::contains_key(0));
		assert_noop!(
			Assets::clear_metadata(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::clear_metadata(RuntimeOrigin::signed(1), 1), Error::<Test>::Unknown);
		assert_ok!(Assets::clear_metadata(RuntimeOrigin::signed(1), 0));
		assert!(!Metadata::<Test>::contains_key(0));
	});
}

/// Destroying an asset works
#[test]
fn finish_destroy_asset_destroys_asset() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 50));
		// Destroy the accounts.
		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		// Asset is gone
		assert!(Asset::<Test>::get(0).unwrap().status == AssetStatus::Destroyed);
	})
}

#[test]
fn imbalances_should_work() {
	use frame_support::traits::tokens::fungibles::Balanced;

	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));

		let imb = Assets::issue(0, 100);
		assert_eq!(Assets::total_supply(0), 100);
		assert_eq!(imb.peek(), 100);

		let (imb1, imb2) = imb.split(30);
		assert_eq!(imb1.peek(), 30);
		assert_eq!(imb2.peek(), 70);

		drop(imb2);
		assert_eq!(Assets::total_supply(0), 30);

		assert!(Assets::resolve(&1, imb1).is_ok());
		assert_eq!(Assets::balance(0, 1), 30);
		assert_eq!(Assets::total_supply(0), 30);
	});
}

#[test]
fn assets_from_genesis_should_exist() {
	new_test_ext().execute_with(|| {
		assert_eq!(asset_ids(), vec![999]);
		assert!(Metadata::<Test>::contains_key(999));
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::total_supply(999), 100);
	});
}

#[test]
fn querying_allowance_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::approvals::{Inspect, Mutate};
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve(0, &1, &2, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 50);
		// Transfer asset 0, from owner 1 and delegate 2 to destination 3
		assert_ok!(Assets::transfer_from(0, &1, &2, &3, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 0);
	});
}

#[test]
fn transfer_large_asset() {
	new_test_ext().execute_with(|| {
		let amount = u64::pow(2, 63) + 2;
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, amount));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, amount - 1));
	})
}

#[test]
fn reserving_and_unreserving_should_work() {
	new_test_ext().execute_with(|| {
		// establish situation before
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::reserved(999, 1), 0);

		// do reservation
		assert_ok!(Assets::do_reserve(999, 1, 40));

		// check reservation worked
		assert_eq!(Assets::balance(999, 1), 60);
		assert_eq!(Assets::reserved(999, 1), 40);

		// undo reservation
		assert_ok!(Assets::do_unreserve(999, 1, 10));

		// check undoing reservation worked
		assert_eq!(Assets::balance(999, 1), 70);
		assert_eq!(Assets::reserved(999, 1), 30);

		// undo the maximum
		assert_ok!(Assets::do_unreserve(999, 1, 1500));

		// check undoing reservation worked
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::reserved(999, 1), 0);
	})
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
fn query_historic_blocks_should_work() {
	new_test_ext().execute_with(|| {
		let asset_id = 95;
		let account_id = 32;
		let account_id2 = 974;
		let amount = 345;
		let amount2 = 9287734;
		let transfer1 = 98;
		let transfer2 = 629984;
		let burn_amount = 127;
		let block0 = System::block_number();

		let history = |block, supply, account, account2| {
			assert_eq!(
				Assets::total_historical_supply(asset_id, block),
				Some(supply),
				"wrong supply"
			);
			assert_eq!(
				Assets::total_historical_balance(asset_id, &account_id, block),
				Some(account),
				"wrong balance for first account"
			);
			assert_eq!(
				Assets::total_historical_balance(asset_id, &account_id2, block),
				Some(account2),
				"wrong balance for second account"
			);
		};
		let history_block0 = || {
			history(block0, 0, 0, 0);
		};
		history_block0();

		// create asset
		assert_ok!(Assets::do_force_create(asset_id, account_id, 1));
		history_block0();

		// advance to block1
		println!("advancing to block1");
		let block1 = block0 + 3;
		let check1 = block0 + 1;
		run_to_block(block1);
		history_block0();
		history(block1, 0, 0, 0);

		// mint into first account
		println!("mint into first account");
		assert_ok!(Assets::do_mint(asset_id, &account_id, amount));

		// history is up to date
		let history_block1 = || {
			history(block1, amount, amount, 0);
		};
		history_block1();

		// history is preserved
		let history_check1 = || {
			history(check1, 0, 0, 0);
		};
		history_check1();
		history_block0();

		// advance to block2
		println!("advancing to block2");
		let block2 = block1 + 5;
		let check2 = block1 + 3;
		run_to_block(block2);
		history_block0();
		history_check1();
		history_block1();
		history(block2, amount, amount, 0);

		// mint into second account
		println!("mint into second account");
		assert_ok!(Assets::do_mint(asset_id, &account_id2, amount2));

		// history is up to date
		let history_block2 = || {
			history(block2, amount + amount2, amount, amount2);
		};
		history_block2();

		// history is preserved
		let history_check2 = || {
			history(check2, amount, amount, 0);
		};
		history_check2();
		history_block1();
		history_check1();
		history_block0();

		// advance to block3
		println!("advancing to block3");
		let block3 = block2 + 8;
		let check3 = block2 + 2;
		run_to_block(block3);
		history_block0();
		history_check1();
		history_block1();
		history_check2();
		history_block2();
		history(block3, amount + amount2, amount, amount2);

		// transfer from first account to second account
		println!("transfer from first account to second account");
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(account_id),
			asset_id,
			account_id2,
			transfer1
		));

		// history is up to date
		let history_block3 = || {
			history(block3, amount + amount2, amount - transfer1, amount2 + transfer1);
		};
		history_block3();

		// history is preserved
		let history_check3 = || {
			history(check3, amount + amount2, amount, amount2);
		};
		history_check3();
		history_block2();
		history_check2();
		history_block1();
		history_check1();
		history_block0();

		// advance to block4
		println!("advancing to block4");
		let block4 = block3 + 7;
		let check4 = block3 + 6;
		run_to_block(block4);
		history_block0();
		history_check1();
		history_block1();
		history_check2();
		history_block2();
		history_check3();
		history_block3();
		history(block4, amount + amount2, amount - transfer1, amount2 + transfer1);

		// transfer from second account to first account
		println!("transfer from second account to first account");
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(account_id2),
			asset_id,
			account_id,
			transfer2
		));

		// history is up to date
		let history_block4 = || {
			history(
				block4,
				amount + amount2,
				amount - transfer1 + transfer2,
				amount2 + transfer1 - transfer2,
			);
		};
		history_block4();

		// history is preserved
		let history_check4 = || {
			history(check4, amount + amount2, amount - transfer1, amount2 + transfer1);
		};
		history_check4();
		history_block3();
		history_check3();
		history_block2();
		history_check2();
		history_block1();
		history_check1();
		history_block0();

		// advance to block5
		println!("advancing to block5");
		let block5 = block4 + 4;
		let check5 = block4 + 1;
		run_to_block(block5);
		history_block0();
		history_check1();
		history_block1();
		history_check2();
		history_block2();
		history_check3();
		history_block3();
		history_check4();
		history_block4();
		history(
			block5,
			amount + amount2,
			amount - transfer1 + transfer2,
			amount2 + transfer1 - transfer2,
		);

		// burn from first account
		println!("burn from first account");
		let flags = DebitFlags { keep_alive: false, best_effort: false };
		assert_ok!(Assets::do_burn(asset_id, &account_id, burn_amount, flags));

		// history is up to date
		let history_block5 = || {
			history(
				block5,
				amount + amount2 - burn_amount,
				amount - transfer1 + transfer2 - burn_amount,
				amount2 + transfer1 - transfer2,
			);
		};
		history_block5();

		// history is preserved
		let history_check5 = || {
			history(
				check5,
				amount + amount2,
				amount - transfer1 + transfer2,
				amount2 + transfer1 - transfer2,
			);
		};
		history_check5();
		history_block4();
		history_check4();
		history_block3();
		history_check3();
		history_block2();
		history_check2();
		history_block1();
		history_check1();
		history_block0();
	})
}
