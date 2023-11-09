use std::vec;
use deadpool_postgres::Client as PostgresClient;
use identity_iota::core::Timestamp;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{db::models::Identity, errors::my_errors::MyError};

use super::models::HoldersRequests;

pub async fn get_identity_did(client: &PostgresClient) -> Result<Identity, MyError> {
    let stmt = include_str!("./sql/get_identity_did.sql"); //TODO: folder as env variable
    let stmt = stmt.replace("$table_fields", &Identity::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    let results = match client
        .query_one(&stmt, &[])
        .await {
            Ok(row ) => Identity::from_row_ref(&row).unwrap(),
            Err(db_error) => {
                log::info!("Issuer identity not present in DB: {:?}", db_error);
                Identity{did: "".to_string(), privkey: vec![0]}
            }
        };
    Ok(results)
}

pub async fn insert_identity_issuer(client: &PostgresClient, identity_info: Identity) -> Result<Identity, MyError> {
    let _stmt = include_str!("./sql/insert_identity_issuer.sql");
    let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
            .query(
                &stmt,
                &[
                    &identity_info.did,
                    &identity_info.privkey,
                ],
            )
            .await?
            .iter()
            .map(|row| Identity::from_row_ref(row).unwrap())
            .collect::<Vec<Identity>>()
            .pop()
            .ok_or(MyError::NotFound) // more applicable for SELECTs
}

pub async fn get_holder_request(client: &PostgresClient, did: &String) -> Result<HoldersRequests, MyError> {
    let _stmt = include_str!("./sql/get_holder_request.sql");
    let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let holder_request_row = match client.query_one(
        &stmt, 
        &[did],
    ).await {
        Ok(holder_request) => HoldersRequests::from_row_ref(&holder_request).unwrap(),
        Err(db_error) => {
            log::info!("Issuer identity not present in DB: {:?}", db_error);
            HoldersRequests { did: "".to_string(), request_expiration: "".to_string(), nonce: "".to_string() }
        },
    };
        
    Ok(holder_request_row)
}

pub async fn insert_holder_request(client: &PostgresClient, did: &String, expiration: Timestamp, nonce: &String) -> Result<HoldersRequests, MyError>{
    let _stmt = include_str!("./sql/insert_holder_request.sql");
    let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client.query(
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
    .ok_or(MyError::NotFound) // more applicable for SELECTs
}

pub async fn remove_holder_request(client: &PostgresClient, did: &String) {
    let _stmt = include_str!("./sql/remove_holder_request.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    client.query(&stmt, &[did]).await.unwrap();
}