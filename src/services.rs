use actix_web::{get, post, web::{Data, Json, Path}, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{self, prelude::FromRow};

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateArticleBody {
    pub title: String,
    pub content: String
}


#[derive(Deserialize, FromRow)]
struct Article {
    id: i32,
    title: String,
    content: String,
    created_by: i32,
}



#[get("/users/{id}/articles")]
pub async fn fetch_user_articles(path: Path<i32>) -> impl Responder {
    "Boba".to_string()
}

#[post("/users/{id}/articles")]
pub async fn create_user_article(path: Path<i32>, body: Json<CreateArticleBody>) -> impl Responder {
    "Boba".to_string()
}
