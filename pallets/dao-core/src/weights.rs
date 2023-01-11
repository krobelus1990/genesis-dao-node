// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_dao_core
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-11, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/genesis-dao
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_dao_core
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// pallets/dao-core/src/weights.rs
// --template
// ./benchmarking/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_dao_core.
pub trait WeightInfo {
	#[rustfmt::skip]
	fn create_dao() -> Weight;
	#[rustfmt::skip]
	fn destroy_dao() -> Weight;
	#[rustfmt::skip]
	fn issue_token() -> Weight;
}

/// Weights for pallet_dao_core using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: DaoCore Daos (r:1 w:1)
	#[rustfmt::skip]
	fn create_dao() -> Weight {
		Weight::from_ref_time(182_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: DaoCore Daos (r:1 w:1)
	#[rustfmt::skip]
	fn destroy_dao() -> Weight {
		Weight::from_ref_time(314_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: DaoCore Daos (r:1 w:1)
	// Storage: DaoCore CurrentAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	#[rustfmt::skip]
	fn issue_token() -> Weight {
		Weight::from_ref_time(428_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: DaoCore Daos (r:1 w:1)
	#[rustfmt::skip]
	fn create_dao() -> Weight {
		Weight::from_ref_time(182_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: DaoCore Daos (r:1 w:1)
	#[rustfmt::skip]
	fn destroy_dao() -> Weight {
		Weight::from_ref_time(314_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: DaoCore Daos (r:1 w:1)
	// Storage: DaoCore CurrentAssetId (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:1)
	#[rustfmt::skip]
	fn issue_token() -> Weight {
		Weight::from_ref_time(428_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
}