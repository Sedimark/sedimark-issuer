use anyhow::Result;
use deadpool_postgres::Client as PostgresClient;

use crate::db::operations::get_holder_request; 
use crate::db::models::is_empty_request;
use crate::dtos::identity_dtos::PurchaseCredentialRequestDTO;
use crate::errors::IssuerError;

pub async fn create_purchase_credential(
    client: &PostgresClient, 
    request_dto: PurchaseCredentialRequestDTO
) -> Result<(), IssuerError>  {

    // read the request from the DB 
    let holder_request = get_holder_request(client, &request_dto.did).await?;
    // first check request is valid (anti replay, the hash serves as nonce)
    match is_empty_request(holder_request.clone()) {
        false => { // valid request
            todo!()
        },
        true => return Err(IssuerError::InvalidOrPendingRequestError), // Return error invalid or expired request
    };

    Ok(())
}