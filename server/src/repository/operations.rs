// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use deadpool_postgres::Client as PostgresClient;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{repository::models::IssuerIdentity, errors::IssuerError};

use super::models::HolderChallenge;


#[async_trait]
pub trait IssuerIdentityExt {
    async fn get_identity_did(&self) -> Result<IssuerIdentity, IssuerError>;
    async fn insert_identity_issuer(&self, identity: &IssuerIdentity) -> Result<IssuerIdentity, IssuerError>;
}

#[async_trait]
pub trait HoldersChallengesExt {
    async fn get_challenge(&self, did: &String, nonce: &String) -> Result<HolderChallenge, IssuerError>;
    async fn insert_challenge(&self, holder_challenge: &HolderChallenge) -> Result<HolderChallenge, IssuerError>;
    async fn remove_challenge(&self, did: &String) ->  Result<(), IssuerError>;
}

#[async_trait]
impl IssuerIdentityExt for PostgresClient {
    async fn get_identity_did(&self) -> Result<IssuerIdentity, IssuerError> {
        let stmt = include_str!("./sql/identities_get.sql"); //TODO: folder as env variable
        let stmt = stmt.replace("$table_fields", &IssuerIdentity::sql_table_fields());
        let stmt = self.prepare(&stmt).await?;
    
        match self
        .query_one(&stmt, &[])
        .await{
            Ok(row) => IssuerIdentity::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
            Err(_) =>  Err(IssuerError::RowNotFound),
        }
    
    }
    
    async fn insert_identity_issuer(&self, identity: &IssuerIdentity) -> Result<IssuerIdentity, IssuerError> {
        let _stmt = include_str!("./sql/identities_insert.sql");
        let _stmt = _stmt.replace("$table_fields", &IssuerIdentity::sql_table_fields());
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
        .map(|row| IssuerIdentity::from_row_ref(row).unwrap())
        .collect::<Vec<IssuerIdentity>>()
        .pop()
        .ok_or(IssuerError::RowNotFound) // more applicable for SELECTs
    }
}


#[async_trait]
impl HoldersChallengesExt for PostgresClient {

    async fn get_challenge(&self, did: &String, nonce: &String) -> Result<HolderChallenge, IssuerError> {

        let _stmt = include_str!("./sql/holders_challenges_get.sql");
        let _stmt = _stmt.replace("$table_fields", &HolderChallenge::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        match self
        .query_one(&stmt, &[did, nonce])
        .await{
            Ok(row) => HolderChallenge::from_row_ref(&row).map_err(|e| IssuerError::from(e)),
            Err(_) =>  Err(IssuerError::RowNotFound),
        }
    
    }

    async fn insert_challenge(&self, holder_challenge: &HolderChallenge) -> Result<HolderChallenge, IssuerError>{
        let _stmt = include_str!("./sql/holders_challenges_insert.sql");
        let _stmt = _stmt.replace("$table_fields", &HolderChallenge::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        self.query(
            &stmt,
            &[
                &holder_challenge.did_holder,
                &holder_challenge.expiration,
                &holder_challenge.challenge
            ],
        )
        .await?
        .iter()
        .map(|row| HolderChallenge::from_row_ref(row).unwrap())
        .collect::<Vec<HolderChallenge>>()
        .pop()
        .ok_or(IssuerError::RowNotFound) // more applicable for SELECTs
    }

    async fn remove_challenge(&self, did: &String) ->  Result<(), IssuerError> {
        let _stmt = include_str!("./sql/holders_challenges_remove.sql");
        let stmt = self.prepare(&_stmt).await?;

        self.query(&stmt, &[did]).await?;
        Ok(())
    }
}