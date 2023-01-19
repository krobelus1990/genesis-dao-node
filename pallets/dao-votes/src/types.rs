use codec::MaxEncodedLen;
use frame_support::RuntimeDebug;

use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<Id, DaoId, AccountId> {
	pub id: Id,
	pub dao_id: DaoId,
	pub creator: AccountId,
	// pub created_at: timestamp
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Vote<AccountId> {
	pub voter: AccountId,
	pub aye: bool,
}
