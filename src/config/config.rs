// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use deadpool_postgres::{ManagerConfig, RecyclingMethod};

pub fn get_db_config() -> deadpool_postgres::Config {
    dotenv::dotenv().ok();

    let usr = env::var("POSTGRES_USER").expect("$POSTGRES_USER must be set.");
    let pass = env::var("POSTGRES_PASSWORD").expect("$POSTGRES_PASSWORD must be set.");

    let mut config = deadpool_postgres::Config::new();
    config.user = Some(usr);
    config.password = Some(pass);
    config.dbname = Some("identity".into());
    config.host = Some("localhost".into());
    config.port = Some(5433);

    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    config
}