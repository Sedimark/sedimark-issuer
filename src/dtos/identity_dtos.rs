use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ReqVCInitDTO {
    pub did: String
}


#[derive(Deserialize, Serialize)]
pub struct ReqVCProofsDTO {
    pub vc_hash: String,
    pub ssi_signature: String,
    pub pseudo_signature: String
}