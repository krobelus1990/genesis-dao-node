use codec::MaxEncodedLen;
use frame_support::RuntimeDebug;

use frame_support::codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<DaoId> {
	pub dao_id: DaoId,

}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Vote<ProposalId> {
	pub proposal_id: ProposalId,
}


