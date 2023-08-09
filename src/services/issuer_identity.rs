use deadpool_postgres::Pool;
use ethers::types::U256;
use identity_iota::core::{FromJson, Url, ToJson, Timestamp, Duration};
use identity_iota::credential::{Credential, CredentialBuilder, CredentialValidationOptions, CredentialValidator, FailFast, Subject};
use identity_iota::crypto::{KeyPair, KeyType, PrivateKey, ProofOptions};
use identity_iota::iota::{IotaDocument, IotaIdentityClientExt};
use identity_iota::prelude::IotaClientExt;
use identity_iota::verification::{MethodScope, VerificationMethod};
use iota_client::{Client, block::{address::Address, output::AliasOutput}};
use iota_client::secret::SecretManager;
use serde_json::json;
use crate::db::models::HolderRequest;
use crate::db::operations::{self, insert_holder_request};
use crate::db::operations::insert_identity_issuer;
use crate::utils::convert_string_to_iotadid;
use crate::{db::models::Identity, errors::my_errors::MyError};

pub async fn create_identity(client: &Client, wallet_address: Address, secret_manager: &mut SecretManager, pool: Pool) -> Result<Identity, MyError> {
    // check if DID is already available
    let issuer_identity = operations::get_identity_did(&pool.get().await.unwrap()).await?;
    if issuer_identity.did.len() > 0 && issuer_identity.privkey.len() > 0 {
        return Ok(issuer_identity);
    }
    log::info!("Creating new Issuer Identity... {:?}", wallet_address.to_bech32(client.get_bech32_hrp().await.unwrap()));
    // Get the Bech32 human-readable part (HRP) of the network.
    let network_name = client.network_name().await.unwrap();

    // Create a new DID document with a placeholder DID.
    // The DID will be derived from the Alias Id of the Alias Output after publishing.
    let mut document: IotaDocument = IotaDocument::new(&network_name);
    
    // Insert a new Ed25519 verification method in the DID document.
    let keypair: KeyPair = KeyPair::new(KeyType::Ed25519).unwrap();
    let method: VerificationMethod = VerificationMethod::new(document.id().clone(), keypair.type_(), keypair.public(), "#key-1").unwrap();
    document.insert_method(method, MethodScope::VerificationMethod).unwrap();

    // Construct an Alias Output containing the DID document, with the wallet address
    // set as both the state controller and governor.
    let alias_output: AliasOutput = client.new_did_output(wallet_address, document, None).await.unwrap();

    // Publish the Alias Output and get the published DID document.
    let document: IotaDocument = client.publish_did_output(secret_manager, alias_output).await.unwrap();
    println!("Published DID document: {document:#}");

    // Insert new identity in the DB
    let new_issuer_identity = Identity { did: document.id().to_string(), privkey: keypair.private().as_ref().to_vec() };
    insert_identity_issuer(&pool.get().await.unwrap(), new_issuer_identity.clone()).await?;
    Ok(new_issuer_identity)
}  

pub async fn resolve_did(client: Client, holder_did: String) -> Result<IotaDocument, identity_iota::iota::Error> {
    let did = convert_string_to_iotadid(holder_did);
    let resolved_doc = client.resolve_did(&did).await?;

    Ok(resolved_doc)
}

pub async fn create_vc(holder_did: String, vc_id: U256, issuer_identity: Identity, client: Client) -> Result<Credential, ()> {
    // Create a credential subject indicating the degree earned by Alice.
    let subject: Subject = Subject::from_json_value(json!({
        "id": holder_did,
        "name": "Alice",
        "degree": {
        "type": "BachelorDegree",
        "name": "Bachelor of Science and Arts",
        },
        "GPA": "4.0",
    })).unwrap();

    // Build credential using subject above and issuer.
    let mut credential_id = "https://example.edu/credentials/".to_owned();
    credential_id.push_str(vc_id.to_owned().to_string().as_str());
    let mut credential: Credential = CredentialBuilder::default()
    .id(Url::parse(credential_id).unwrap())
    .issuer(Url::parse(issuer_identity.did.clone()).unwrap())
    .type_("MarketplaceCredential")
    .subject(subject)
    .build().unwrap();

    let issuer_doc = resolve_did(client, issuer_identity.did.clone()).await.unwrap();
    issuer_doc.sign_data(&mut credential, &PrivateKey::try_from(issuer_identity.privkey.clone()).unwrap(), "#key-1", ProofOptions::default()).unwrap();

    // Validate the credential's signature using the issuer's DID Document, the credential's semantic structure,
    // that the issuance date is not in the future and that the expiration date is not in the past:
    CredentialValidator::validate(
        &credential,
        &issuer_doc,
        &CredentialValidationOptions::default(),
        FailFast::FirstError,
    )
    .unwrap();

    Ok(credential)
}

pub fn hash_vc(vc: Credential) -> Vec<u8> {
    ethers::utils::keccak256(vc.to_json_vec().unwrap()).to_vec()
}

pub async fn create_hash_and_store_vc(pool: Pool, holder_did: String, vc_id: U256, issuer_identity: Identity, iota_client: Client) -> Result<HolderRequest, MyError> {
    let vc = create_vc(holder_did.clone(), vc_id, issuer_identity, iota_client).await.unwrap();
    let vc_digest = hash_vc(vc.clone());

    let expiration = Timestamp::now_utc().checked_add(Duration::minutes(1)).unwrap();
    insert_holder_request(
        &pool.get().await.unwrap(), 
        vc.clone().to_json().unwrap(), 
        vc_digest.to_json().unwrap().to_string(), 
        holder_did.clone(),
        expiration
    ).await
}