INSERT INTO identity(did, privkey)
VALUES ($1, $2)
RETURNING $table_fields;