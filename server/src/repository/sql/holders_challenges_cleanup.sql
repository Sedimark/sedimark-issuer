-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

DELETE FROM holders_challenges WHERE expiration < $1;
