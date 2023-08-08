use actix_web::{web, HttpResponse, Responder, post, get};
use deadpool_postgres::Pool;
use crate::{dtos::identity_dtos::ReqVCInitDTO, services::{issuer_vc::check_and_clean_holder_requests, idsc_wrappers::get_free_vc_id}, IssuerState};

///
/// Store did with expiration so that the client should resend the signatures in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the second API (signatures) within a minute.
/// @param req --> holder's did (as string)
/// @param res --> 200, 400, 500
///
#[post("")]
async fn req_vcinit(
    req_body: web::Json<ReqVCInitDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>) -> impl Responder {
    let resp = match check_and_clean_holder_requests(pool.get_ref().to_owned(), req_body.did.to_string()).await {
        true => {
            // get VC id from IDSC
            let free_vc_id = get_free_vc_id(issuer_state.idsc_instance.clone(), issuer_state.eth_client.clone()).await;
            HttpResponse::Ok().body(free_vc_id.to_string())
            // create VC
            // hash the created VC
            // if no error store holder request (did, request expiration, VC)
            // send back the H(VC)    

            // HttpResponse::Ok().body("1".to_string())
        },
        false => {
            HttpResponse::BadRequest().body("Holder has still a pending reauest".to_string())
            // return error (400 status code)
        },
    };
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