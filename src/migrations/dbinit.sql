CREATE TABLE identity(
    did text PRIMARY KEY,
    privkey bytea NOT NULL
);

CREATE TABLE holders_requests(
    did text PRIMARY KEY NOT NULL,
    request_expiration text NOT NULL,
    nonce text NOT NULL
);