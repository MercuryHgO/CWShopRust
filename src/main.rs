use std::sync::Arc;

use actix_web::{web::Data, App, HttpServer};
use app::controllers::services;
use repository::db::GetPool;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod app;
mod error;
mod repository;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Pool<Postgres>>,
    cache: Arc<redis::Connection>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let app_state = AppState {
        db: Arc::new(repository::db::Database::get_pool().await),
        cache: Arc::new(repository::cache::create_connection().await)
    };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .service(services())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
