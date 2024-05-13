// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use deadpool_postgres::Client as PostgresClient;
use identity_iota::core::Timestamp;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{repository::models::Identity, errors::IssuerError};

use super::models::HoldersRequests;


#[async_trait]
pub trait IdentityExt {
    async fn get_identity_did(&self) -> Result<Identity, IssuerError>;
    async fn insert_identity_issuer(&self, identity: &Identity) -> Result<Identity, IssuerError>;
}

#[async_trait]
pub trait HoldersRequestsExt {
    async fn get_holder_request(&self, did: &String) -> Result<HoldersRequests, IssuerError>;
    async fn insert_holder_request(&self, did: &String, expiration: Timestamp, nonce: &String) -> Result<HoldersRequests, IssuerError>;
    async fn remove_holder_request(&self, did: &String) ->  Result<(), IssuerError>;
}

#[async_trait]
impl IdentityExt for PostgresClient {
    async fn get_identity_did(&self) -> Result<Identity, IssuerError> {
        let stmt = include_str!("./sql/get_identity_did.sql"); //TODO: folder as env variable
        let stmt = stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&stmt).await?;
    
        match self
        .query_one(&stmt, &[])
        .await{
            Ok(row) => Identity::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
            Err(_) =>  Err(IssuerError::RowNotFound),
        }
    
    }
    
    async fn insert_identity_issuer(&self, identity: &Identity) -> Result<Identity, IssuerError> {
        let _stmt = include_str!("./sql/insert_identity_issuer.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        self.query(
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
}


#[async_trait]
impl HoldersRequestsExt for PostgresClient {

    async fn get_holder_request(&self, did: &String) -> Result<HoldersRequests, IssuerError> {

        let _stmt = include_str!("./sql/get_holder_request.sql");
        let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        match self
        .query_one(&stmt, &[did])
        .await{
            Ok(row) => HoldersRequests::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
            Err(_) =>  Err(IssuerError::RowNotFound),
        }
    
    }

    async fn insert_holder_request(&self, did: &String, expiration: Timestamp, nonce: &String) -> Result<HoldersRequests, IssuerError>{
        let _stmt = include_str!("./sql/insert_holder_request.sql");
        let _stmt = _stmt.replace("$table_fields", &HoldersRequests::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        self.query(
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

    async fn remove_holder_request(&self, did: &String) ->  Result<(), IssuerError> {
        let _stmt = include_str!("./sql/remove_holder_request.sql");
        let stmt = self.prepare(&_stmt).await?;

        self.query(&stmt, &[did]).await?;
        Ok(())
    }
}