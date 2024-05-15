// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use deadpool_postgres::Pool;
use identity_iota::{iota::{IotaClientExt, IotaDID, IotaIdentityClientExt, NetworkName}, prelude::IotaDocument, storage::{JwkDocumentExt, JwkMemStore, Storage}, verification::{jws::JwsAlgorithm, MethodScope}};
use identity_stronghold::StrongholdStorage;
use iota_sdk::{client::{secret::{stronghold::StrongholdSecretManager, SecretManager}, Client, node_api::indexer::query_parameters::QueryParameter, Password, api::GetAddressesOptions}, crypto::keys::bip39::Mnemonic, types::block::{address::Bech32Address, output::AliasOutput}};
use std::env;
use anyhow::{Result,Context};
use reqwest;
use serde_json::{self};
use std::fs::File;
use std::io::prelude::*;
use crate::{dtos::identity_dtos::AbiDTO, errors::IssuerError, repository::{models::Identity, operations::IdentityExt}};


pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;

// pub struct IotaState {
//   pub client: Client,
//   pub stronghold_storage: StrongholdStorage,
//   pub key_storage: MemStorage,
//   pub address: Bech32Address,
//   pub faucet_url: String
// }

pub struct IotaState {
  pub client: Client,
  pub key_storage: MemStorage,
  pub stronghold_storage: StrongholdStorage,
  pub issuer_identity: Identity,
  pub issuer_document: IotaDocument,
  pub faucet_url: String
}

impl IotaState {
	pub async fn init(db_pool: &Pool) -> Result<Self> {

		log::info!("Creating or recovering issuer state...");

		let pg_client = &db_pool.get().await?;

		let node_url = std::env::var("NODE_URL").expect("$NODE_URL must be set.");
		let faucet_url = std::env::var("FAUCET_URL").expect("$FAUCET_URL must be set.");

		let client = Client::builder()
		.with_node(&node_url)?
		.finish()
		.await?;

		// Create or load issuer's identity.
		let (key_storage, secret_manager) = create_or_recover_key_storage().await?;
		// check if a did already exists
		let (issuer_identity, issuer_document ) = match pg_client.get_identity_did().await {
			Ok(identity) => {
				let issuer_document = client.resolve_did(&IotaDID::parse(&identity.did)?).await?;
				(identity, issuer_document)
			},
			Err(_) => {
				log::info!("Creating new identity... ");
	
				// create a did with a verification method
				let (_, issuer_document, fragment) = create_did(&client, secret_manager.as_secret_manager(), &key_storage).await?;
				// save the created identity
				let new_issuer_identity = Identity { did: issuer_document.id().to_string(), fragment:  fragment};
				pg_client.insert_identity_issuer(&new_issuer_identity).await?;
				(new_issuer_identity, issuer_document)
			},
		};

		let iota_state = IotaState{
			client,
			key_storage: key_storage,
			stronghold_storage: secret_manager,
			issuer_identity,
			issuer_document,
			faucet_url,
		};
		Ok(iota_state)
	}

}

/// Creates a DID Document and publishes it in a new Alias Output.
///
/// Its functionality is equivalent to the "create DID" Iota example.
pub async fn create_did(
  	client: &Client,
  	secret_manager: &SecretManager,
  	storage: &MemStorage,
) -> anyhow::Result<(Bech32Address, IotaDocument, String)> {
	let faucet_endpoint = env::var("FAUCET_URL").expect("$FAUCET_URL must be set");
	let bech32_hrp = client.get_bech32_hrp().await?;
	let address = secret_manager.generate_ed25519_addresses(
		GetAddressesOptions::default()
		.with_range(0..1)
		.with_bech32_hrp(bech32_hrp)
	).await?[0];
	ensure_address_has_funds(client, &address, &faucet_endpoint).await?;
	
	let network_name: NetworkName = client.network_name().await?;
	let (document, fragment): (IotaDocument, String) = create_did_document(&network_name, storage).await?;
	let alias_output: AliasOutput = client.new_did_output(address.into_inner(), document, None).await?;
	let document: IotaDocument = client.publish_did_output(secret_manager, alias_output).await?;

	Ok((address, document, fragment))
}

/// Creates an example DID document with the given `network_name`.
///
/// Its functionality is equivalent to the "create DID" example
/// and exists for convenient calling from the other examples.
pub async fn create_did_document(
  network_name: &NetworkName,
  storage: &MemStorage,
) -> anyhow::Result<(IotaDocument, String)> {
  let mut document: IotaDocument = IotaDocument::new(network_name);

  let fragment: String = document.generate_method(
		storage,
		JwkMemStore::ED25519_KEY_TYPE,
		JwsAlgorithm::EdDSA,
		None,
		MethodScope::VerificationMethod,
	)
	.await?;

  Ok((document, fragment))
}



pub async fn create_or_recover_key_storage() -> Result<(MemStorage, StrongholdStorage)> {
  log::info!("Creating or recovering storage...");

  // Setup Stronghold secret_manager
  let stronghold = StrongholdSecretManager::builder()
  .password(Password::from(std::env::var("KEY_STORAGE_STRONGHOLD_PASSWORD").unwrap()))
  .build(&std::env::var("KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH").unwrap())?;

  // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
  let mnemonic = Mnemonic::from(std::env::var("KEY_STORAGE_MNEMONIC").unwrap());

  match stronghold.store_mnemonic(mnemonic).await {
	Ok(()) => log::info!("Stronghold mnemonic stored"),
	Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => log::info!("Stronghold mnemonic already stored"),
	Err(error) => panic!("Error: {:?}", error)
  }

  // Create a `StrongholdStorage`.
  // `StrongholdStorage` creates internally a `SecretManager` that can be
  // referenced to avoid creating multiple instances around the same stronghold snapshot.
  let stronghold_storage = StrongholdStorage::new(stronghold);

  // Create storage for key-ids and JWKs.
  //
  // In this example, the same stronghold file that is used to store
  // key-ids as well as the JWKs.
  let key_storage = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

  Ok((key_storage, stronghold_storage))
}

/// Requests funds from the faucet for the given `address`.
pub async fn request_faucet_funds(client: &Client, address: &Bech32Address, faucet_endpoint: &str) -> anyhow::Result<()> {
  iota_sdk::client::request_funds_from_faucet(faucet_endpoint, &address).await?;

  tokio::time::timeout(std::time::Duration::from_secs(45), async {
	  loop {
	  tokio::time::sleep(std::time::Duration::from_secs(5)).await;

	  let balance = get_address_balance(client, &address)
		  .await
		  .context("failed to get address balance")?;
	  if balance > 0 {
		  break;
	  }
	  }
	  Ok::<(), anyhow::Error>(())
  })
  .await
  .context("maximum timeout exceeded")??;

  Ok(())
}

/// Returns the balance of the given Bech32-encoded `address`.
pub async fn get_address_balance(client: &Client, address: &Bech32Address) -> anyhow::Result<u64> {
  let output_ids = client
	  .basic_output_ids(vec![
	  QueryParameter::Address(address.to_owned()),
	  QueryParameter::HasExpiration(false),
	  QueryParameter::HasTimelock(false),
	  QueryParameter::HasStorageDepositReturn(false),
	  ])
	  .await?;

  let outputs = client.get_outputs(&output_ids).await?;

  let mut total_amount = 0;
  for output_response in outputs {
	  total_amount += output_response.output().amount();
  }

  Ok(total_amount)
}

pub async fn ensure_address_has_funds(client: &Client, address: &Bech32Address, faucet_endpoint: &String) -> anyhow::Result<()> {

  let balance = get_address_balance(client, address)
  .await
  .context("failed to get address balance")?;

  if balance == 0 {
	log::info!("Funding address {}", address);
	request_faucet_funds(client, address, faucet_endpoint).await?;

  } else {
	log::info!("Address has already enough funds: {}.", balance);
  }
  Ok(())
}










pub async fn download_contract_abi_file() -> anyhow::Result<(), ()> {
  dotenv::dotenv().ok();
  let shimmer_evm_explorer: String = env::var("SHIMMER_EVM_EXPLORER").unwrap();
  let contract_address = env::var("IDENTITY_SC_ADDRESS").unwrap();

  let url = String::from(shimmer_evm_explorer + "/api?module=contract&action=getabi&address=" + &contract_address);
  log::info!("Downloading ABI from {}", url);
  let body = reqwest::get(url)
	.await.unwrap()
	.text()
	.await.unwrap();

  let mut file = File::create("../abi/identity_sc.json.").unwrap();
  let correct_json: AbiDTO = serde_json::from_str(&body).unwrap();
  file.write_all(correct_json.result.as_bytes()).unwrap();

  Ok(())
}

// pub fn extract_pub_key_from_doc(did_doc: IotaDocument) -> Vec<u8> {
//   did_doc.methods(Some(MethodScope::VerificationMethod))[0].data().try_decode().unwrap()
// }

// pub fn get_vc_id_from_credential(vc: Credential) -> i64 {
//   let full_id = vc.id.unwrap();

//   let split: Vec<&str> = full_id.as_str().split("/").collect();
//   let id = split.get(split.len() - 1).unwrap().to_owned();
//   let num: i64 = id.parse().unwrap();
//   num
// }