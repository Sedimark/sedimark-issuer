pub mod config;
pub mod handler;
pub mod dtos;
pub mod services;
pub mod utils;
pub mod errors;
pub mod db;

use std::sync::Arc;

use iota_wallet::secret::SecretManager;
use iota_wallet::account::AccountHandle;
// use iota_wallet::account_manager::AccountManager;
use db::models::Identity;
use tokio::sync::RwLock;

use ethers::providers::{Provider, Http};
use ethers::prelude::{SignerMiddleware, k256};
use ethers::prelude::Wallet;

type EthClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

// This struct represents state
pub struct IssuerState {
    // pub account_manager: AccountManager,
    pub issuer_account: AccountHandle,
    pub secret_manager: Arc<RwLock<SecretManager>>,
    pub issuer_identity: Identity,
    pub eth_client: EthClient
}