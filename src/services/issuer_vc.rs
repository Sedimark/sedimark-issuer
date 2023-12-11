use anyhow::Result;
use deadpool_postgres::Pool;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Credential, Subject, CredentialBuilder, Jwt, JwtCredentialValidator, JwtCredentialValidationOptions, FailFast, DecodedJwtCredential}, core::{Url, Timestamp, FromJson, Duration, Object}, iota::IotaDocument, did::DID, storage::{JwkDocumentExt, JwsSignatureOptions}};
use iota_sdk::U256;
use serde_json::json;
use crate::{db::{operations::{remove_holder_request}, models::{is_empty_request, Identity}}, IssuerState, errors::IssuerError, utils::iota_utils::{get_vc_id_from_credential, MemStorage}, dtos::identity_dtos::CredentialSubject};

use crate::services::idsc_wrappers::{get_free_vc_id, register_new_vc_idsc};

async fn issue_vc(
    holder_document: &IotaDocument, 
    issuer_document: &IotaDocument, 
    vc_id: U256,  
    storage_issuer: &mut MemStorage,
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
    let credential_base_url = "https://example.market/credentials/";

    let credential: Credential = CredentialBuilder::default()
    .id(Url::parse( format!("{}{}", credential_base_url, vc_id))?) //TODO: define a uri
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

    // Before sending this credential to the holder the issuer wants to validate that some properties
    // of the credential satisfy their expectations.

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

// pub fn hash_vc(vc: Credential) -> Vec<u8> {
//     ethers::utils::keccak256(vc.to_json_vec().unwrap()).to_vec()
// }

pub async fn create_vc(
    holder_document: &IotaDocument, 
    issuer_document: &IotaDocument, 
    issuer_state: &IssuerState,
    credential_subject: CredentialSubject
) -> Result<(Jwt, DecodedJwtCredential, U256),IssuerError> {
    // get credential id from Identity Smart Contract
    let credential_id = get_free_vc_id(issuer_state.idsc_instance.clone(), issuer_state.eth_client.clone()).await;
    
    // issue the credential
    let (credential_jwt, decoded_jwt_credential) = issue_vc(
        holder_document,
        issuer_document, 
        credential_id, 
        &mut issuer_state.key_storage.write().unwrap(),
        &issuer_state.issuer_identity.fragment,
        credential_subject
    ).await.unwrap();
    Ok((credential_jwt, decoded_jwt_credential, credential_id))
}

pub async fn register_new_vc(
    pool: &Pool, 
    issuer_state: &IssuerState, 
    decoded_jwt_credential: DecodedJwtCredential, 
    credential_id: U256,
    challenge: String, 
    wallet_sign: &String, 
    holder_did: &String
) -> anyhow::Result<(), >{

    // issuer_state.idsc_instance.event_with_filter(Filter::new().event(event_name))
    register_new_vc_idsc(
        issuer_state.idsc_instance.clone(),
        issuer_state.eth_client.clone(),
        credential_id, 
        wallet_sign, 
        &holder_did,
        decoded_jwt_credential.credential.expiration_date.unwrap().to_unix(), 
        decoded_jwt_credential.credential.issuance_date.to_unix(), 
        challenge
    ).await?;
    
    remove_holder_request(&pool.get().await.unwrap(), holder_did).await?;
    Ok(())
}