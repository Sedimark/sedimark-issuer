use std::env;

use anyhow::Result;
use deadpool_postgres::Pool;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Credential, Jws}, document::verifiable::JwsVerificationOptions, iota::{IotaIdentityClientExt, IotaDID}};
use iota_sdk::client::Client;

use crate::{db::operations::get_holder_request, utils::iota_utils::setup_client}; 
use crate::db::models::is_empty_request;
use crate::errors::IssuerError;
// use ethers::utils::hex::FromHex;

use crate::{
    dtos::identity_dtos::CredentialRequestDTO, 
    services::issuer_vc::{register_new_vc, create_vc}, 
    IssuerState, 
    utils::iota_utils::extract_pub_key_from_doc
};

pub async fn create_credential(
    pool: Pool,
    issuer_state: &IssuerState,
    request_dto: CredentialRequestDTO
) -> Result<Credential, IssuerError>  {
    
    // read the request from the DB 
    let holder_request = get_holder_request(&pool.get().await?, &request_dto.did).await?;
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
                    let vc = create_vc(&holder_document, &issuer_state.issuer_document, &issuer_state).await?;

                    // register_new_vc(
                    //     &pool,
                    //     issuer_state, 
                    //     vc.to_string(), 
                    //     // "0x".to_owned() + &hex::encode(hash_vc(vc.clone())), 
                    //     &holder_request.nonce,
                    //     request_dto.wallet_signature.clone(), 
                    //     &holder_request.did
                    // ).await.unwrap();
                    Ok(vc)
                },
                Err(_) => Err(IssuerError::InvalidIdentitySignatureError),
            }
        },
        true => Err(IssuerError::NonExistingRequestError),
    };
    resp
}