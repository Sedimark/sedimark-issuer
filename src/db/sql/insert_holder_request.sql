INSERT INTO holders_requests(did, request_expiration, nonce)
VALUES ($1, $2, $3)
RETURNING $table_fields;