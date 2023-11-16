use deadpool_postgres::Pool;
use ethers::utils::hex;
use identity_iota::{credential::{Credential, Subject}, core::Url, iota::IotaDocument};
use iota_sdk::{U256, client::Client};
use serde_json::{json, Value};
use crate::{db::{operations::{remove_holder_request}, models::{is_empty_request, Identity}}, IssuerState, errors::IssuerError, utils::iota_utils::get_vc_id_from_credential};

use crate::services::idsc_wrappers::{get_free_vc_id, register_new_vc_idsc};

async fn issue_vc(holder_did: String, vc_id: U256, issuer_identity: Identity, client: Client) -> Result<Credential, ()> {
    todo!();
    // Create a credential subject indicating the degree earned by Alice.
    // let subject: Subject = Subject::from_json_value(json!({
    //     "id": holder_did,
    //     "name": "Alice",
    //     "degree": {
    //     "type": "BachelorDegree",
    //     "name": "Bachelor of Science and Arts",
    //     },
    //     "GPA": "4.0",
    // })).unwrap();

    // // Build credential using subject above and issuer.
    // let mut credential_id = "https://example.edu/credentials/".to_owned();
    // credential_id.push_str(vc_id.to_owned().to_string().as_str());
    // let mut credential: Credential = CredentialBuilder::default()
    // .id(Url::parse(credential_id).unwrap())
    // .issuer(Url::parse(issuer_identity.did.clone()).unwrap())
    // .type_("MarketplaceCredential")
    // .expiration_date(Timestamp::now_utc().checked_add(Duration::days(365)).unwrap())
    // .issuance_date(Timestamp::now_utc().checked_sub(Duration::days(1)).unwrap())
    // .subject(subject)
    // .build().unwrap();

    // let issuer_doc = resolve_did(client, &issuer_identity.did).await.unwrap();
    // issuer_doc.sign_data(&mut credential, &PrivateKey::try_from(issuer_identity.privkey.clone()).unwrap(), "#key-1", ProofOptions::default()).unwrap();

    // // Validate the credential's signature using the issuer's DID Document, the credential's semantic structure,
    // // that the issuance date is not in the future and that the expiration date is not in the past:
    // CredentialValidator::validate(
    //     &credential,
    //     &issuer_doc,
    //     &CredentialValidationOptions::default(),
    //     FailFast::FirstError,
    // )
    // .unwrap();

    // Ok(credential)
}

pub fn hash_vc(vc: Credential) -> Vec<u8> {
    todo!();

    // ethers::utils::keccak256(vc.to_json_vec().unwrap()).to_vec()
}

pub async fn create_vc(holder_document: &IotaDocument, issuer_document: &IotaDocument, issuer_state: &IssuerState,) -> Result<Credential,IssuerError> {
    todo!();
    // get credential id from Identity Smart Contract
    let vc_id: U256 = get_free_vc_id(issuer_state.idsc_instance.clone(), issuer_state.eth_client.clone()).await;
    
    // let vc = issue_vc(
    //     holder_did.clone(), 
    //     vc_id, 
    //     issuer_state.issuer_identity.clone(), 
    //     issuer_state.issuer_account.client().clone().to_owned()
    // ).await.unwrap();
    // Ok(vc)
}

pub async fn register_new_vc(pool: &Pool, issuer_state: &IssuerState, vc: String, challenge: &String, pseudo_sign: String, holder_did: &String) -> anyhow::Result<(), >{
    todo!();

    // let vc_json: Value = serde_json::from_str(vc.as_str()).unwrap();
    // let credential: Credential = Credential::from_json_value(vc_json).unwrap();

    // let vc_id = get_vc_id_from_credential(credential.clone());

    // // issuer_state.idsc_instance.event_with_filter(Filter::new().event(event_name))
    // register_new_vc_idsc(
    //     issuer_state.idsc_instance.clone(),
    //     issuer_state.eth_client.clone(),
    //     vc_id, 
    //     pseudo_sign, 
    //     holder_did,
    //     credential.expiration_date.unwrap().to_unix(), 
    //     credential.issuance_date.to_unix(), 
    //     challenge
    // ).await?;
    
    // remove_holder_request(&pool.get().await.unwrap(), holder_did).await;
    Ok(())
}