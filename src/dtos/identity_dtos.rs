use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CredentialRequestDTO {
    pub did: String,
    pub nonce: String,
    pub identity_signature: String,
    pub wallet_signature: String
}

#[derive(Deserialize, Serialize)]
pub struct AbiDTO {
    pub message: String,
    pub result: String,
    pub status: String
}

#[derive(Deserialize, Serialize)]
pub struct VcIssuingResponse {
    pub message: String,
    pub vc: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PurchaseCredentialRequestDTO {
    pub did: String,
    pub nft_address: String,
    pub challenge: String,
    pub wallet_signature: String,
    pub identity_signature: String
}