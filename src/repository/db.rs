extern crate proc_macro;
use proc_macro::TokenStream;

use crate::PgPoolOptions;

use dotenv::dotenv;
use sqlx::{ Pool, Postgres};

pub struct Database;



pub trait GetPool<T: sqlx::Database> {
    async fn get_pool() -> Pool<T>;
}

impl GetPool<Postgres> for Database {
    async fn get_pool() -> Pool<Postgres> {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Error building a connection pool")
    }
}

