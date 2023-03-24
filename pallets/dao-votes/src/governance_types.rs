use codec::MaxEncodedLen;
use frame_support::{codec::{Decode, Encode}, RuntimeDebug};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Governance<Balance> {
	// the number of blocks a proposal is open for voting
	pub proposal_duration: u32,
	// the token deposit required to create a proposal
	pub proposal_token_deposit: Balance,
	// the rules for accepting proposals
	pub voting: Voting,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Voting {
	Majority {
		// how many more ayes than nays there must be for proposal acceptance
		// thus proposal acceptance requires: ayes >= nays + token_supply / 1024 * minimum_majority_per_1024
		minimum_majority_per_1024: u8,
	},
}
