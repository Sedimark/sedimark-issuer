use anyhow::Result;
use deadpool_postgres::Pool;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Jws, Jwt}, document::verifiable::JwsVerificationOptions, iota::{IotaIdentityClientExt, IotaDID}};
use iota_sdk::U256;

use crate::{db::operations::get_holder_request, utils::iota_utils::setup_client}; 
use crate::db::models::is_empty_request;
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
                    let (credential_jwt, decoded_jwt_credential, credential_id) = create_vc(
                        &holder_document, 
                        &issuer_state.issuer_document,
                        &issuer_state,
                        request_dto.credential_subject
                    ).await?;

                    register_new_vc(
                        &pool,
                        issuer_state, 
                        decoded_jwt_credential,
                        credential_id, 
                        holder_request.nonce,
                        &request_dto.wallet_signature, 
                        &holder_document.id().to_string()
                    ).await.unwrap();
                    Ok((credential_id,credential_jwt))
                },
                Err(_) => Err(IssuerError::InvalidIdentitySignatureError),
            }
        },
        true => Err(IssuerError::NonExistingRequestError),
    };
    resp
}