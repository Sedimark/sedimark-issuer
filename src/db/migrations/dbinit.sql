-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

CREATE TABLE identity(
    did text PRIMARY KEY,
    fragment text NOT NULL
);

CREATE TABLE holders_requests(
    did text PRIMARY KEY NOT NULL,
    request_expiration text NOT NULL,
    nonce text NOT NULL
);