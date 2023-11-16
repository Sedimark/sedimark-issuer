use deadpool_postgres::PoolError;
use actix_web::{HttpResponse, ResponseError};

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
        match *self {
            IssuerError::RowNotFound => HttpResponse::NotFound().finish(),
            IssuerError::PoolError(ref err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
    // fn error_response(&self) -> HttpResponse {
    //     HttpResponse::build(self.status_code())
    //         .insert_header(ContentType::html())
    //         .body(self.to_string())
    // }

    // fn status_code(&self) -> StatusCode {
    //     match *self {
    //         UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    //     }
    // }
}