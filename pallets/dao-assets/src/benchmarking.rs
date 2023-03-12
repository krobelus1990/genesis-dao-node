//! Assets pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::{EnsureOrigin, Get};
use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::traits::Bounded;

use crate::Pallet as Assets;

const SEED: u32 = 0;

fn default_asset_id<T: Config>() -> T::AssetIdParameter {
	T::BenchmarkHelper::create_asset_id_parameter(0)
}

fn create_default_asset<T: Config>(
	is_sufficient: bool,
) -> (T::AssetIdParameter, T::AccountId) {
	let asset_id = default_asset_id::<T>();
	let caller: T::AccountId = whitelisted_caller();
	assert!(Assets::<T>::do_force_create(
		asset_id.into(),
		caller,
		is_sufficient,
		1u32.into(),
	)
	.is_ok());
	(asset_id, caller)
}

fn create_default_minted_asset<T: Config>(
	is_sufficient: bool,
	amount: T::Balance,
) -> (T::AssetIdParameter, T::AccountId) {
	let (asset_id, caller) = create_default_asset::<T>(is_sufficient);
	if !is_sufficient {
		T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance());
	}
	assert!(Assets::<T>::do_mint(
		asset_id.into(),
		&caller,
		amount,
		Some(caller)
	)
	.is_ok());
	(asset_id, caller)
}

fn swap_is_sufficient<T: Config>(s: &mut bool) {
	let asset_id = default_asset_id::<T>();
	Asset::<T>::mutate(&asset_id.into(), |maybe_a| {
		if let Some(ref mut a) = maybe_a {
			sp_std::mem::swap(s, &mut a.is_sufficient)
		}
	});
}

fn add_sufficients<T: Config>(minter: T::AccountId, n: u32) {
	let asset_id = default_asset_id::<T>();
	let origin = SystemOrigin::Signed(minter);
	let mut s = true;
	swap_is_sufficient::<T>(&mut s);
	for i in 0..n {
		let target = account("sufficient", i, SEED);
		let target_lookup = T::Lookup::unlookup(target);
		assert!(Assets::<T>::mint(origin.clone().into(), asset_id, target_lookup, 100u32.into())
			.is_ok());
	}
	swap_is_sufficient::<T>(&mut s);
}

fn add_approvals<T: Config>(minter: T::AccountId, n: u32) {
	let asset_id = default_asset_id::<T>();
	T::Currency::deposit_creating(&minter, T::ApprovalDeposit::get() * n.into());
	let minter_lookup = T::Lookup::unlookup(minter.clone());
	let origin = SystemOrigin::Signed(minter);
	Assets::<T>::mint(origin.clone().into(), asset_id, minter_lookup, (100 * (n + 1)).into())
		.unwrap();
	for i in 0..n {
		let target = account("approval", i, SEED);
		T::Currency::make_free_balance_be(&target, T::Currency::minimum_balance());
		let target_lookup = T::Lookup::unlookup(target);
		Assets::<T>::approve_transfer(
			origin.clone().into(),
			asset_id,
			target_lookup,
			100u32.into(),
		)
		.unwrap();
	}
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn assert_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {
	create {
		let asset_id = default_asset_id::<T>();
		let origin = T::CreateOrigin::successful_origin(&asset_id.into());
		let caller = T::CreateOrigin::ensure_origin(origin, &asset_id.into()).unwrap();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, caller_lookup, 1u32.into())
	verify {
		assert_last_event::<T>(Event::Created { asset_id: asset_id.into(), creator: caller.clone(), owner: caller }.into());
	}

	force_create {
		let asset_id = default_asset_id::<T>();
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
	}: _(SystemOrigin::Root, asset_id, caller_lookup, true, 1u32.into())
	verify {
		assert_last_event::<T>(Event::ForceCreated { asset_id: asset_id.into(), owner: caller }.into());
	}

	start_destroy {
		let (asset_id, caller) = create_default_minted_asset::<T>(true, 100u32.into());
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::DestructionStarted { asset_id: asset_id.into() }.into());
	}

	destroy_accounts {
		let c in 0 .. T::RemoveItemsLimit::get();
		let (asset_id, caller) = create_default_asset::<T>(true);
		add_sufficients::<T>(caller.clone(), c);
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::AccountsDestroyed {
			asset_id: asset_id.into(),
			accounts_destroyed: c,
			accounts_remaining: 0,
		}.into());
	}

	destroy_approvals {
		let a in 0 .. T::RemoveItemsLimit::get();
		let (asset_id, caller) = create_default_minted_asset::<T>(true, 100u32.into());
		add_approvals::<T>(caller.clone(), a);
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::ApprovalsDestroyed {
			asset_id: asset_id.into(),
			approvals_destroyed: a,
			approvals_remaining: 0,
		}.into());
	}

	finish_destroy {
		let (asset_id, caller) = create_default_asset::<T>(true);
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::Destroyed {
			asset_id: asset_id.into(),
		}.into()
		);
	}

	mint {
		let (asset_id, caller) = create_default_asset::<T>(true);
		let amount = T::Balance::from(100u32);
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, amount)
	verify {
		assert_last_event::<T>(Event::Issued { asset_id: asset_id.into(), owner: caller, total_supply: amount }.into());
	}

	burn {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(true, amount);
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, amount)
	verify {
		assert_last_event::<T>(Event::Burned { asset_id: asset_id.into(), owner: caller, balance: amount }.into());
	}

	transfer {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(true, amount);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target_lookup, amount)
	verify {
		assert_last_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: caller, to: target, amount }.into());
	}

	transfer_keep_alive {
		let mint_amount = T::Balance::from(200u32);
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(true, mint_amount);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target_lookup, amount)
	verify {
		assert!(frame_system::Pallet::<T>::account_exists(&caller));
		assert_last_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: caller, to: target, amount }.into());
	}

	force_transfer {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(true, amount);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target_lookup, amount)
	verify {
		assert_last_event::<T>(
			Event::Transferred { asset_id: asset_id.into(), from: caller, to: target, amount }.into()
		);
	}

	transfer_ownership {
		let (asset_id, caller) = create_default_asset::<T>(true);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller), asset_id, target_lookup)
	verify {
		assert_last_event::<T>(Event::OwnerChanged { asset_id: asset_id.into(), owner: target }.into());
	}

	set_team {
		let (asset_id, caller) = create_default_asset::<T>(true);
		let target0 = T::Lookup::unlookup(account("target", 0, SEED));
		let target1 = T::Lookup::unlookup(account("target", 1, SEED));
	}: _(SystemOrigin::Signed(caller), asset_id, target0, target1)
	verify {
		assert_last_event::<T>(Event::TeamChanged {
			asset_id: asset_id.into(),
			issuer: account("target", 0, SEED),
			admin: account("target", 1, SEED),
		}.into());
	}

	set_metadata {
		let n in 0 .. T::StringLimit::get();
		let s in 0 .. T::StringLimit::get();

		let name = vec![0u8; n as usize];
		let symbol = vec![0u8; s as usize];
		let decimals = 12;

		let (asset_id, caller) = create_default_asset::<T>(true);
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
	}: _(SystemOrigin::Signed(caller), asset_id, name.clone(), symbol.clone(), decimals)
	verify {
		assert_last_event::<T>(Event::MetadataSet { asset_id: asset_id.into(), name, symbol, decimals }.into());
	}

	clear_metadata {
		let (asset_id, caller) = create_default_asset::<T>(true);
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
		let dummy = vec![0u8; T::StringLimit::get() as usize];
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Assets::<T>::set_metadata(origin, asset_id, dummy.clone(), dummy, 12)?;
	}: _(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::MetadataCleared { asset_id: asset_id.into() }.into());
	}

	force_set_metadata {
		let n in 0 .. T::StringLimit::get();
		let s in 0 .. T::StringLimit::get();

		let name = vec![0u8; n as usize];
		let symbol = vec![0u8; s as usize];
		let decimals = 12;

		let (asset_id, _) = create_default_asset::<T>(true);

		let origin = T::ForceOrigin::successful_origin();
		let call = Call::<T>::force_set_metadata {
			id: asset_id,
			name: name.clone(),
			symbol: symbol.clone(),
			decimals,
		};
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_last_event::<T>(Event::MetadataSet { asset_id: asset_id.into(), name, symbol, decimals }.into());
	}

	force_clear_metadata {
		let (asset_id, caller) = create_default_asset::<T>(true);
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
		let dummy = vec![0u8; T::StringLimit::get() as usize];
		let origin = SystemOrigin::Signed(caller).into();
		Assets::<T>::set_metadata(origin, asset_id, dummy.clone(), dummy, 12)?;

		let origin = T::ForceOrigin::successful_origin();
		let call = Call::<T>::force_clear_metadata { id: asset_id };
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_last_event::<T>(Event::MetadataCleared { asset_id: asset_id.into() }.into());
	}

	approve_transfer {
		let (asset_id, caller) = create_default_minted_asset::<T>(true, 100u32.into());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());

		let delegate: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
		let amount = 100u32.into();
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, delegate_lookup, amount)
	verify {
		assert_last_event::<T>(Event::ApprovedTransfer { asset_id: asset_id.into(), source: caller, delegate, amount }.into());
	}

	transfer_approved {
		let (asset_id, owner) = create_default_minted_asset::<T>(true, 100u32.into());
		let owner_lookup = T::Lookup::unlookup(owner.clone());
		T::Currency::make_free_balance_be(&owner, DepositBalanceOf::<T>::max_value());

		let delegate: T::AccountId = account("delegate", 0, SEED);
		whitelist_account!(delegate);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
		let amount = 100u32.into();
		let origin = SystemOrigin::Signed(owner.clone()).into();
		Assets::<T>::approve_transfer(origin, asset_id, delegate_lookup, amount)?;

		let dest: T::AccountId = account("dest", 0, SEED);
		let dest_lookup = T::Lookup::unlookup(dest.clone());
	}: _(SystemOrigin::Signed(delegate.clone()), asset_id, owner_lookup, dest_lookup, amount)
	verify {
		assert!(T::Currency::reserved_balance(&owner).is_zero());
		assert_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: owner, to: dest, amount }.into());
	}

	cancel_approval {
		let (asset_id, caller) = create_default_minted_asset::<T>(true, 100u32.into());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());

		let delegate: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
		let amount = 100u32.into();
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Assets::<T>::approve_transfer(origin, asset_id, delegate_lookup.clone(), amount)?;
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, delegate_lookup)
	verify {
		assert_last_event::<T>(Event::ApprovalCancelled { asset_id: asset_id.into(), owner: caller, delegate }.into());
	}

	force_cancel_approval {
		let (asset_id, caller) = create_default_minted_asset::<T>(true, 100u32.into());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());

		let delegate: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
		let amount = 100u32.into();
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Assets::<T>::approve_transfer(origin, asset_id, delegate_lookup.clone(), amount)?;
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, delegate_lookup)
	verify {
		assert_last_event::<T>(Event::ApprovalCancelled { asset_id: asset_id.into(), owner: caller, delegate }.into());
	}

	impl_benchmark_test_suite!(Assets, crate::mock::new_test_ext(), crate::mock::Test)
}
