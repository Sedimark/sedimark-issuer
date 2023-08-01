use iota_wallet::ClientOptions;
use iota_client::{Client, secret::{SecretManager, stronghold::StrongholdSecretManager}};
use std::{env, path::PathBuf};

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