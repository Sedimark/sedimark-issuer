// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "identities")] 
pub struct IssuerIdentity {
    pub did: String,
    pub fragment: String,
}

#[derive(Deserialize, PostgresMapper, Serialize, Clone, Debug)]
#[pg_mapper(table = "holders_challenges")] 
pub struct HolderChallenge {
    pub did_holder: String,
    pub challenge: String,
    pub expiration: String,
}