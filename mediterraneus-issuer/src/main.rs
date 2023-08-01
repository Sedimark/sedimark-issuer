use std::env;
use mediterraneus_issuer::issuer::issuer_wallet;
use mediterraneus_issuer::{config::config, issuer::common};
use mediterraneus_issuer::controllers::issuer_controller;
use tokio_postgres::NoTls;
use actix_web::{web, App, HttpServer, middleware::Logger, http};
use actix_cors::Cors;

// #[tokio::main]
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = config::get_db_config();
    let pool = config.create_pool(None, NoTls).unwrap();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    // first create or load issuer's identity.
    let secret_manager = common::setup_secret_manager().await;
    let _client = common::setup_client();
    let client_options = common::setup_client_options();

    let (_account_manager, _account) = issuer_wallet::create_or_load_wallet_account(secret_manager, client_options).await?;


    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin() 
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api")
                .configure(issuer_controller::scoped_config)
            )
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}
