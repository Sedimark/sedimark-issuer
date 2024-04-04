// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::sync::{Arc, RwLock};
use mediterraneus_issuer::repository::postgres_repo::init;
use mediterraneus_issuer::services::issuer_identity::create_or_recover_identity;
use mediterraneus_issuer::utils::iota_utils::create_or_recover_key_storage;
use mediterraneus_issuer::IssuerState;
use mediterraneus_issuer::services::issuer_wallet::setup_eth_wallet;
use mediterraneus_issuer::handlers::{credentials_handler, challenges_handler};
use actix_web::{web, App, HttpServer, middleware::Logger, http};
use actix_cors::Cors;
use ethers::providers::{Provider, Http};
use ethers::middleware::SignerMiddleware;
use ethers::signers::{LocalWallet, Signer};

use clap::{Parser, ArgAction};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether the provider must be configured with a local Hardhat node or not.
    /// By default, the Shimmer Provider will be configured, if no custom url and chain id are provided.
    #[arg(short, long, action=ArgAction::SetTrue)]
    local_node: bool,

    /// Custom json rpc url
    #[arg(long, required=false, requires="chain_id" )]
    custom_node: Option<String>,

    /// Custom chain id
    #[arg(long, required=false, requires="custom_node")]
    chain_id: Option<u64>,
}


#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    log::info!("{}", args.local_node);

    dotenv::dotenv().ok();
    env_logger::init();

    // Initialize database connection pool
    let db_pool = init().await?;

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    // Create or load issuer's identity.
    let (key_storage, secret_manager) = create_or_recover_key_storage().await?;
    let (issuer_identity, issuer_document ) = create_or_recover_identity(&key_storage,  &secret_manager, &db_pool).await?;
        
    let (provider, chain_id) = if args.custom_node.is_some() && args.chain_id.is_some() {
        log::info!("Initializing custom provider");
        (Provider::<Http>::try_from( args.custom_node.unwrap())? , args.chain_id.unwrap())
    } else if args.local_node == false {
        log::info!("Initializing Shimmer provider");
        (Provider::<Http>::try_from(env::var("SHIMMER_JSON_RPC_URL")
            .expect("$SHIMMER_JSON_RPC_URL must be set"))?, 1072u64)
    } else {
        log::info!("Initializing local provider");
        (Provider::<Http>::try_from(env::var("LOCAL_JSON_RPC_URL")
            .expect("$LOCAL_JSON_RPC_URL must be set"))?, 31337u64)
    };
    
    // Transactions will be signed with the private key below
    let eth_wallet = env::var("PRIVATE_KEY")
        .expect("$PRIVATE_KEY must be set")
        .parse::<LocalWallet>()?
        .with_chain_id(chain_id);

    let eth_client = Arc::new(SignerMiddleware::new(provider, eth_wallet.clone()));

    let idsc_instance = setup_eth_wallet(eth_client.clone()).await;

    let key_storage_arc = Arc::new(RwLock::new(key_storage));
    let secret_manager_arc =  Arc::new(RwLock::new( secret_manager.clone() ));

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
            .app_data(web::Data::new(
                IssuerState {
                    secret_manager: secret_manager_arc.clone(),
                    key_storage: key_storage_arc.clone(),
                    issuer_identity: issuer_identity.clone(),
                    issuer_document: issuer_document.clone(),
                    eth_client: eth_client.clone(),
                    idsc_instance: idsc_instance.clone()
                })
            )
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
