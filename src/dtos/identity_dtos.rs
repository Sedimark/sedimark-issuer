use identity_iota::credential::Jwt;
use iota_sdk::U256;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRequestDTO {
    pub did: String,
    pub nonce: String,
    pub identity_signature: String,
    pub wallet_signature: String,
    pub credential_subject: CredentialSubject
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbiDTO {
    pub message: String,
    pub result: String,
    pub status: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialIssuedResponse {
    pub message: String,
    pub issuer_did: String,
    pub credential_id: U256,
    pub credential_jwt: Jwt
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseCredentialRequestDTO {
    pub did: String,
    pub nft_address: String,
    pub challenge: String,
    pub wallet_signature: String,
    pub identity_signature: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSubject {
    pub name: String,
    pub surname: String
}