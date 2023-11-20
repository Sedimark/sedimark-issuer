use identity_iota::{prelude::{IotaDID, IotaDocument}, verification::{MethodScope, jws::JwsAlgorithm}, credential::Credential, storage::{Storage, JwkDocumentExt, JwkMemStore}, iota::{NetworkName, IotaIdentityClientExt, IotaClientExt}};
use identity_stronghold::StrongholdStorage;
use iota_sdk::{client::{secret::{stronghold::StrongholdSecretManager, SecretManager}, stronghold::StrongholdAdapter, Client, node_api::indexer::query_parameters::QueryParameter, Password, constants::SHIMMER_COIN_TYPE, api::GetAddressesOptions}, crypto::keys::bip39::Mnemonic, types::block::{address::{Bech32Address, Address}, output::AliasOutput}, Wallet, wallet::ClientOptions};
use std::{env, path::PathBuf, path::Path};
use anyhow::{Result,Context};
use reqwest;
use serde_json::{self};
use std::fs::File;
use std::io::prelude::*;
use crate::{dtos::identity_dtos::AbiDTO, errors::IssuerError};


pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;

// pub fn setup_client_options() -> ClientOptions { 
//     ClientOptions::new().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap()
// }

pub async fn setup_client() -> Result<Client, IssuerError> {
  Client::builder().with_node(&env::var("NODE_URL").unwrap())?.finish().await.map_err(|e| IssuerError::from(e))
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

  let fragment: String = document
    .generate_method(
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

  let mut file = File::create("../abi/idsc_abi.json.").unwrap();
  let correct_json: AbiDTO = serde_json::from_str(&body).unwrap();
  file.write_all(correct_json.result.as_bytes()).unwrap();

  Ok(())
}

pub fn is_abi_downloaded() -> bool {
  let env_path = env::var("ABI_LOCAL_PATH").unwrap();
  let path = Path::new(&env_path);
  if path.exists() == true && path.metadata().unwrap().len() > 0 {
      return true
  };  
  return false;
}

pub fn get_abi_from_file() -> String {
  let abi = if is_abi_downloaded() == false {
    "".to_string()
  } else {
    let env_path = env::var("ABI_LOCAL_PATH").unwrap();
    let path = Path::new(&env_path);
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    data
  };
  abi
}

pub fn convert_string_to_iotadid(did: &String) -> IotaDID {
  IotaDID::parse(did).unwrap()
}

pub fn extract_pub_key_from_doc(did_doc: IotaDocument) -> Vec<u8> {
  did_doc.methods(Some(MethodScope::VerificationMethod))[0].data().try_decode().unwrap()
}

pub fn get_vc_id_from_credential(vc: Credential) -> i64 {
  let full_id = vc.id.unwrap();

  let split: Vec<&str> = full_id.as_str().split("/").collect();
  let id = split.get(split.len() - 1).unwrap().to_owned();
  let num: i64 = id.parse().unwrap();
  num
}

pub fn remove_0x_prefix(hex_string: &String) -> String {
  hex_string.strip_prefix("0x").unwrap().to_string()
}