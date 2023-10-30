use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CredentialRequestDTO {
    pub did: String,
    pub nonce: String,
    pub ssi_signature: String,
    pub pseudo_sign: String
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
    pub vc: String,
}