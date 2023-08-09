use actix_web::{web, HttpResponse, Responder, post, get};
use deadpool_postgres::Pool;
use identity_iota::crypto::Ed25519;
use iota_client::crypto::signatures::ed25519::{PublicKey, Signature};
use crate::{dtos::identity_dtos::{ReqVCInitDTO, ReqVCProofsDTO}, 
            services::{issuer_vc::{check_and_clean_holder_requests, register_new_vc}, 
            issuer_vc::create_hash_and_store_vc, issuer_identity::resolve_did}, 
            IssuerState, db::{operations::get_holder_request_by_vc_hash, models::is_empty_request}, utils::extract_pub_key_from_doc};

/// Store did with expiration so that the client should resend the signatures in a short time.
/// Expiration allows to maintain a light db.
/// It is expected that the holder calls the second API (signatures) within a minute.
/// @param req --> holder's did (as string)
/// @param res --> 200, 400, 500
#[post("")]
async fn req_vcinit(
    req_body: web::Json<ReqVCInitDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>) -> impl Responder {
    let resp = match check_and_clean_holder_requests(pool.get_ref().to_owned(), req_body.did.to_string()).await {
        true => {
            // create VC, hash, 
            // if no error store holder request (did, request expiration, VC)
            let holder_request = create_hash_and_store_vc(
                pool.get_ref().to_owned(),
                req_body.did.clone(), 
                issuer_state.get_ref().to_owned())
            .await.unwrap();

            // send back the H(VC)    
            HttpResponse::Ok().body(holder_request.vchash)
        },
        false => {
            HttpResponse::BadRequest().body("Holder has still a pending reauest".to_string())
        },
    };
    resp
}

/// Verifies the SSI signature and fills up the IDSC (also with the pseudo signature).
/// @param req --> vc_hash, ssi_signature, psuedo_signature
/// @param res --> 200, 400, 500
#[post("")]
async fn req_vc_proofs(
    req_body: web::Json<ReqVCProofsDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>
) -> impl Responder {
    // read the request from the DB 
    let holder_request = get_holder_request_by_vc_hash(&pool.get().await.unwrap(), req_body.vc_hash.clone()).await.unwrap();
    // first check request is valid (anti replay, the hash serves as nonce)
    let resp = match is_empty_request(holder_request.clone()) {
        true => { // request is valid
            // resolve DID Doc and extract public key
            let holder_did_document = resolve_did(issuer_state.issuer_account.client().clone(), holder_request.did.clone()).await.unwrap();
            let holder_pub_key = extract_pub_key_from_doc(holder_did_document.clone());
            
            let key = PublicKey::try_from_bytes(<[u8; Ed25519::PUBLIC_KEY_LENGTH]>::try_from(holder_pub_key).unwrap()).unwrap();
            match key.verify(
                &Signature::from_bytes(<[u8; Ed25519::SIGNATURE_LENGTH]>::try_from(req_body.ssi_signature.as_bytes()).unwrap()), 
                holder_request.vchash.as_bytes()
            ) {
                true => {
                    register_new_vc(
                        issuer_state.get_ref().to_owned(), 
                        holder_request.vc, 
                        holder_request.vchash, 
                        req_body.pseudo_signature.clone(), 
                        holder_request.did
                    ).await;
                    HttpResponse::Ok().body("body".to_string())
                },
                false => {
                    HttpResponse::BadRequest().body("Invalid ssi_signature".to_string())
                },
            }
        },
        false => { // request is not valid
            HttpResponse::BadRequest().body("Invalid ssi_signature".to_string())
        },
    };
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
            .service(req_vc_proofs)
            .service(echo_api)
    );
}