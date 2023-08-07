use deadpool_postgres::Pool;
use identity_iota::core::Timestamp;
use crate::db::{operations::{get_holder_request_by_did, remove_holder_request_by_did}, models::is_empty_request};

/// returns @true if the request can continue, @false if the holder has a pending request.
/// If the holder has an expired request, it gets cleared from the DB and the new one
/// will be inserted later by the handler (so the function will return true)
pub async fn check_and_clean_holder_requests(pool: Pool, did: String) -> bool {
    let holder_request = get_holder_request_by_did(&pool.get().await.unwrap(), did.clone()).await.unwrap();
    
    if is_empty_request(holder_request.clone()) == false {
        // request already exists
        // check that it is not expired, if expired remove from db
        let holder_request_timestamp = Timestamp::parse(&holder_request.clone().request_expiration).unwrap();
        if holder_request_timestamp < Timestamp::now_utc() {
            // request expired --> remove it from DB and let handler continue
            remove_holder_request_by_did(&pool.get().await.unwrap(), did).await;
            return true;
        } else {
            // request still not expired --> stop handler from continuing
            return false;
        }
    }
    return true;
}