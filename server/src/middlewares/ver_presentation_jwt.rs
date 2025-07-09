// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, Error, HttpMessage};
use actix_web_lab::middleware::Next;

use actix_web::web;
use deadpool_postgres::Pool;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{core::{Object, Timestamp}, credential::{DecodedJwtPresentation, FailFast, Jwt, JwtCredentialValidationOptions, JwtCredentialValidator, JwtCredentialValidatorUtils, JwtPresentationValidationOptions, JwtPresentationValidator, JwtPresentationValidatorUtils, SubjectHolderRelationship}, did::{CoreDID, DID}, document::verifiable::JwsVerificationOptions, iota::IotaDocument, resolver::Resolver, verification::{jws::JwsHeader, jwu::decode_b64_json}};
use std::str::FromStr;

use crate::{errors::IssuerError, repository::operations::HoldersChallengesExt, utils::iota::IotaState};
#[derive(Debug, Clone)]
pub struct VerifiedPresentation{
    pub challenge: String,
    pub vc_id: i64,
    pub did: String
}

pub async fn verify_presentation_jwt(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    log::info!("Hi from start 1. You requested: {}", req.path());
    let db_pool = req.app_data::<web::Data<Pool>>().ok_or(IssuerError::MiddlewareError("no db pool".to_string()))?;
    let iota_state = req.app_data::<web::Data<IotaState>>().ok_or(IssuerError::MiddlewareError("no iota state".to_string()))?;

    log::info!("Resources initialized");
    // Extract the JWT from the request.
    let bearer = req.headers()
        .get("authorization")
        .inspect(|_| log::debug!("header found"))
        .and_then(|header_value| header_value.to_str().ok())
        .ok_or(IssuerError::MiddlewareError("JWT not found in the HTTP header".to_owned()))?;
    

    log::info!("Presentation jwt: {}", bearer);

    let presentation_jwt = Jwt::from(bearer.to_owned());
    // retrieve the header
    let header_b64 = presentation_jwt.as_str().split('.').next().unwrap_or("");
    let header = decode_b64_json::<JwsHeader>(header_b64)
    .map_err(|_e| {IssuerError::MiddlewareError("JWT header not found".to_owned())})?;
    let received_nonce = header.nonce().ok_or(IssuerError::ChallengeExpired)?;

    // ===========================================================================
    // Verifier receives the Verifiable Presentation and verifies it.
    // ===========================================================================

    // The verifier wants the following requirements to be satisfied:
    // - JWT verification of the presentation (including checking the requested challenge to mitigate replay attacks)
    // - JWT verification of the credentials.
    // - The presentation holder must always be the subject, regardless of the presence of the nonTransferable property
    // - The issuance date must not be in the future.

    let mut resolver: Resolver<IotaDocument> = Resolver::new();
    resolver.attach_iota_handler(iota_state.client.clone());

    // Resolve the holder's document.
    let holder_did: CoreDID = JwtPresentationValidatorUtils::extract_holder(&presentation_jwt)
        .map_err(|e| IssuerError::MiddlewareError(e.to_string()))?;
    let holder = resolver.resolve(&holder_did).await
        .map_err(|e| IssuerError::MiddlewareError(e.to_string()))?;

    // Recover the expected challenge from the database
    let pg_client = db_pool.get().await.map_err(IssuerError::PoolError)?;
    log::info!("Holder did: {}", holder.id());
    // check and clean holder requests
    let download_request = pg_client
        .get_challenge(&holder.id().to_string(), &received_nonce.to_owned())
        .await?;

    //check challenge expiration
    let expiration = Timestamp::from_str(&download_request.expiration)
      .map_err(|_| {IssuerError::OtherError("Unsupported timestamp format".to_owned())})?;

    // guard the code returning early if the challenge is expired
    if Timestamp::now_utc() > expiration {
        return Err(IssuerError::ChallengeExpired.into())
    }
    
    log::debug!("Nonce found {:?}", download_request);

    let presentation_verifier_options = JwsVerificationOptions::default().nonce(download_request.challenge.clone());

    // Validate presentation. Note that this doesn't validate the included credentials.
    let presentation_validation_options = JwtPresentationValidationOptions::default().presentation_verifier_options(presentation_verifier_options);
    let presentation: DecodedJwtPresentation<Jwt> = JwtPresentationValidator::with_signature_verifier(
        EdDSAJwsVerifier::default(),
    )
    .validate(&presentation_jwt, &holder, &presentation_validation_options)
    .map_err(|e| IssuerError::MiddlewareError(e.to_string()))?;

    // Concurrently resolve the issuers' documents.
    let jwt_credential = presentation.presentation.verifiable_credential
        .first()
        .ok_or(IssuerError::MiddlewareError("Jwt credential not found".to_owned()))?;

    let issuer_did: CoreDID = JwtCredentialValidatorUtils::extract_issuer_from_jwt(jwt_credential)
        .map_err(|_| IssuerError::MiddlewareError("Issuer DID not found".to_owned()))?;
    
    let issuer_document = resolver.resolve(&issuer_did)
        .await
        .map_err(|e| IssuerError::MiddlewareError(e.to_string()))?;

    log::debug!("Issuer document: {}", issuer_document);

    // Validate the credentials in the presentation.
    let credential_validator: JwtCredentialValidator<EdDSAJwsVerifier> =
        JwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default());
    let validation_options: JwtCredentialValidationOptions = JwtCredentialValidationOptions::default()
        .subject_holder_relationship(holder_did.to_url().into(), SubjectHolderRelationship::AlwaysSubject);


    let decoded_credential = credential_validator
    .validate::<_, Object>(jwt_credential, &issuer_document, &validation_options, FailFast::FirstError)
    .map_err(|e| IssuerError::MiddlewareError(e.to_string()))?;

    log::debug!("Computing the credential id");
    let segments = decoded_credential.credential.id
        .ok_or(IssuerError::MiddlewareError("Credential id not found".to_owned()))?;

    let segments = segments.path_segments()
        .map(|c| c.collect::<Vec<_>>());
    log::debug!("Credential segments found {:?}", segments);
    let credential_id = segments
        .and_then(|parsed| parsed.last().cloned())
        .and_then(|str_segment| str_segment.parse::<i64>().ok())
        .ok_or(IssuerError::MiddlewareError("Credential id not found".to_owned()))?;

    // Since no errors were thrown by `verify_presentation` we know that the validation was successful.
    log::info!("VP successfully validated: {:#?}", presentation.presentation);

    req.extensions_mut()
    .insert( VerifiedPresentation {
        challenge: download_request.challenge.clone(),
        vc_id: credential_id,
        did: holder.id().to_string().clone(),
    });

    let response = next.call(req).await
    .map_err(|e|IssuerError::MiddlewareError(e.to_string()))?;
    Ok(response)

    
}
