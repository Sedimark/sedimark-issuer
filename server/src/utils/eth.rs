use std::sync::Arc;

use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::providers::{Provider, Http};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::SignerMiddleware;
use ethers::prelude::Wallet;
use ethers::types::Bytes;
use ethers::core::types::U256;

use ethers::utils::hex::FromHex;
use identity_iota::credential::DecodedJwtCredential;

use crate::contracts::identity::Identity;
use crate::contracts::identity::VcAddedFilter;
use crate::errors::IssuerError;

pub type SignerMiddlewareShort = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;



pub async fn update_identity_sc(
    identity_sc: Identity<&Arc<SignerMiddlewareShort>>,
    decoded_jwt_credential: DecodedJwtCredential, 
    credential_id: U256,
    challenge: String, 
    wallet_sign: &String, 
) -> Result<(), IssuerError> {

    let wallet_sign_bytes = Bytes::from(Vec::from_hex(wallet_sign.strip_prefix("0x").ok_or(IssuerError::OtherError("Error during strip prefix".to_owned()))?.to_string()).map_err(|_| IssuerError::OtherError("Conversion error".to_owned()))?);
    let challenge_bytes = Bytes::from(challenge.into_bytes());
    let expiration_date = U256::from(decoded_jwt_credential.credential.expiration_date.ok_or(IssuerError::OtherError("Expiration date not found".to_owned()))?.to_unix());
    let issuance_date = U256::from(decoded_jwt_credential.credential.issuance_date.to_unix());
      
    let call = identity_sc.add_user(
        credential_id, 
        expiration_date,
        issuance_date,
        wallet_sign_bytes.into(), 
        challenge_bytes.into()
    );
    let pending_tx = call.send().await.map_err(|err| IssuerError::ContractError(err.to_string()))?;
    let receipt = pending_tx.confirmations(1).await.map_err(|err| IssuerError::ContractError(err.to_string()))?;

    let logs = receipt.ok_or(IssuerError::OtherError("No receipt".to_owned()))?.logs;

    // reading the log   
    for log in logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        };
        // finding the event
        if let Ok(event) =  <VcAddedFilter as EthEvent>::decode_log(&raw_log){
            log::info!("VcAdded event:\n{:?}", event);
            return Ok(());
        }
    }
    Err(IssuerError::OtherError("no VcAdded event found in the receipt".to_owned()))

}