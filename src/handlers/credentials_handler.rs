use actix_web::{web, HttpResponse, Responder, post};
use deadpool_postgres::Pool;
use ethers::utils::hex::{FromHex};
use identity_iota::crypto::Ed25519;
use iota_client::crypto::signatures::ed25519::{PublicKey, Signature};
use serde_json::json;
use crate::{
    dtos::identity_dtos::{VcIssuingResponse, CredentialRequestDTO}, 
    services::{
        issuer_vc::{register_new_vc, create_vc}, 

        issuer_identity::resolve_did
    }, 
    IssuerState, 
    db::{operations::get_holder_request, models::is_empty_request}, 
    utils::extract_pub_key_from_doc
};

#[post("")]
async fn create_credential (
    req_body: web::Json<CredentialRequestDTO>, 
    pool: web::Data<Pool>,
    issuer_state: web::Data<IssuerState>
) -> impl Responder {
    // read the request from the DB 
    let holder_request = get_holder_request(&pool.get().await.unwrap(), &req_body.did).await.unwrap();
    // first check request is valid (anti replay, the hash serves as nonce)
    let resp = match is_empty_request(holder_request.clone()) {
        false => { // request is not empty ==>  valid
            // resolve DID Doc and extract public key
            let holder_did_document = resolve_did(issuer_state.issuer_account.client().clone(), holder_request.did.clone()).await.unwrap();
            let holder_pub_key = extract_pub_key_from_doc(holder_did_document.clone());
            

            let key = PublicKey::try_from_bytes(<[u8; Ed25519::PUBLIC_KEY_LENGTH]>::try_from(holder_pub_key).unwrap()).unwrap();
            match key.verify(
                &Signature::from_bytes(<[u8; Ed25519::SIGNATURE_LENGTH]>::try_from(Vec::from_hex(req_body.ssi_signature.clone()).unwrap()).unwrap()), 
                holder_request.nonce.as_bytes()
            ) {
                true => {
                    let vc = create_vc(holder_request.did.clone(), issuer_state.get_ref().to_owned()).await.unwrap();

                    match register_new_vc(
                        &pool,
                        issuer_state.get_ref().to_owned(), 
                        vc.to_string(), 
                        // "0x".to_owned() + &hex::encode(hash_vc(vc.clone())), 
                        holder_request.nonce,
                        req_body.pseudo_sign.clone(), 
                        &holder_request.did
                    ).await {
                        Ok(_) => 
                            HttpResponse::Ok()
                            .body(serde_json::to_string::<VcIssuingResponse>(
                            &VcIssuingResponse { 
                                message: "VC issued. In order to activate it contact the IDentity SC.".to_string(), 
                                vc: vc.to_string()
                            })
                            .unwrap())
                        ,
                        Err(err) => {
                            log::info!("{:?}", err.to_string());
                            HttpResponse::BadRequest().body(err.to_string())
                        }
                    }
                },
                false => {
                    HttpResponse::BadRequest().json(json!({"error": "Invalid ssi_signature"}))
                },
            }
        },
        true => { // request is not valid
            HttpResponse::BadRequest().json(json!({"error": "Holder request does not exist"}))
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