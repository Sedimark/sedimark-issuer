use ethers::prelude::Contract;
use iota_wallet::ClientOptions;
use iota_client::{Client, secret::{SecretManager, stronghold::StrongholdSecretManager}, block::{address::Address, output::Output}, node_api::indexer::query_parameters::QueryParameter};
use std::{env, path::PathBuf};
use anyhow::Context;
use reqwest;
use serde_json::{self, json};

pub fn setup_client_options() -> ClientOptions { 
    dotenv::dotenv().ok();
    ClientOptions::new().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap()
}

pub fn setup_client() -> Client {
    dotenv::dotenv().ok();
    Client::builder()
        .with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap()
        .finish()
        .unwrap()
}

pub async fn setup_secret_manager() -> SecretManager {
    dotenv::dotenv().ok();
    let mut ss = StrongholdSecretManager::builder()
    .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
    .build(PathBuf::from("../wallet.stronghold")).unwrap();

    let mnemonic = &env::var("NON_SECURE_MNEMONIC").unwrap();
    // let mnemonic = iota_client::generate_mnemonic().unwrap();

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    // The mnemonic only needs to be stored the first time
    match ss.store_mnemonic(mnemonic.to_string()).await {
        Ok(()) => log::info!("Stronghold mnemonic stored"),
        Err(iota_client::Error::StrongholdMnemonicAlreadyStored) => log::info!("Stronghold mnemonic already stored"),
        Err(error) => panic!("Error: {:?}", error)
    }
 
    // log::info!("Mnemonic generated {}. Save it.", mnemonic);
    SecretManager::Stronghold(ss)
}

/// Requests funds from the faucet for the given `address`.
async fn request_faucet_funds(
    client: &Client,
    address: Address,
    network_hrp: &str,
    faucet_endpoint: &str,
  ) -> anyhow::Result<()> {
    let address_bech32 = address.to_bech32(network_hrp);
  
    iota_client::request_funds_from_faucet(faucet_endpoint, &address_bech32).await?;
  
    tokio::time::timeout(std::time::Duration::from_secs(45), async {
      loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
  
        let balance = get_address_balance(client, &address_bech32)
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
async fn get_address_balance(client: &Client, address: &str) -> anyhow::Result<u64> {
    let output_ids = client
      .basic_output_ids(vec![
        QueryParameter::Address(address.to_owned()),
        QueryParameter::HasExpiration(false),
        QueryParameter::HasTimelock(false),
        QueryParameter::HasStorageDepositReturn(false),
      ])
      .await?;
  
    let outputs_responses = client.get_outputs(output_ids).await?;
  
    let mut total_amount = 0;
    for output_response in outputs_responses {
      let output = Output::try_from_dto(&output_response.output, client.get_token_supply().await?)?;
      total_amount += output.amount();
    }
    
    Ok(total_amount)
  }

pub async fn ensure_address_has_funds(client: &Client, address: Address, faucet_endpoint: &str) -> anyhow::Result<()> {
    let network_hrp = &client.get_bech32_hrp().await?;
    let address_bech32 = address.to_bech32(network_hrp);

    let balance = get_address_balance(client, &address_bech32)
          .await
          .context("failed to get address balance")?;
    if balance == 0 {
        log::info!("Funding address {}", address.to_bech32(network_hrp));
        request_faucet_funds(client, address, &network_hrp, faucet_endpoint).await?;
    } else {
        log::info!("Address has already enough funds: {}.", balance);
    }
    Ok(())
  }

pub async fn get_contract_abi() -> Result<(), ()> {
  dotenv::dotenv().ok();
  let shimmer_evm_explorer: String = env::var("SHIMMER_EVM_EXPLORER").unwrap();
  let contract_address = env::var("IDENTITY_SC_ADDRESS").unwrap();

  let url = String::from(shimmer_evm_explorer + "/api?module=contract&action=getabi&address=" + &contract_address);
  log::info!("Downloading ABI from {}", url);
  let body = reqwest::get(url)
    .await.unwrap()
    .text()
    .await.unwrap();

  let json_abi: serde_json::Value = serde_json::from_str(body.as_str()).unwrap();
  // let contract_istance = Contract::new(ethers::addressbook::Address::from(contract_address), json_abi, client);
  log::info!("Got {}", body);
  Ok(())
}