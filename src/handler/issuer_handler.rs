use actix_web::{web, HttpResponse, Responder, post, get};
use deadpool_postgres::Pool;

use crate::{dtos::identity_dtos::ReqVCInitDTO, services::issuer_vc::check_and_clean_holder_requests};

///
/// Store did with expiration so that the client should resend the signatures in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the second API (signatures) within a minute.
/// @param req 
/// @param res 
///
#[post("")]
async fn req_vcinit(req_body: web::Json<ReqVCInitDTO>, pool: web::Data<Pool>) -> impl Responder {
    match check_and_clean_holder_requests(pool.get_ref().to_owned(), req_body.did.to_string()).await {
        true => {
            // get VC id from IDSC
            // create VC
            // hash the created VC
            // if no error store holder request (did, request expiration, VC)
            // send back the H(VC)    
        },
        false => {
            // return error (400 status code)
        },
    };
    let resp = HttpResponse::Accepted();
    resp
}

#[post("")]
async fn req_vcproofs() -> impl Responder {
    let resp = HttpResponse::Accepted();
    resp
}

#[get("/{sentence}")]
async fn echo_api(path: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(path.into_inner())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/identity")
            .service(req_vcinit)
            .service(req_vcproofs)
            .service(echo_api)
    );
}