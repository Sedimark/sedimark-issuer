-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT $table_fields 
FROM holders_challenges 
WHERE did_holder=$1
AND challenge=$2;