// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
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
    /// JSON RPC provider url
    #[arg(short, long, env, required=true)]
    rpc_provider: String,

    /// chain id
    #[arg(short, long, env, required=true)]
    chain_id: u64,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    // Initialize database connection pool
    let db_pool = init().await?;

    // Initialize provider
    let rpc_provider =  args.rpc_provider; 
    let chain_id = args.chain_id;

    // Transactions will be signed with the private key below
    let local_wallet = std::env::var("L2_PRIVATE_KEY").expect("$L2_PRIVATE_KEY must be set")
    .parse::<LocalWallet>()?
    .with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(rpc_provider)?;

    let signer: Arc<SignerMiddlewareShort> = Arc::new(SignerMiddleware::new(provider, local_wallet));
    let signer_data: web::Data<Arc<SignerMiddlewareShort>> = web::Data::new(signer);

    // Initialize iota_state (client, did, etc.), create or load issuer's identity.
    let iota_state = IotaState::init(&db_pool).await?;
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
