use std::env;
use postgres::{Client, NoTls, Error};
use anyhow::Ok;
use actix_web::{web, App, HttpServer, middleware::Logger};

// #[tokio::main]
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    let usr = env::var("POSTGRES_USER").expect("$POSTGRES_USER must be set.");
    let pass = env::var("POSTGRES_PASSWORD").expect("$POSTGRES_PASSWORD must be set.");
    log::info!("Starting up on {}:{}", address, port);

    let conn_string = &String::from("host=".to_owned() + &address + " user=" + &usr + " password=" + &pass + " port=5433 dbname=identity");
    eprintln!("connection string: {}", conn_string);
    // Connect to the database.
    let mut client = Client::connect(&conn_string, NoTls).unwrap();

    eprintln!("PostgreSql connection successfull!");
    Ok(())
    // HttpServer::new(move || {
    //     App::new()
    //         .app_data(web::Data::new(client))
    //         .service(web::scope("/api"))
    //         .wrap(Logger::default())
    // })
    // .bind((address, port))?
    // .run()
    // .await.map_err(anyhow::Error::from)
}
