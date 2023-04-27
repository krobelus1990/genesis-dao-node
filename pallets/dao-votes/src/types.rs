use codec::MaxEncodedLen;
use frame_support::{
	codec::{Decode, Encode},
	traits::ConstU32,
	BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProposalSlot<DaoId, AccountId> {
	pub dao_id: DaoId,
	pub creator: AccountId,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<DaoId, AccountId, BlockId, Balance, Metadata> {
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
	Running,
	Counting,
	Accepted,
	Rejected,
	Faulty,
	Implemented,
}
