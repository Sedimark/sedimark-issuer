// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use deadpool_postgres::{ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls;

pub static SQL_PATH: &str = "../../sql";

pub async fn init() -> Result<Pool> {
    log::info!("init database");

    let pg_usr = std::env::var("POSTGRES_USER").expect("$POSTGRES_USER must be set.");
    let pg_pass = std::env::var("POSTGRES_PASSWORD").expect("$POSTGRES_PASSWORD must be set.");
    let dbname = std::env::var("POSTGRES_DB").expect("$POSTGRES_PASSWORD must be set.");
    let pg_host = std::env::var("PG.HOST").expect("$PG.HOST must be set.");
    let pg_port = std::env::var("PG.PORT").expect("$PG.PORT must be set.").parse::<u16>()?;

    let mut config = deadpool_postgres::Config::new();
    config.user = Some(pg_usr);
    config.password = Some(pg_pass);
    config.dbname = Some(dbname);
    config.host = Some(pg_host);
    config.port = Some(pg_port);

    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    let pool = config.create_pool(None, NoTls)?;
    log::info!("pool database");
    Ok(pool)
}   