use actix_web::{web, HttpResponse, Responder, post};
use deadpool_postgres::Pool;
use serde_json::json;
use crate::dtos::identity_dtos::{CredentialRequestDTO, VcIssuingResponse};
use crate::IssuerState;
use crate::errors::IssuerError;
use crate::services::credentials_service::create_credential as create_credential_service;

#[post("")]
async fn create_credential (
    req_body: web::Json<CredentialRequestDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>
) -> impl Responder {
    let resp = match create_credential_service(
        pool.get_ref().to_owned(),
        issuer_state.get_ref(), 
        req_body.into_inner()
    ).await {
        Ok(credential) => {
            HttpResponse::Ok()
            .body(serde_json::to_string::<VcIssuingResponse>(
            &VcIssuingResponse { 
                message: "VC issued. In order to activate it contact the IDentity SC.".to_string(), 
                vc: credential.to_string()
            })
            .unwrap())
        },
        Err(error) => match error {
            IssuerError::InvalidOrPendingRequestError => HttpResponse::BadRequest().json(json!({"error": error.to_string()})),
            _ => HttpResponse::InternalServerError().finish(),
        },
    };
    resp
}

// TODO: revoke API (must be admin api to let only issuer revoke a VC)


pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/credentials")
            // .service(request_credential)
            // .service(activate_credential)
            .service(create_credential)

    );
}