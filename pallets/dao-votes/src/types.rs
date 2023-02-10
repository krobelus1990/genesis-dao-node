use codec::MaxEncodedLen;
use frame_support::{traits::ConstU32, BoundedVec, RuntimeDebug};

use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<Id, DaoId, AccountId, BlockId, Metadata> {
	pub id: Id,
	pub dao_id: DaoId,
	pub creator: AccountId,
	pub birth_block: BlockId,
	pub meta: Metadata,
	pub meta_hash: BoundedVec<u8, ConstU32<64>>,
	pub status: ProposalStatus,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Vote<AccountId> {
	pub voter: AccountId,
	pub aye: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProposalStatus {
	#[default]
	Active,
	Success,
	Faulty,
	Rejected,
}
