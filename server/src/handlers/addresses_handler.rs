// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{get, web, HttpResponse, Responder};
use identity_iota::core::ToJson;
use serde::Serialize;

use crate::{errors::IssuerError, utils::iota::{IotaState, SCAddresses}};

#[derive(Serialize)]
struct AddressResponse{
    identity: String,
    factory: String,
    fresc: String
}

impl From<&SCAddresses> for AddressResponse{
    fn from(value: &SCAddresses) -> Self {
        Self { identity: value.identity.to_string(), factory: value.factory.to_string(), fresc: value.fresc.to_string() }
    }
}

/// Get SC Addresses managed by the issuer
#[get("/addresses")]
async fn get_addresses(iota_state: web::Data<IotaState>)
-> Result<impl Responder, IssuerError>
{
    let addresses = Into::<AddressResponse>::into(&iota_state.addresses)
        .to_json_value()
        .map_err(|_| IssuerError::OtherError("Address serialization failed".to_owned()))?;
    return Ok(HttpResponse::Ok().json(addresses))
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(get_addresses);
}