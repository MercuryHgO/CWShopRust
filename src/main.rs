use actix_web::{web::Data, App, HttpServer};
use app::controllers::services;
use repository::db::GetPool;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod app;
mod error;
mod repository;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let app_state = AppState {
        db: repository::db::Database::get_pool().await,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .service(services())
            .service(app::controllers::user::sign_up)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
