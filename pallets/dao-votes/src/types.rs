use codec::MaxEncodedLen;
use frame_support::RuntimeDebug;

use frame_support::codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Vote<DaoId> {
	pub dao_id: DaoId,
}
