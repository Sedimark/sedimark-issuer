// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
}