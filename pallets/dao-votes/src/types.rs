use codec::MaxEncodedLen;
use frame_support::{
	codec::{Decode, Encode},
	traits::ConstU32,
	BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<Id, DaoId, AccountId, BlockId, Balance, Metadata> {
	pub id: Id,
	pub dao_id: DaoId,
	pub creator: AccountId,
	pub birth_block: BlockId,
	pub meta: Metadata,
	pub meta_hash: BoundedVec<u8, ConstU32<64>>,
	pub status: ProposalStatus,
	pub in_favor: Balance,
	pub against: Balance,
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProposalStatus {
	#[default]
	Active,
	Counting,
	Accepted,
	Rejected,
	Faulty,
}
