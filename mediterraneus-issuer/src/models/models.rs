use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "identity")] 
pub struct Identity {
    pub did: String,
    pub priv_key: String,
}

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "holder_request")] 
pub struct HolderRequest {
    pub vc_hash: String,
    pub did: String,
    pub request_expiration: String,
    pub vc: String
}