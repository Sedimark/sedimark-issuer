// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use ethers::prelude::{abigen, Contract};
use ethers::contract::Lazy;
use ethers::types::Address;
use iota_sdk::Wallet;
use iota_sdk::client::constants::SHIMMER_COIN_TYPE;
use iota_sdk::client::secret::SecretManager;
use iota_sdk::client::stronghold::StrongholdAdapter;
use iota_sdk::wallet::ClientOptions;
use std::{env, path::PathBuf, sync::Arc};
use crate::LocalContractInstance;
use crate::utils::iota_utils::get_abi_from_file;
use crate::EthClient;

/// creates or loads the issuer's wallet account. It also ensures that the main account address ([0]) has 
/// funds. Funds are obtained from the faucet. If no funds are given to the account's address 
/// the issuer cannot publish new DIDs, hence he cannot create its SSI.



/// Checks if the idsc_abi.json has been already downloaded, if not the ABI is retrieved and stored locally.
/// The abigen! macro generates a type-safe binding to an Ethereum smart contract from its ABI (for the IDSC in this case). 
pub async fn setup_eth_wallet(eth_client: Arc<EthClient>) -> LocalContractInstance {
    abigen!(IDSC, "$CARGO_MANIFEST_DIR/abi/idsc_abi.json");

    let json_abi: Lazy<ethers::abi::Abi> = Lazy::new(|| {
        let from_file_abi = get_abi_from_file();
        if from_file_abi.len() == 0 {
            panic!("ABI file does not exist or is empty");
        }        
        serde_json::from_str(from_file_abi.as_str()).expect("Invalid ABI")
    });

    let contract_address: Address = env::var("IDENTITY_SC_ADDRESS").unwrap().as_str().parse().unwrap();
    let idsc_istance = Contract::new(contract_address, json_abi.clone(), eth_client.clone());

    idsc_istance  
}

