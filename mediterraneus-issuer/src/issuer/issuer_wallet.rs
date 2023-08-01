use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::SecretManager,
    ClientOptions, Result, account::AccountHandle,
};
use std::{env, path::PathBuf};

pub async fn create_or_load_wallet_account(secret_manager: SecretManager, client_options: ClientOptions) -> Result<(AccountManager, AccountHandle)> {
    dotenv::dotenv().ok();
    let manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        .with_storage_path(PathBuf::from("../walletdb").to_str().unwrap())
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Create a new account
    let _account = match manager
        .create_account()
        .with_alias(env::var("WALLET_ALIAS").unwrap().to_string())
        .finish()
        .await {
            Ok(_account) => _account,
            Err(iota_wallet::Error::AccountAliasAlreadyExists(_account)) => {
                log::info!("Account already exists, loading it.");
                manager.get_account(env::var("WALLET_ALIAS").unwrap().to_string()).await?
            },
            Err(error) => panic!("Error: {:?}", error)
        };
        
    log::info!("Account address: {:?}.", _account.addresses().await?[0].address().to_bech32());
    Ok((manager, _account))
}