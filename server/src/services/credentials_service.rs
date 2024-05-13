// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use deadpool_postgres::Pool;
use ethers::{types::{Signature, RecoveryMessage, Address}, utils::hex};
use std::str::FromStr;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Jws, Jwt}, document::verifiable::JwsVerificationOptions, iota::{IotaIdentityClientExt, IotaDID}};
use iota_sdk::{U256, types::block::address};

use crate::repository::operations::HoldersRequestsExt;
use crate::utils::iota_utils::setup_client; 
use crate::repository::models::is_empty_request;
use crate::errors::IssuerError;
// use ethers::utils::hex::FromHex;

use crate::{
    dtos::identity_dtos::CredentialRequestDTO, 
    services::issuer_vc::{register_new_vc, create_vc}, 
    IssuerState, 
};

pub async fn create_credential(
    pool: Pool,
    issuer_state: &IssuerState,
    request_dto: CredentialRequestDTO
) -> Result<(U256,Jwt), IssuerError>  {
    
    let pg_client = &pool.get().await?;
    // read the request from the DB 
    let holder_request = pg_client.get_holder_request(&request_dto.did).await?;
    // first check request is valid (anti replay, the hash serves as nonce)
    let resp = match is_empty_request(holder_request.clone()) {
        false => { // request is not empty ==>  valid
            // resolve DID Doc and extract public key
            let client = setup_client().await?;
            let holder_document = client.resolve_did(&IotaDID::parse(holder_request.did)?).await?;
            
            match holder_document.verify_jws(
                &Jws::from(request_dto.identity_signature), // TODO: use auth header
                None,
                &EdDSAJwsVerifier::default(),
                &JwsVerificationOptions::default().nonce(&holder_request.nonce),
            ) {
                Ok(_decoded_jws) => { // TODO: use informations from the jws
                    let (credential_jwt, decoded_jwt_credential, credential_id) = create_vc(
                        &holder_document, 
                        &issuer_state.issuer_document,
                        &issuer_state,
                        request_dto.credential_subject
                    ).await?;

                    log::info!("Wallet sign: {:?}", request_dto.wallet_signature);
                    let wallet_sign = Signature::from_str(request_dto.wallet_signature.as_str())?;
                    log::info!("signature {:?}", wallet_sign);
                    let vm = holder_document.resolve_method("#ethAddress", None).ok_or(IssuerError::EthMethodNotFound)?;

                    vm.type_().to_string().eq("EcdsaSecp256k1RecoverySignature2020").then(|| Some(())).ok_or(IssuerError::InvalidVerificationMethodType)?;
                    let eth_addr = vm.properties()
                    .get("blockchainAccountId")
                    .ok_or(IssuerError::InvalidVerificationMethodType)?
                    .as_str().ok_or(IssuerError::InvalidVerificationMethodType)?
                    .strip_prefix("eip155:1:")
                    .ok_or(IssuerError::InvalidVerificationMethodType)?;

                    log::info!("eth addr: {}", eth_addr);
                    let address: Address = eth_addr.parse().map_err(|_| IssuerError::AddressRecoveryError)?;

                    wallet_sign.verify(holder_request.nonce.clone(), address)?;
                    log::info!("Wallet signature verification success!");
                    
                    register_new_vc(
                        issuer_state, 
                        decoded_jwt_credential,
                        credential_id, 
                        holder_request.nonce,
                        &request_dto.wallet_signature, 
                        &holder_document.id().to_string()
                    ).await.unwrap();

                    pg_client.remove_holder_request(&holder_document.id().to_string()).await?;

                    Ok((credential_id,credential_jwt))
                },
                Err(_) => Err(IssuerError::InvalidIdentitySignatureError),
            }
        },
        true => Err(IssuerError::NonExistingRequestError),
    };
    resp
}