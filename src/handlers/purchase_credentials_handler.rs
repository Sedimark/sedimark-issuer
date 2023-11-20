use actix_web::{web, HttpResponse, Responder, post};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::IssuerState;
use crate::dtos::identity_dtos::PurchaseCredentialRequestDTO;
use crate::services::purchase_credentials_service::create_purchase_credential as create_purchase_credential_service;
use crate::errors::IssuerError;

#[post("")]
async fn create_purchase_credential (
    req_body: web::Json<PurchaseCredentialRequestDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>
) -> Result<impl Responder, IssuerError> {

    let _ = create_purchase_credential_service(
        &pool.get().await.unwrap(),
        req_body.into_inner()
    ).await?;
    Ok(HttpResponse::Ok().finish())
}


pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/purchase-credentials")
            .service(create_purchase_credential)
    );
}