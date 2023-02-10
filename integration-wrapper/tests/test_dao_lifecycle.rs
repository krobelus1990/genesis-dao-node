use integration_wrapper::*;
use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, utils::AccountId32};

#[test]
fn dao_lifecycle() {
	// test account to be used
	let user = PairSigner::new(AccountKeyring::Alice.pair());

	let (dao_id, dao_name) = (b"DAO".to_vec(), b"Test DAO".to_vec());

	match create_dao(&user, dao_id.clone(), dao_name) {
		Err(error) => assert!(false, "Error creating DAO: {error}"),
		Ok(None) => assert!(false, "No DaoCreated event"),
		Ok(Some(event)) => {
			assert_eq!(&event.owner, user.account_id(), "Created DAO with wrong owner");
			assert_eq!(event.dao_id.0, dao_id, "Created DAO with wrong id");
		},
	}

	let metadata = b"http://my.cool.dao".to_vec();
	// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	match set_metadata(&user, dao_id.clone(), metadata, hash) {
		Err(error) => assert!(false, "Error setting DAO metadata: {error}"),
		Ok(None) => assert!(false, "No DaoMetadataSet event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Set metadata for wrong DAO");
		},
	}

	let token_supply = 1_000_000;
	let mut asset_id = None;
	match issue_token(&user, dao_id.clone(), token_supply) {
		Err(error) => assert!(false, "Error issuing DAO token: {error}"),
		Ok(None) => assert!(false, "No DaoTokenIssued event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Issued token for wrong DAO");
			assert_eq!(event.supply, token_supply, "Issued token with wrong supply");
			asset_id = Some(event.asset_id);
		},
	}
	let asset_id = asset_id.unwrap();

	match get_dao(dao_id.clone()) {
		Err(error) => assert!(false, "Error reading DAO: {error}"),
		Ok(None) => assert!(false, "No DAO known by this id"),
		Ok(Some(dao)) => {
			assert!(dao.asset_id.is_some(), "DAO has no asset id");
			assert_eq!(
				dao.asset_id.unwrap(),
				asset_id,
				"Mismatch between asset id in storage and in event"
			);
		},
	}

	let other_user: AccountId32 = AccountKeyring::Bob.to_account_id().into();
	match transfer_tokens(&user, asset_id, other_user.clone(), token_supply) {
		Err(error) => assert!(false, "Error transferring DAO tokens: {error}"),
		Ok(None) => assert!(false, "No Transferred event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Transfer of wrong asset");
			assert_eq!(event.from, *user.account_id(), "Transfer from wrong account");
			assert_eq!(event.to, other_user, "Transfer to wrong account");
			assert_eq!(event.amount, token_supply, "Transfer of wrong amount");
		},
	}

	match start_destroy_asset(&user, asset_id) {
		Err(error) => assert!(false, "Error starting destroying asset: {error}"),
		Ok(None) => assert!(false, "No DestructionStarted event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Start destroying wrong asset");
		},
	}

	match destroy_accounts(&user, asset_id) {
		Err(error) => assert!(false, "Error destroying accounts: {error}"),
		Ok(None) => assert!(false, "No AccountsDestroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Destroying accounts of wrong asset");
		},
	}

	match finish_destroy_asset(&user, asset_id) {
		Err(error) => assert!(false, "Error finishing destroying asset: {error}"),
		Ok(None) => assert!(false, "No Destroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.asset_id, asset_id, "Destroyed wrong asset");
		},
	}

	match destroy_dao(&user, dao_id.clone()) {
		Err(error) => assert!(false, "Error destroying DAO: {error}"),
		Ok(None) => assert!(false, "No DaoDestroyed event"),
		Ok(Some(event)) => {
			assert_eq!(event.dao_id.0, dao_id, "Destroyed wrong DAO");
		},
	}
}
