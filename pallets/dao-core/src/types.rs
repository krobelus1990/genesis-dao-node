use codec::MaxEncodedLen;
use frame_support::RuntimeDebug;

use frame_support::codec::{Encode, Decode};
use scale_info::TypeInfo;

/// The DAO model
///
/// - `id`: The unique identifier of the DAO
/// - `owner`: AccountId of the owner of this DAO
/// - `name`: Unique identifier of the DAO
/// - `asset_id`: The identifier of the asset of the issued token (optional, as token maybe issued later)
///
#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Dao<DaoId, AccountId, DaoName, AssetId> {
	pub id: DaoId,
	pub owner: AccountId,
	pub name: DaoName,
	pub asset_id: Option<AssetId>
}
