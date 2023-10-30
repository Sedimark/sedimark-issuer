use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
}