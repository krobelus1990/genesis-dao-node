use node_runtime::{
	assets::events::{AccountsDestroyed, Destroyed, DestructionStarted, Transferred},
	dao_core::events::{DaoCreated, DaoDestroyed, DaoMetadataSet, DaoTokenIssued},
	runtime_types::{
		bounded_collections::bounded_vec::BoundedVec, pallet_dao_core::types::Dao as DaoInternal,
	},
	votes::events::{
		ProposalCreated, ProposalFaulted, ProposalMetadataSet, SetGovernanceMajorityVote, VoteCast,
	},
};
use subxt::{tx::Signer, utils::AccountId32, OnlineClient, PolkadotConfig};

// when this file becomes out of sync, regenerate it like so
// 1) cargo install subxt-cli
// 2) start a local node
// 3) ~/.cargo/bin/subxt metadata -f bytes > metadata.scale
// this step is optional
// 4) ~/.cargo/bin/subxt codegen | rustfmt > api.rs
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

// The Config of the node to be tested
type Config = PolkadotConfig;

type ProposalId = u64;

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
pub async fn set_metadata(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
	meta: Vec<u8>,
	hash: Vec<u8>,
) -> Result<Option<DaoMetadataSet>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().dao_core().set_metadata(dao_id, meta, hash);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

type Dao = DaoInternal<BoundedVec<u8>, AccountId32, BoundedVec<u8>, u32, BoundedVec<u8>>;

#[tokio::main]
pub async fn get_dao(
	dao_id: Vec<u8>,
) -> Result<Option<Dao>, Box<(dyn std::error::Error + 'static)>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	let dao_storage = node_runtime::storage().dao_core().daos(BoundedVec(dao_id));
	let maybe_dao = api.storage().at(None).await?.fetch(&dao_storage).await?;
	Ok(maybe_dao)
}

#[tokio::main]
pub async fn set_governance(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
	proposal_duration: u32,
	proposal_token_deposit: u128,
	minimum_majority_per_1024: u8,
) -> Result<Option<SetGovernanceMajorityVote>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().votes().set_governance_majority_vote(
		dao_id,
		proposal_duration,
		proposal_token_deposit,
		minimum_majority_per_1024,
	);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn create_proposal(
	signer: &impl Signer<Config>,
	dao_id: Vec<u8>,
) -> Result<Option<ProposalCreated>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().votes().create_proposal(dao_id);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn set_proposal_metadata(
	signer: &impl Signer<Config>,
	proposal_id: u64,
	metadata: Vec<u8>,
	hash: Vec<u8>,
) -> Result<Option<ProposalMetadataSet>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().votes().set_metadata(proposal_id, metadata, hash);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn fault_proposal(
	signer: &impl Signer<Config>,
	proposal_id: ProposalId,
	reason: Vec<u8>,
) -> Result<Option<ProposalFaulted>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().votes().fault_proposal(proposal_id, reason);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
}

#[tokio::main]
pub async fn vote(
	signer: &impl Signer<Config>,
	proposal_id: ProposalId,
	in_favor: Option<bool>,
) -> Result<Option<VoteCast>, Box<dyn std::error::Error>> {
	// client that can submit transactions
	let api = OnlineClient::<Config>::new().await?;

	// transaction to be submitted
	let tx = node_runtime::tx().votes().vote(proposal_id, in_favor);

	// submit the transaction and wait for its event
	let progress = api.tx().sign_and_submit_then_watch_default(&tx, signer).await?;
	Ok(progress.wait_for_finalized_success().await?.find_first()?)
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
