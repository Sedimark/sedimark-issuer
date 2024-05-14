// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use actix_web::{web, HttpResponse, Responder, post, delete};
use deadpool_postgres::Pool;
use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::providers::{Http, Provider};
use ethers::core::types::U256;
use ethers::types::Address;
use crate::contracts::identity::{Identity, VcRevokedFilter};
use crate::dtos::identity_dtos::{CredentialRequestDTO, CredentialIssuedResponse};
use crate::{IotaState, SignerMiddlewareShort};
use crate::errors::IssuerError;
use crate::services::credentials_service::create_credential_service;

// use actix_web_lab::middleware::from_fn;
// use crate::middlewares::ver_presentation_jwt::verify_presentation_jwt;

#[post("/credentials")]
async fn issue_credential (
    req_body: web::Json<CredentialRequestDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IotaState>,
    signer_data: web::Data<Arc<SignerMiddlewareShort>>,
) -> Result<impl Responder, IssuerError> {
    log::info!("Issuing credential...");
    let (credential_id, credential_jwt) = create_credential_service(
        pool.get_ref().to_owned(),
        signer_data.get_ref(),
        issuer_state.get_ref(), 
        req_body.into_inner()
    ).await?; 
    
    let response = CredentialIssuedResponse { 
        message: "Verifiable Credential issued".to_string(),
        issuer_did: issuer_state.get_ref().issuer_identity.did.clone(),
        credential_id: credential_id,
        credential_jwt: credential_jwt
    };
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/credentials/{credential_id}")] //, wrap = "from_fn(verify_presentation_jwt)")]
async fn revoke_credential (
    path: web::Path<i64>,
    eth_provider: web::Data<Arc<SignerMiddlewareShort>>,
) -> Result<impl Responder, IssuerError> {
    log::info!("Revoking credential...");
    let credential_id = path.into_inner();
    let client = eth_provider.get_ref().clone();
    let identity_addr: Address = std::env::var("IDENTITY_SC_ADDRESS").expect("$IDENTITY_SC_ADDRESS must be set").parse().map_err(|_| IssuerError::ContractAddressRecoveryError)?;
    let identity_sc = Identity::new(identity_addr, client);
    
    let call = identity_sc.revoke_vc(U256::from(credential_id));
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
        if let Ok(event) =  <VcRevokedFilter as EthEvent>::decode_log(&raw_log){
            log::info!("VcRevoked event:\n{:?}", event);
            return Ok(HttpResponse::Ok().finish());
        }
    }
    Err(IssuerError::OtherError("no VcRevoked event found in the receipt".to_owned()))
}


pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(issue_credential)
    .service(revoke_credential);

}