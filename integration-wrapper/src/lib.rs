use node_runtime::{
	assets::events::{AccountsDestroyed, Destroyed, DestructionStarted, Transferred},
	dao_core::events::{DaoCreated, DaoDestroyed, DaoTokenIssued},
	runtime_types::{pallet_dao_core::types::Dao, sp_core::bounded::bounded_vec::BoundedVec},
};
use subxt::{tx::Signer, utils::AccountId32, OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

// The Config of the node to be tested
type Config = PolkadotConfig;

#[tokio::main]
pub async fn create_dao(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
	dao_name: Vec<u8>,
) -> Result<Option<DaoCreated>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().dao_core().create_dao(dao_id, dao_name);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn destroy_dao(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
) -> Result<Option<DaoDestroyed>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().dao_core().destroy_dao(dao_id);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn issue_token(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
	supply: u128,
) -> Result<Option<DaoTokenIssued>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().dao_core().issue_token(dao_id, supply);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn get_dao(
	dao_id: Vec<u8>,
) -> Result<Option<Dao<AccountId32, BoundedVec<u8>, u32>>, Box<(dyn std::error::Error + 'static)>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	let dao_storage = node_runtime::storage().dao_core().daos(BoundedVec(dao_id));
	let maybe_dao = api.storage().at(None).await?.fetch(&dao_storage).await?;
	Ok(maybe_dao)
}

#[tokio::main]
pub async fn transfer_tokens(
	signer: &impl Signer<Config>,
	asset_id: u32,
	target: AccountId32,
	amount: u128,
) -> Result<Option<Transferred>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().assets().transfer(
		asset_id,
		subxt::utils::MultiAddress::Id(target),
		amount,
	);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn start_destroy_asset(
	signer: &impl Signer<Config>,
	asset_id: u32,
) -> Result<Option<DestructionStarted>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().assets().start_destroy(asset_id);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn destroy_accounts(
	signer: &impl Signer<Config>,
	asset_id: u32,
) -> Result<Option<AccountsDestroyed>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().assets().destroy_accounts(asset_id);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn finish_destroy_asset(
	signer: &impl Signer<Config>,
	asset_id: u32,
) -> Result<Option<Destroyed>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().assets().finish_destroy(asset_id);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}
