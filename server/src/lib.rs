// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod handlers;
pub mod dtos;
pub mod services;
pub mod utils;
pub mod errors;
pub mod repository;
pub mod middlewares;
pub mod contracts;

use std::sync::{Arc, RwLock};

use repository::models::Identity;

use ethers::providers::{Provider, Http};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{SignerMiddleware, ContractInstance};
use ethers::prelude::Wallet;

use identity_iota::iota::IotaDocument;
use identity_stronghold::StrongholdStorage;
use utils::iota_utils::MemStorage;


pub type EthClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;
pub type LocalContractInstance = ContractInstance<Arc<SignerMiddleware<ethers::providers::Provider<Http>, Wallet<SigningKey>>>, SignerMiddleware<ethers::providers::Provider<Http>, Wallet<SigningKey>>>;

// This struct represents the Issuer state
pub struct IssuerState {
    pub key_storage: Arc<RwLock<MemStorage>>,
    pub secret_manager: Arc<RwLock<StrongholdStorage>>,
    pub issuer_identity: Identity,
    pub issuer_document: IotaDocument,
    pub eth_client: Arc<EthClient>,
    pub idsc_instance: LocalContractInstance
}