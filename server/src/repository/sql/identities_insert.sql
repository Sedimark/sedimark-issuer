-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

INSERT INTO identities(did, fragment)
VALUES ($1, $2)
RETURNING $table_fields;