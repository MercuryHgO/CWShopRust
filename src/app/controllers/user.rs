use std::collections::HashMap;

use actix_web::{http::StatusCode, post, web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use lib_utils::validation;
use serde::Deserialize;

use crate::{app::models::user::{self, Login, Name, Password, User, UserRepository}, repository::db::SqlxError, AppState};


#[derive(Deserialize)]
struct SignInBody {
    login: String,
    password: String,
    name: String
}

#[post("/signup")]
pub async fn sign_up(_req: HttpRequest, body: Json<SignInBody>, db: Data<AppState>) -> impl Responder {
    let mut validation_errors: HashMap<String,Vec<validation::Error<'static>>> = HashMap::new();
    let mut user_builder = user::Builder::new();

    match Password::parse(body.password.clone()) {
        Ok(parsed_password) => { user_builder.password(parsed_password); },
        Err(err) => { validation_errors.insert("Password".to_string(),err); },
    };
    
    match Login::parse(body.login.clone()) {
        Ok(parsed_login) => { user_builder.login(parsed_login); },
        Err(err) => {validation_errors.insert("Login".to_string(),err); },
    };
        
    match Name::parse(body.name.clone()) {
        Ok(parsed_name) => { user_builder.name(parsed_name); },
        Err(err) => { validation_errors.insert("Name".to_string(),err); },
    };

    if validation_errors.is_empty() {
        user_builder.id(user::Uuid::parse(uuid::Uuid::new_v4()));

        let user: User = user_builder
            .try_get()
            .expect("Error building user");

        let res = User::create_user(&db.db, user).await;

        match res {
            Ok(_) => {
                HttpResponse::new(StatusCode::CREATED)
            },
            Err(err) => {
                let respose_code: StatusCode = err.into();

                match respose_code {
                    StatusCode::CONFLICT => HttpResponse::build(respose_code)
                                                            .body("User already exist"),
                    _ => HttpResponse::new(respose_code)
                }
            },
        }
    } else {
        HttpResponse::build(StatusCode::BAD_REQUEST)
            .json(validation_errors)
    }
}

// fn log_in(arg: Type) -> RetType {
//     unimplemented!();
// }
