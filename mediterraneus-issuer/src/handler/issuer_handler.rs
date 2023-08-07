use actix_web::{web, HttpResponse, Responder, post, get};
use deadpool_postgres::Pool;

use crate::dtos::identity_dtos::ReqVCInitDTO;

///
/// Store did with expiration so that the client should resend the signatures in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the second API (signatures) within a minute.
/// @param req 
/// @param res 
///
#[post("")]
async fn req_vcinit(_req_body: web::Json<ReqVCInitDTO>, _pool: web::Data<Pool>) -> impl Responder {
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