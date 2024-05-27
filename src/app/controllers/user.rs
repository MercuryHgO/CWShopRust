use std::{collections::HashMap, fmt::Debug};

use actix_web::{http::StatusCode, post, web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use lib_utils::validation;
use serde::Deserialize;

use crate::{app::models::user::{self, Login, Name, Password, User, UserRepository}, AppState};


#[derive(Deserialize)]
struct SignUpBody {
    login: String,
    password: String,
    name: String
}

#[post("/signup")]
pub async fn sign_up(_req: HttpRequest, body: Json<SignUpBody>, db: Data<AppState>) -> impl Responder {
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


#[derive(Deserialize)]
struct SignInBody {
    login: String,
    password: String,
}

#[post("/signin")]
async fn sign_in(_req: HttpRequest, body: Json<SignInBody>, db: Data<AppState>) -> impl Responder {
    let mut validation_errors: HashMap<String,Vec<validation::Error<'static>>> = HashMap::new();

    let mut login: Option<Login> = None;
    match Login::parse(body.login.to_string()) {
        Ok(val) => { login = Some(val) },
        Err(e) => { validation_errors.insert("Login".to_string(), e); },
    }

    let mut password: Option<Password> = None;
    match Password::parse(body.password.to_string()) {
        Ok(val) => { password = Some(val) },
        Err(e) => { validation_errors.insert("Password".to_string(), e); },
    }

    if !validation_errors.is_empty() {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
                .json(validation_errors)
    }

    let user = User::get_user(
        &db.db,
        user::UserSearch {
            login,
            password,
            id: None,
            name: None
        }
    ).await;

    match user {
        Ok(user) => {
            HttpResponse::build(StatusCode::OK)
                .json(user)
        },
        Err(e) => {
            eprintln!("{:?}",e);

            let err_code: StatusCode = e.into();
           
            match err_code {
                StatusCode::INTERNAL_SERVER_ERROR => HttpResponse::new(err_code),
                _ => HttpResponse::build(err_code)
                        .body("Wrong login or password")
            }
        },
    }

}
