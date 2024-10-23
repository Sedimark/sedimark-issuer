// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::get;
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::Deserialize;
use uuid::Uuid;

use crate::dtos::challenges_dtos::ChallengeResponse;
use crate::errors::IssuerError;
use crate::repository::models::HolderChallenge;
use crate::repository::operations::HoldersChallengesExt;
use identity_iota::core::{Timestamp, Duration};


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Params {
    did: String,
}

/// Return a challenge that the client should sign and send back in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the API for creating a credential within a minute.
/// @param res --> 200, 400, 500
#[get("/challenges")]
async fn get_challenge(
    params: web::Query<Params>, 
    db_pool: web::Data<Pool>,
) -> Result<impl Responder, IssuerError> {
    
    // let challenge = get_challenge_service(pool.get_ref().to_owned(), &params.did).await?;
    // Ok(HttpResponse::Ok().json(ChallengeResponse {nonce: challenge}))

    log::info!("get_challenge");
    let pg_client = db_pool.get().await.map_err(IssuerError::PoolError)?;
    log::info!("{}", params.did);

    // create nonce and store holder request (did, request expiration, nonce)
    let expiration = Timestamp::now_utc().checked_add(Duration::minutes(1)).unwrap();
    // let nonce = "0x".to_owned() + &Uuid::new_v4().simple().to_string();
    let nonce = Uuid::new_v4().to_string();

    let holder_challenge = HolderChallenge { 
        did_holder: params.did.clone(), 
        challenge: nonce.clone(), 
        expiration: expiration.to_rfc3339() 
    };

    log::info!("Download request: {:?}", holder_challenge);
    pg_client.insert_challenge(&holder_challenge).await?;
    
    Ok(HttpResponse::Ok().json(ChallengeResponse {nonce: nonce}))
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(get_challenge);
}