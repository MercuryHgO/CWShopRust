use actix_web::{web::{self, scope}, Responder, Scope};

pub mod user;

pub fn services() -> Scope {
    scope("/user/")
        .service(user::sign_up)
}
