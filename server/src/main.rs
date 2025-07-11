// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Deref;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use alloy::network::Ethereum;
use alloy::primitives::U256;
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolEvent;
use deadpool_postgres::Pool;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use lib_issuer::contracts::Identity::{IdentityInstance, VC_Revoked};
use lib_issuer::contracts::{Identity};
use lib_issuer::errors::IssuerError;
use lib_issuer::handlers::{challenges_handler, credentials_handler};
use lib_issuer::repository::postgres_repo::init;
use lib_issuer::utils::configs::{
    Commands, DLTConfig, DatabaseConfig, HttpServerConfig, IssuerConfig, KeyStorageConfig
};

use lib_issuer::utils::iota::IotaState;

use clap::Parser;

/// Issuer command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Issuer configuration
    #[command(flatten)]
    issuer_config: IssuerConfig,

    /// Configuration parameters for the DLT
    #[command(flatten)]
    dlt_config: DLTConfig,

    /// HTTP Server configuration
    #[command(flatten)]
    http_server_config: HttpServerConfig,

    /// Configuration section for the KeyStorage
    #[command(flatten)]
    key_storage_config: KeyStorageConfig,

    /// Database configuration args
    #[command(flatten)]
    database_config: DatabaseConfig,

    #[command(subcommand)]
    commands: Option<Commands>
}


#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize database connection pool
    let db_pool = init(args.database_config).await?;

    // Initialize provider
    let rpc_provider = &args.dlt_config.rpc_provider;

    // Transactions will be signed with the private key below
    let signer = args.issuer_config.issuer_private_key.value().parse::<PrivateKeySigner>()?;
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(rpc_provider.deref().clone());
    provider.client().set_poll_interval(Duration::from_millis(500));
    let provider = DynProvider::<Ethereum>::new(provider);
    let identity_address = args.dlt_config.identity_sc_address;
    let identity_sc = Identity::new(identity_address, provider);

    // Initialize iota_state (client, did, etc.), create or load issuer's identity.
    let iota_state = IotaState::init(&db_pool, args.dlt_config, args.key_storage_config).await?;
    let iota_state_data = web::Data::new(iota_state);
    
    match args.commands {
        None => 
            {
                let identity_sc= web::Data::new(identity_sc);
                start_server(db_pool, identity_sc, iota_state_data, args.issuer_config, args.http_server_config).await
            },
        Some(Commands::Revoke { credential }) => revoke_credential(identity_sc, credential).await,
    }

}

async fn start_server(db_pool: Pool, 
    sc_instance: web::Data<IdentityInstance<DynProvider>>, 
    iota_state_data: web::Data<IotaState>,
    issuer_config: IssuerConfig,
    http_config: HttpServerConfig) 
    -> Result<(), anyhow::Error> {

        log::info!("Starting up on {}:{}", http_config.host_address, http_config.host_port);

        HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin() // TODO: define who is allowed
                .allowed_methods(vec!["GET", "POST", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600);

            App::new()
                .app_data(web::Data::new(db_pool.clone()))
                .app_data(sc_instance.clone())
                .app_data(iota_state_data.clone())
                .app_data(web::Data::new(issuer_config.issuer_url.clone()))
                .service(
                    web::scope("/api")
                        .configure(credentials_handler::scoped_config)
                        .configure(challenges_handler::scoped_config),
                )
                .wrap(cors)
                .wrap(Logger::default())
        })
        .bind((http_config.host_address, http_config.host_port))?
        .run()
        .await
        .map_err(anyhow::Error::from)
}

async fn revoke_credential(identity_sc: IdentityInstance<DynProvider>, credential_id: i64) -> Result<(), anyhow::Error> {    
    let call = identity_sc.revokeVC(U256::from(credential_id));
    let receipt = call
        .gas_price(10_000_000_000)
        .send()
        .await
        .map_err(|err| IssuerError::ContractError(err.to_string()))?
        .get_receipt()
        .await
        .map_err(|err| IssuerError::ContractError(err.to_string()))?;

    // reading the log   
    for log in receipt.logs() {
        // finding the event
        if let Ok(_) =  <VC_Revoked as SolEvent>::decode_log(&log.inner){
            log::info!("VcRevoked event:\n{:?}", log);
            return Ok(());
        }
    }
    Err(IssuerError::OtherError("no VcRevoked event found in the receipt".to_owned()).into())
}