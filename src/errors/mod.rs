use deadpool_postgres::PoolError;
use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum IssuerError {

    // Services Errors
    #[error("Client has still a pending request")]
    ChallengePendingError,
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
    // Smart Contracts Errors
    

    // Database Errors
    #[error("Row not found")]   
    RowNotFound,
    #[error("tokio_postgres error")]
    TokioPostgresError(#[from] tokio_postgres::error::Error),
    #[error("tokio_pg_mapper error")]
    TokioPostgresMapperError(#[from] tokio_pg_mapper::Error),
    #[error("Pool error")]
    PoolError(#[from] PoolError),
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
        }
    }
}