// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use deadpool_postgres::Pool;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use ethers::core::types::U256;
use mediterraneus_issuer::contracts::{Identity, VcRevokedFilter};
use mediterraneus_issuer::errors::IssuerError;
use mediterraneus_issuer::handlers::{challenges_handler, credentials_handler};
use mediterraneus_issuer::repository::postgres_repo::init;
use mediterraneus_issuer::utils::configs::{
    Commands, DLTConfig, DatabaseConfig, HttpServerConfig, IssuerConfig, KeyStorageConfig
};
use mediterraneus_issuer::utils::eth::SignerMiddlewareShort;
use mediterraneus_issuer::utils::iota::IotaState;
use std::sync::Arc;
use std::str::FromStr;

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
    let chain_id = args.dlt_config.chain_id;

    // Transactions will be signed with the private key below
    let local_wallet = ((&args
        .issuer_config
        .issuer_private_key
        .value()))
        .parse::<LocalWallet>()?
        .with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(rpc_provider)?;

    let signer: Arc<SignerMiddlewareShort> =
        Arc::new(SignerMiddleware::new(provider, local_wallet));
    let signer_data: web::Data<Arc<SignerMiddlewareShort>> = web::Data::new(signer.clone());

    // Initialize iota_state (client, did, etc.), create or load issuer's identity.
    let iota_state = IotaState::init(&db_pool, args.dlt_config, args.key_storage_config).await?;
    let iota_state_data = web::Data::new(iota_state);
    
    match args.commands {
        None => start_server(db_pool, signer_data, iota_state_data, args.issuer_config, args.http_server_config).await,
        Some(Commands::Revoke { credential }) => revoke_credential(args.issuer_config, signer, credential).await,
    }

}

async fn start_server(db_pool: Pool, 
    signer_data: web::Data<Arc<SignerMiddlewareShort>>, 
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
                .app_data(signer_data.clone())
                .app_data(iota_state_data.clone())
                .app_data(web::Data::new(issuer_config.identity_sc_address.clone()))
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

async fn revoke_credential(issuer_config: IssuerConfig, signer: Arc<SignerMiddlewareShort>, credential_id: i64) -> Result<(), anyhow::Error> {
    // Middleware authenticated the holder, the issuer can delete the account from the Identity SC
    let identity_addr: Address = Address::from_str(&issuer_config.identity_sc_address).map_err(|_| IssuerError::ContractAddressRecoveryError)?;
    let identity_sc = Identity::new(identity_addr, signer);
    
    let call = identity_sc.revoke_vc(U256::from(credential_id));
    let pending_tx = call.send().await.map_err(|err| IssuerError::ContractError(err.to_string()))?;
    let receipt = pending_tx.confirmations(1).await.map_err(|err| IssuerError::ContractError(err.to_string()))?;

    let logs = receipt.ok_or(IssuerError::OtherError("No receipt".to_owned()))?.logs;

    // reading the log   
    for log in logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        };
        // finding the event
        if let Ok(event) =  <VcRevokedFilter as EthEvent>::decode_log(&raw_log){
            log::info!("VcRevoked event:\n{:?}", event);
            return Ok(());
        }
    }
    Err(IssuerError::OtherError("no VcRevoked event found in the receipt".to_owned()).into())
}