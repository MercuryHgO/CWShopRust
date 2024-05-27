use crate::PgPoolOptions;

use actix_web::http::StatusCode;
use dotenv::dotenv;
use sqlx::{ error::{DatabaseError, ErrorKind}, Pool, Postgres};

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

// SQLX Wrappers

#[derive(Debug)]
pub struct SqlxError(sqlx::Error);

impl std::fmt::Display for SqlxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

// Into implementations

impl From<sqlx::Error> for SqlxError {
    fn from(value: sqlx::Error) -> Self {
        SqlxError(value)
    }
}

impl From<SqlxError> for sqlx::Error {
    fn from(value: SqlxError) -> Self {
        value.0
    }
}

impl From<SqlxError> for StatusCode {
    fn from(value: SqlxError) -> Self {

        match sqlx::Error::from(value) {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }

    }
}

impl serde::Serialize for SqlxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&format!("{}",self.0))
    }
}


impl std::error::Error for SqlxError { }
