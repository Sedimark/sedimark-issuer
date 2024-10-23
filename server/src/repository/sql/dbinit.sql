-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

CREATE TABLE identities (
    did text PRIMARY KEY,
    fragment text NOT NULL
);

CREATE TABLE holders_challenges (
    did_holder          TEXT NOT NULL,
    challenge           TEXT NOT NULL,
    expiration			TEXT NOT NULL
);