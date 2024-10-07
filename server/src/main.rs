// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;
use mediterraneus_issuer::repository::postgres_repo::init;
use mediterraneus_issuer::utils::eth::SignerMiddlewareShort;
use mediterraneus_issuer::utils::iota::IotaState;
use mediterraneus_issuer::handlers::{credentials_handler, challenges_handler};
use actix_web::{web, App, HttpServer, middleware::Logger, http};
use actix_cors::Cors;
use ethers::providers::{Provider, Http};
use ethers::middleware::SignerMiddleware;
use ethers::signers::{LocalWallet, Signer};

use clap::Parser;

/// Issuer command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /* DLT CONFIG */

    /// JSON RPC provider url
    #[arg(short, long, env, required=true)]
    rpc_provider: String,

    /// Chain id
    #[arg(short, long, env, required=true)]
    chain_id: u64,

    /// URL for reaching the DLT node
    #[arg(long, env, required=true)]
    node_url: String,

    /// Faucet API endpoint
    #[arg(long, env, required=true)]
    faucet_api_endpoint: String,

    /* ISSUER CONFIG */

    /// L2 private key that should be used for the issuer
    #[arg(short, long, env, required=true)]
    issuer_private_key: String,

    /// Address of the Identity Smart Contract for the Mediterraneus Protocol 
    #[arg(id="sc_ddress", short, long, env, required=true)]
    identity_sc_address: String,

    /* KEY STORAGE CONFIG */

    /// File path where secrets are stored
    #[arg(long, env, required=true)]
    key_storage_path: String,

    /// Passphrase for unlocking the storage
    #[arg(long, env, required=true)]
    key_storage_password: String,

    /// Mnemonic to be stored in the key storage
    #[arg(long, env, required=true)]
    key_storage_mnemonic: String,

    /* HTTP SERVER SETUP */

    /// Bind address for the http server
    #[arg(long, env, required=true)]
    host_address: String,

    /// Listening port for the http server
    #[arg(long, env, default_value_t=3213)]
    host_port:u16,

    /* DATABASE CONNECTION CONFIGURATION */
    /// Postgres connection username
    #[arg(long, env, required=true)]
    postgres_user: String,
    /// Postgres connection password
    #[arg(long, env, required=true)]
    postgres_password: String,
    /// Postgres connection database
    #[arg(long, env, required=true)]
    postgres_db: String,
    /// Postgres connection port
    #[arg(long, env, required=true)]
    postgres_port: u16,
    /// Postgres max pool size
    #[arg[long, env, default_value_t=16]]
    postgres_pool_max_size: u16
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    let address = args.host_address;
    let port = args.host_port;
    // Initialize database connection pool
    let db_pool = init().await?;

    // Initialize provider
    let rpc_provider =  args.rpc_provider; 
    let chain_id = args.chain_id;

    // Transactions will be signed with the private key below
    let local_wallet = args.issuer_private_key
    .parse::<LocalWallet>()?
    .with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(rpc_provider)?;

    let signer: Arc<SignerMiddlewareShort> = Arc::new(SignerMiddleware::new(provider, local_wallet));
    let signer_data: web::Data<Arc<SignerMiddlewareShort>> = web::Data::new(signer);

    // Initialize iota_state (client, did, etc.), create or load issuer's identity.
    let iota_state = IotaState::init(&db_pool, args.node_url, args.faucet_api_endpoint, args.key_storage_path, args.key_storage_password, args.key_storage_mnemonic).await?;
    let iota_state_data = web::Data::new(iota_state);
    
    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin() // TODO: define who is allowed
        .allowed_methods(vec!["GET", "POST", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600);

        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(signer_data.clone())
            .app_data(iota_state_data.clone())
            .service(web::scope("/api")
                .configure(credentials_handler::scoped_config)
                .configure(challenges_handler::scoped_config)
            )
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}
