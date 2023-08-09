INSERT INTO holder_request(vchash, did, request_expiration, vc)
VALUES ($1, $2, $3, $4)
RETURNING $table_fields;