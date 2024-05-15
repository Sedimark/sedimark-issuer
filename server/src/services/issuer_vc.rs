// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyhow::Result;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Credential, Subject, CredentialBuilder, Jwt, JwtCredentialValidator, JwtCredentialValidationOptions, FailFast, DecodedJwtCredential}, core::{Url, Timestamp, FromJson, Duration, Object}, iota::IotaDocument, did::DID, storage::{JwkDocumentExt, JwsSignatureOptions}};
use serde_json::json;
use ethers::{abi::{Bytes, RawLog}, contract::EthEvent, utils::hex::FromHex};
use ethers::core::types::U256;

use crate::{contracts::identity::{Identity, VcAddedFilter}, dtos::identity_dtos::CredentialSubject, errors::IssuerError, utils::{eth::SignerMiddlewareShort, iota::MemStorage}};


pub async fn create_credential(
    holder_document: &IotaDocument, 
    issuer_document: &IotaDocument, 
    vc_id: U256,  
    storage_issuer: &MemStorage,
    fragment_issuer: &String,
    credential_subject: CredentialSubject
) -> Result<(Jwt, DecodedJwtCredential)> {
    // Create a credential subject // TODO: fill this from user request
    let subject: Subject = Subject::from_json_value(json!({
        "id": holder_document.id().as_str(),
        "name": credential_subject.name,
        "surname": credential_subject.surname,
        "userOf": "SEDIMARK marketplace"
    }))?;

    // Build credential using subject above and issuer.
    let credential_base_url = "https://example.market/credentials/";  //TODO: define a uri

    let credential: Credential = CredentialBuilder::default()
    .id(Url::parse( format!("{}{}", credential_base_url, vc_id))?)
    .issuer(Url::parse(issuer_document.id().as_str())?)
    .type_("MarketplaceCredential") // TODO: define a type somewhere else
    .expiration_date(Timestamp::now_utc().checked_add(Duration::days(365)).unwrap()) // TODO: define this as a parameter
    .issuance_date(Timestamp::now_utc().checked_sub(Duration::days(1)).unwrap()) //TODO: this solved an error with the eth node time 
    .subject(subject)
    .build()?;

    // Sign the credential
    let credential_jwt: Jwt = issuer_document
    .create_credential_jwt(
      &credential,
      &storage_issuer,
      &fragment_issuer,
      &JwsSignatureOptions::default(),
      None,
    )
    .await?;

    // To ensure the credential's validity, the issuer must validate it before issuing it to the holder

    // Validate the credential's signature using the issuer's DID Document, the credential's semantic structure,
    // that the issuance date is not in the future and that the expiration date is not in the past:
    let decoded_credential: DecodedJwtCredential<Object> =
    JwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default())
    .validate::<_, Object>(
      &credential_jwt,
      &issuer_document,
      &JwtCredentialValidationOptions::default(),
      FailFast::FirstError,
    )?;
    
    Ok((credential_jwt, decoded_credential))
}

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