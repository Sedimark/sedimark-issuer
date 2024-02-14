INSERT INTO identity(did, fragment)
VALUES ($1, $2)
RETURNING $table_fields;