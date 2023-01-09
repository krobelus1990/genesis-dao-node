//! Money matters.
pub mod currency {

	pub type Balance = u128;

	pub const UNITS: Balance = 1_000_000_000_000;

	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as u128 * 15 * CENTS + (bytes as u128) * 6 * CENTS
	}
}
