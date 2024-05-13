// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{web, HttpResponse, Responder, post, delete};
use deadpool_postgres::Pool;
use serde_json::json;
use crate::dtos::identity_dtos::{CredentialRequestDTO, CredentialIssuedResponse};
use crate::IssuerState;
use crate::errors::IssuerError;
use crate::services::credentials_service::create_credential as create_credential_service;

use actix_web_lab::middleware::from_fn;
use crate::middlewares::ver_presentation_jwt::verify_presentation_jwt;

#[post("/credentials")]
async fn create_credential (
    req_body: web::Json<CredentialRequestDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>
) -> Result<impl Responder, IssuerError> {
    let (credential_id, credential_jwt) = create_credential_service(
        pool.get_ref().to_owned(),
        issuer_state.get_ref(), 
        req_body.into_inner()
    ).await?; 
    
    Ok(HttpResponse::Ok()
    .body(serde_json::to_string::<CredentialIssuedResponse>(
        &CredentialIssuedResponse { 
            message: "Verifiable Credential issued".to_string(),
            issuer_did: issuer_state.get_ref().issuer_identity.did.clone(),
            credential_id: credential_id,
            credential_jwt: credential_jwt
        })
    .unwrap()))
}

// TODO: revoke API
#[delete("/credentials/{credential_id}", wrap = "from_fn(verify_presentation_jwt)")]
async fn revoke_credential (
    path: web::Path<i64>,
) -> Result<impl Responder, IssuerError> {
    let credential_id = path.into_inner();
    todo!();
}


pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_credential);
}