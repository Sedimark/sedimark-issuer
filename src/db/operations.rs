use anyhow::Context;
use deadpool_postgres::Client as PostgresClient;
use identity_iota::core::Timestamp;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{db::models::Identity, errors::IssuerError};

use super::models::HoldersRequests;

pub async fn get_identity_did(postgres_client: &PostgresClient) -> Result<Identity, IssuerError> {
    let stmt = include_str!("./sql/get_identity_did.sql"); //TODO: folder as env variable
    let stmt = stmt.replace("$table_fields", &Identity::sql_table_fields());
    let stmt = postgres_client.prepare(&stmt).await?;

    match postgres_client
    .query_one(&stmt, &[])
    .await{
        Ok(row) => Identity::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
        Err(_) =>  Err(IssuerError::RowNotFound),
    }

}

pub async fn insert_identity_issuer(postgres_client: &PostgresClient, identity: &Identity) -> Result<Identity, IssuerError> {
    let _stmt = include_str!("./sql/insert_identity_issuer.sql");
    let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
    let stmt = postgres_client.prepare(&_stmt).await?;

    postgres_client.query(
        &stmt,
        &[
            &identity.did,
            &identity.fragment,
        ],
    )
    .await?
    .iter()
    .map(|row| Identity::from_row_ref(row).unwrap())
    .collect::<Vec<Identity>>()
    .pop()
    .ok_or(IssuerError::RowNotFound) // more applicable for SELECTs
}

pub async fn get_holder_request(
    postgres_client: &PostgresClient, 
    did: &String
) -> Result<HoldersRequests, IssuerError> {

    let _stmt = include_str!("./sql/get_holder_request.sql");
    let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
    let stmt = postgres_client.prepare(&_stmt).await?;

    match postgres_client
    .query_one(&stmt, &[did])
    .await{
        Ok(row) => HoldersRequests::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
        Err(_) =>  Err(IssuerError::RowNotFound),
    }
   
}

pub async fn insert_holder_request(postgres_client: &PostgresClient, did: &String, expiration: Timestamp, nonce: &String) -> Result<HoldersRequests, IssuerError>{
    let _stmt = include_str!("./sql/insert_holder_request.sql");
    let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
    let stmt = postgres_client.prepare(&_stmt).await?;

    postgres_client.query(
        &stmt,
        &[
            &did,
            &expiration.to_rfc3339(),
            &nonce
        ],
    )
    .await?
    .iter()
    .map(|row| HoldersRequests::from_row_ref(row).unwrap())
    .collect::<Vec<HoldersRequests>>()
    .pop()
    .ok_or(IssuerError::RowNotFound) // more applicable for SELECTs
}

pub async fn remove_holder_request(postgres_client: &PostgresClient, did: &String) ->  Result<(), IssuerError> {
    let _stmt = include_str!("./sql/remove_holder_request.sql");
    let stmt = postgres_client.prepare(&_stmt).await?;

    postgres_client.query(&stmt, &[did]).await?;
    Ok(())
}