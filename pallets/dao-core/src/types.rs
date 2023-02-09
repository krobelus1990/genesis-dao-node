use codec::MaxEncodedLen;
use frame_support::{
	codec::{Decode, Encode},
	traits::ConstU32,
	BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;

/// The DAO model
///
/// - `id`: Unique identifier of the DAO
/// - `owner`: AccountId of the owner of this DAO
/// - `name`: Name of the DAO
/// - `asset_id`: Identifier of the issued token (optional, as token may be issued later)
/// - `meta` : HTTP or IPFS address for the metadata about this DAO (description, logo)
/// - `meta_hash` : SHA3 hash of the metadata to be found via `meta`
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Dao<DaoId, AccountId, DaoName, AssetId, Metadata> {
	pub id: DaoId,
	pub owner: AccountId,
	pub name: DaoName,
	pub asset_id: Option<AssetId>,
	pub meta: Metadata,
	pub meta_hash: BoundedVec<u8, ConstU32<256>>,
}
