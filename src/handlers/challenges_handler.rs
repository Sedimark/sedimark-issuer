// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::get;
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::Deserialize;

use crate::dtos::challenges_dtos::ChallengeResponse;
use crate::errors::errors::ChallengeError;
use crate::services::challenges_service::get_challenge_service;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Params {
    did: String,
}

/// Return a challenge that the client should sign and send back in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the API for creating a credential within a minute.
/// @param res --> 200, 400, 500
#[get("")]
async fn get_challenge(params: web::Query<Params>, pool: web::Data<Pool>,) -> impl Responder {
    let resp = match get_challenge_service(pool.get_ref().to_owned(), &params.did).await {
        Ok(challenge) => {
            HttpResponse::Ok().json(ChallengeResponse {nonce: challenge})
        },
        //TODO: handle this error
        // Err(ChallengeError::ChallengePendingError) => HttpResponse::TooManyRequests().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/challenges")
            .service(get_challenge)
    );
}