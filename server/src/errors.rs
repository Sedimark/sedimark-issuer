// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use deadpool_postgres::PoolError;
use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum IssuerError {

    // Services Errors
    #[error("Client has still a pending request")]
    ChallengePendingError,
    #[error("Challenge expired")]
    ChallengeExpired,
    #[error("Invalid or pending request")]
    InvalidOrPendingRequestError,
    #[error("Holder request does not exist")]
    NonExistingRequestError,
    #[error("Invalid identity signature")]
    InvalidIdentitySignatureError,
    
    // Iota Errors
    #[error("Identity Iota Error")]
    IdentityIotaError(#[from] identity_iota::iota::Error),
    #[error("Iota Client Error")]
    IotaClientError(#[from] iota_sdk::client::Error),
    #[error("Iota DID Error")]
    IotaDidError(#[from] identity_iota::did::Error),
    #[error("Verification method for ethereum address verification not found")]
    EthMethodNotFound,
    #[error("Verification method type is not EcdsaSecp256k1RecoveryMethod2020")]
    InvalidVerificationMethodType,
    // Smart Contracts Errors
    #[error("Public key recovery error")]
    SignatureError(#[from] alloy::primitives::SignatureError),
    #[error("Address recovery error")]
    AddressRecoveryError,
    #[error("Contract error: {0}")]
    ContractError(String),
    #[error("Smart Contract address recovery Error")]
    ContractAddressRecoveryError,
    
    // Database Errors
    #[error("Row not found")]   
    RowNotFound,
    #[error("tokio_postgres error")]
    TokioPostgresError(#[from] tokio_postgres::error::Error),
    #[error("tokio_pg_mapper error")]
    TokioPostgresMapperError(#[from] tokio_pg_mapper::Error),
    #[error("Pool error")]
    PoolError(#[from] PoolError),
    
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    #[error("Other error: {0}")]
    OtherError(String),

    #[error("Unknown service error")]
    Unknown,
}

impl ResponseError for IssuerError {

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            IssuerError::ChallengePendingError => StatusCode::TOO_MANY_REQUESTS,
            IssuerError::ChallengeExpired => StatusCode::GONE,
            IssuerError::InvalidOrPendingRequestError => StatusCode::BAD_REQUEST,
            IssuerError::NonExistingRequestError => StatusCode::NOT_FOUND,
            IssuerError::InvalidIdentitySignatureError => StatusCode::BAD_REQUEST,
            IssuerError::IdentityIotaError(_) => StatusCode::INTERNAL_SERVER_ERROR, //TODO: check error code
            IssuerError::IotaClientError(_) => StatusCode::INTERNAL_SERVER_ERROR, //TODO: check error code
            IssuerError::IotaDidError(_) => StatusCode::INTERNAL_SERVER_ERROR, //TODO: check error code
            IssuerError::RowNotFound => StatusCode::NOT_FOUND,
            IssuerError::TokioPostgresError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::TokioPostgresMapperError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::EthMethodNotFound => StatusCode::BAD_REQUEST,
            IssuerError::InvalidVerificationMethodType => StatusCode::BAD_REQUEST,
            IssuerError::SignatureError(_) => StatusCode::BAD_REQUEST,
            IssuerError::AddressRecoveryError => StatusCode::BAD_REQUEST,
            IssuerError::MiddlewareError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::ContractAddressRecoveryError => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::ContractError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IssuerError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}