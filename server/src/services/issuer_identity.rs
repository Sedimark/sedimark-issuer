// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use anyhow::Result;
use deadpool_postgres::Pool;
use identity_iota::iota::{IotaDocument, IotaDID, IotaIdentityClientExt};
use identity_stronghold::StrongholdStorage;
use iota_sdk::client::Client;
use crate::repository::operations::{self, IdentityExt};
use crate::repository::models::Identity;
use crate::utils::iota::{MemStorage, create_did, setup_client};


pub async fn create_or_recover_identity(key_storage: &MemStorage, stronghold_storage: &StrongholdStorage, pool: &Pool) -> Result<(Identity, IotaDocument)>{
    let client = setup_client().await?;
    let pg_client = &pool.get().await?;
    // check if a did already exists
    match pg_client.get_identity_did().await {
        Ok(identity) => {
            let issuer_document = client.resolve_did(&IotaDID::parse(&identity.did)?).await?;
            Ok((identity, issuer_document))
        },
        Err(_) => {
            log::info!("Creating new identity... ");

            // create a did with a verification method
            let (_, issuer_document, fragment) = create_did(&client, stronghold_storage.as_secret_manager(), key_storage).await?;
            // save the created identity
            let new_issuer_identity = Identity { did: issuer_document.id().to_string(), fragment:  fragment};
            pg_client.insert_identity_issuer(&new_issuer_identity).await?;
            Ok((new_issuer_identity, issuer_document))
        },
    }
}