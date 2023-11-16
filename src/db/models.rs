use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "identity")] 
pub struct Identity {
    pub did: String,
    pub fragment: String,
}

#[derive(Deserialize, PostgresMapper, Serialize, Clone, Debug)]
#[pg_mapper(table = "holders_requests")] 
pub struct HoldersRequests {
    pub did: String,
    pub request_expiration: String,
    pub nonce: String
}

pub fn is_empty_request(request: HoldersRequests) -> bool {
    if request.did.len() > 0 && request.request_expiration.len() > 0 && request.nonce.len() > 0 {
        return false;
    }
    return true;
}