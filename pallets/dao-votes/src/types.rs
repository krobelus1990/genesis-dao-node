use codec::MaxEncodedLen;
use frame_support::RuntimeDebug;

use frame_support::codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<BoundedString, DaoId, AccountId> {
	pub id: BoundedString,
	pub dao_id: DaoId,
	pub creator: AccountId,
	// pub created_at: timestsamp
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Vote<BoundedString, ProposalId, AccountId> {
	pub id: BoundedString,
	pub proposal_id: ProposalId,
	pub voter: AccountId,
	pub in_favour: bool
}
