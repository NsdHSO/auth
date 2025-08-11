use crate::components::auth::AuthService;
use crate::components::tokens::TokensService;
use crate::components::users::UsersService;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Local;
use dotenv::dotenv;
use env_logger::{Builder, Env};
use listenfd::ListenFd;
use std::env;
use crate::components::config::ConfigService;

mod components;
mod db;
mod entity;
mod http_response;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let conn: sea_orm::DatabaseConnection = db::config::init()
        .await
        .expect("Failed to initialize database connection"); // Initialize connection here

    Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] {} {} - {}",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();

    let data_base_conn = conn.clone();
    let user_service = UsersService::new(&data_base_conn.clone());
    let token_service = TokensService::new(&data_base_conn.clone(), &user_service.clone());
    let auth_service = AuthService::new(
        &data_base_conn.clone(),
        &user_service.clone(),
        &token_service.clone(),
    );

    let mut listened = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| origin.as_bytes().starts_with(b"http://"))
            .allowed_origin_fn(|origin, _req| {
                origin.as_bytes().starts_with(b"https://")
                    && origin.to_str().unwrap().contains("vercel")
            })
            .allowed_origin("https://nsdhso.github.io")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(ConfigService::new().clone()))
            .app_data(web::Data::new(data_base_conn.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(token_service.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/v1")
                    .configure(components::users::init_routes)
                    .configure(components::auth::init_routes),
            )
    });

    server = match listened.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Please set host in .env");
            let port = env::var("PORT").expect("Please set port in .env");
            server
                .bind(format!("{host}:{port}"))
                .unwrap_or_else(|_| panic!("host: {host}> Port {port}"))
        }
    };

    server.run().await
}
