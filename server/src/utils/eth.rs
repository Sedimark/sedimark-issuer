use std::time::Duration;

use alloy::hex::FromHex;

use alloy::primitives::Bytes;
use alloy::primitives::U256;
use alloy::providers::DynProvider;
use alloy::sol_types::SolEvent;
use identity_iota::credential::DecodedJwtCredential;
use crate::contracts::Identity::IdentityInstance;
use crate::contracts::Identity::VC_added;
use crate::errors::IssuerError;



pub async fn update_identity_sc(
    identity_sc: actix_web::web::Data<IdentityInstance<DynProvider>>,
    decoded_jwt_credential: DecodedJwtCredential, 
    credential_id: U256,
    challenge: String, 
    wallet_sign: &String, 
    nonce: u64
) -> Result<(), IssuerError> {

    let wallet_sign_bytes = Bytes::from(Vec::from_hex(wallet_sign.strip_prefix("0x").ok_or(IssuerError::OtherError("Error during strip prefix".to_owned()))?.to_string()).map_err(|_| IssuerError::OtherError("Conversion error".to_owned()))?);
    let challenge_bytes = Bytes::from(challenge.into_bytes());
    let expiration_date = U256::from(decoded_jwt_credential.credential.expiration_date.ok_or(IssuerError::OtherError("Expiration date not found".to_owned()))?.to_unix());
    let issuance_date = U256::from(decoded_jwt_credential.credential.issuance_date.to_unix());
    
    let call = identity_sc.addUser(
        credential_id, 
        expiration_date,
        issuance_date,
        wallet_sign_bytes.into(), 
        challenge_bytes.into()
    )
    .gas_price(10_000_000_000)
    .nonce(nonce);

    let receipt = call
        .send()
        .await
        .map_err(|err| IssuerError::ContractError(format!("User registration failed: {}",err.to_string())))?
        .with_timeout(Some(Duration::from_secs(20)))
        .get_receipt()
        .await
        .map_err(|err| IssuerError::ContractError(format!("User registration failed: {}",err.to_string())))?;

    // reading the log   
    for log in receipt.logs() {
        // finding the event
        if let Ok(event) =  <VC_added as SolEvent>::decode_log(&log.inner){
            log::info!("VcAdded event:\n{:?}", event.reserialize());
            return Ok(());
        }
    }
    Err(IssuerError::OtherError("no VcAdded event found in the receipt".to_owned()))

}