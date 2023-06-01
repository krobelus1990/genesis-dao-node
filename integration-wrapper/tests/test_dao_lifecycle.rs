use integration_wrapper::*;
use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, utils::AccountId32};

#[test]
fn dao_lifecycle() {
	// test account to be used
	let user = PairSigner::new(AccountKeyring::Alice.pair());

	let (dao_id, dao_name) = (b"DAO".to_vec(), b"Test DAO".to_vec());

	match create_dao(&user, dao_id.clone(), dao_name) {
		Err(error) => panic!("Error creating DAO: {error}"),
		Ok(None) => panic!("No DaoCreated event"),
		Ok(Some(event)) => {
			assert_eq!(&event.owner, user.account_id(), "Created DAO with wrong owner");
			assert_eq!(event.dao_id.0, dao_id, "Created DAO with wrong id");
		},
	}

	let metadata = b"http://my.cool.dao".to_vec();
	// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	match set_metadata(&user, dao_id.clone(), metadata, hash) {
		Err(error) => panic!("Error setting DAO metadata: {error}"),
		Ok(None) => panic!("No DaoMetadataSet event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Set metadata for wrong DAO");
		},
	}

	let token_supply = 1_000_000;
	let asset_id;
	match issue_token(&user, dao_id.clone(), token_supply) {
		Err(error) => panic!("Error issuing DAO token: {error}"),
		Ok(None) => panic!("No DaoTokenIssued event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Issued token for wrong DAO");
			assert_eq!(event.supply, token_supply, "Issued token with wrong supply");
			asset_id = event.asset_id;
		},
	}

	match get_dao(dao_id.clone()) {
		Err(error) => panic!("Error reading DAO: {error}"),
		Ok(None) => panic!("No DAO known by this id"),
		Ok(Some(dao)) => {
			assert!(dao.asset_id.is_some(), "DAO has no asset id");
			assert_eq!(
				dao.asset_id.unwrap(),
				asset_id,
				"Mismatch between asset id in storage and in event"
			);
		},
	}

	let proposal_duration = 100_u32.into();
	let proposal_token_deposit = 1_u32.into();
	let minimum_majority_per_1024 = 10;
	match set_governance(
		&user,
		dao_id.clone(),
		proposal_duration,
		proposal_token_deposit,
		minimum_majority_per_1024,
	) {
		Err(error) => panic!("Error setting governance: {error}"),
		Ok(None) => panic!("No SetGovernanceMajorityVote event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Set governance for wrong DAO");
			assert_eq!(event.proposal_duration, proposal_duration, "Set wrong proposal duration");
			assert_eq!(
				event.proposal_token_deposit, proposal_token_deposit,
				"Set wrong proposal token deposit"
			);
			assert_eq!(
				event.minimum_majority_per_1024, minimum_majority_per_1024,
				"Set wrong minimum majority"
			);
		},
	}

	// create proposal to be faulted
	let faulty_proposal_id;
	match create_proposal(&user, dao_id.clone()) {
		Err(error) => panic!("Error creating proposal: {error}"),
		Ok(None) => panic!("No ProposalCreated event"),
		Ok(Some(event)) => {
			faulty_proposal_id = event.proposal_id;
		},
	}

	// set its metadata
	let metadata = b"http://my.cool.proposal".to_vec();
	// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	match set_proposal_metadata(&user, faulty_proposal_id, metadata.clone(), hash.clone()) {
		Err(error) => panic!("Error setting proposal metadata: {error}"),
		Ok(None) => panic!("No ProposalMetadataSet event"),
		Ok(Some(event)) => {
			assert_eq!(event.proposal_id, faulty_proposal_id, "Set metadata for wrong proposal");
		},
	}

	let reason = b"Bad".to_vec();
	match fault_proposal(&user, faulty_proposal_id, reason.clone()) {
		Err(error) => panic!("Error faulting proposal: {error}"),
		Ok(None) => panic!("No ProposalFaulted event"),
		Ok(Some(event)) => {
			assert_eq!(event.proposal_id, faulty_proposal_id, "Faulted proposal with wrong id");
			assert_eq!(event.reason, reason, "Faulted proposal for wrong reason");
		},
	}

	// create fresh proposal
	let proposal_id;
	match create_proposal(&user, dao_id.clone()) {
		Err(error) => panic!("Error creating proposal: {error}"),
		Ok(None) => panic!("No ProposalCreated event"),
		Ok(Some(event)) => {
			proposal_id = event.proposal_id;
		},
	}

	// set its metadata
	match set_proposal_metadata(&user, proposal_id, metadata, hash) {
		Err(error) => panic!("Error setting proposal metadata: {error}"),
		Ok(None) => panic!("No ProposalMetadataSet event"),
		Ok(Some(event)) => {
			assert_eq!(event.proposal_id, proposal_id, "Set metadata for wrong proposal");
		},
	}

	let in_favor = Some(true);
	match vote(&user, proposal_id, in_favor) {
		Err(error) => panic!("Error voting: {error}"),
		Ok(None) => panic!("No VoteCast event"),
		Ok(Some(event)) => {
			assert_eq!(event.proposal_id, proposal_id, "Created vote with wrong proposal id");
			assert_eq!(event.voter, *user.account_id(), "Created vote for wrong voter");
		},
	}

	let to: AccountId32 = AccountKeyring::Bob.to_account_id().into();
	let transfer_amount = 1000;
	match transfer_tokens(&user, asset_id, to.clone(), transfer_amount) {
		Err(error) => panic!("Error transferring DAO tokens: {error}"),
		Ok(None) => panic!("No Transferred event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Transfer of wrong asset");
			assert_eq!(event.from, *user.account_id(), "Transfer from wrong account");
			assert_eq!(event.to, to, "Transfer to wrong account");
			assert_eq!(event.amount, transfer_amount, "Transfer of wrong amount");
		},
	}

	match start_destroy_asset(&user, asset_id) {
		Err(error) => panic!("Error starting destroying asset: {error}"),
		Ok(None) => panic!("No DestructionStarted event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Start destroying wrong asset");
		},
	}

	match destroy_accounts(&user, asset_id) {
		Err(error) => panic!("Error destroying accounts: {error}"),
		Ok(None) => panic!("No AccountsDestroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Destroying accounts of wrong asset");
		},
	}

	match finish_destroy_asset(&user, asset_id) {
		Err(error) => panic!("Error finishing destroying asset: {error}"),
		Ok(None) => panic!("No Destroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Destroyed wrong asset");
		},
	}

	match destroy_dao(&user, dao_id.clone()) {
		Err(error) => panic!("Error destroying DAO: {error}"),
		Ok(None) => panic!("No DaoDestroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Destroyed wrong DAO");
		},
	}
}
