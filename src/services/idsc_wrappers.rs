// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;
use ethers::{core::types::{Bytes, U256}, prelude::abigen, utils::hex::FromHex};
use crate::{EthClient, LocalContractInstance, utils::iota_utils::remove_0x_prefix};

// Generate the type-safe contract bindings by providing the ABI definition
// TODO: use Abigen to generate the contract bindings from the ABI
abigen!(
    IDSC,
    "./abi/identity_sc.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// Retrieves the first free VC ID from the IDSC
pub async fn get_free_vc_id(
    idsc_instance: LocalContractInstance,
    eth_client: Arc<EthClient>
) -> U256 {    
    idsc_instance
        .connect(eth_client.clone())
        .method::<_, U256>("getFreeVCid", ())
        .unwrap()
        .call()
        .await.unwrap()
}

pub async fn register_new_vc_idsc(
    idsc_instance: LocalContractInstance,
    eth_client: Arc<EthClient>,
    credential_id: U256, 
    wallet_sign: &String, 
    holder_did: &String, 
    exp_unix: i64, 
    issuance_unix: i64, 
    challenge: String
) -> anyhow::Result<()> {
    let pseudo_sign_bytes = Bytes::from(Vec::from_hex(remove_0x_prefix(&wallet_sign))?);
    let challenge_bytes = Bytes::from(challenge.into_bytes());
 
    // 2 ways of doing the same thing 
    
    // let address: Address = "0xa3740B38131A0738DA7A6097261f5Bc5500cb24d".parse().unwrap();
    // let contract = IDSC::new(address, eth_client.clone());
    
    // let tx = contract.add_user(
    //     credential_id, 
    //     bytes, 
    //     holder_did, 
    //     U256::from_dec_str(exp_unix.to_string().as_str()).unwrap(), 
    //     U256::from_dec_str(issuance_unix.to_string().as_str()).unwrap(), 
    //     <[u8; 32]>::try_from(Vec::from_hex(remove_0x_prefix(vc_hash.clone())).unwrap()).unwrap()
    // ).send().await.unwrap().await.unwrap();
    // log::info!("Transaction Receipt: {}", serde_json::to_string(&tx).unwrap());

    
    let call = idsc_instance
        .connect(eth_client.clone())
        .method::<(U256, U256, U256, Bytes, Bytes), ()>(
        "add_user",
        (
            credential_id, 
            U256::from_dec_str(exp_unix.to_string().as_str()).unwrap(),
            U256::from_dec_str(issuance_unix.to_string().as_str()).unwrap(), 
            pseudo_sign_bytes, 
            challenge_bytes
            // <[u8; 16]>::try_from(Vec::from_hex(challenge.clone()).unwrap()).unwrap()
        ))
        .expect("method not found (this should never happen)");
    let pending_tx = call.send().await?;
    // `await`ing on the pending transaction resolves to a transaction receipt
    let receipt = pending_tx.confirmations(1).await?;
    log::info!("{:?}", receipt.unwrap());
    Ok(())
}