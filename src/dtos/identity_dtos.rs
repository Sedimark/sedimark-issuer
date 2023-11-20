use identity_iota::credential::Jwt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRequestDTO {
    pub did: String,
    pub nonce: String,
    pub identity_signature: String,
    pub wallet_signature: String
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