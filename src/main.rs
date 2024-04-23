use actix_web::{web::Data, App, HttpServer};
use repository::db::GetPool;
use sqlx::{ postgres::PgPoolOptions, Pool, Postgres};

mod services;
mod repository;
mod app;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let app_state = AppState { db: repository::db::Database::get_pool().await };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            // .service(fetch_users)
    })
    .bind(("127.0.0.1",8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use crate::app::models::user::{Login, Name, Password};

    use super::*;

    #[test]
    fn test_name_parse() {
        // Test valid name
        let name = "John Doe".to_string();
        let result = Name::parse(name.clone());
        assert!(result.is_ok());
        let name_result = result.unwrap();
        assert_eq!(name_result.to_string(), name);

        // Test name with less than minimum length
        let name = "J".to_string();
        let result = Name::parse(name.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Name must be minimum 3 charaters long".to_string());

        // Test name with digits
        let name = "John123".to_string();
        let result = Name::parse(name.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Name must not contain digits".to_string());
    }

    #[test]
    fn test_login_parse() {
        // Test valid login
        let login = "johndoe".to_string();
        let result = Login::parse(login.clone());
        assert!(result.is_ok());
        let login_result = result.unwrap();
        assert_eq!(login_result.to_string(), login);

        // Test login with less than minimum length
        let login = "jo".to_string();
        let result = Login::parse(login.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Login must be minimum 3 charaters long".to_string());

        // Test login with more than maximum length
        let login = "johndoelonglonglonglonglong".to_string();
        let result = Login::parse(login.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Login must be maximum 20 charaters long".to_string());

        // Test login with invalid characters
        let login = "johndoe@".to_string();
        let result = Login::parse(login.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Login can contain only letters, numbers, underscores, and periods.".to_string());
    }

    #[test]
    fn test_password_parse() {
        // Test valid password
        let password = "Password123!".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_ok());
        let password_result = result.unwrap();
        assert_eq!(password_result.to_string(), password); // Error here

        // Test password with less than minimum length
        let password = "Pa".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Password must contain minimum one lowercase letter".to_string());

        // Test password with less than minimum length
        let password = "pass123!".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Password must contain minimus one uppercate character".to_string());

        // Test password with less than minimum length
        let password = "Password123".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Password must contain at least one digit".to_string());

        // Test password with less than minimum length
        let password = "Password123@".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_err());
        // let errors = result.unwrap_err();
        // assert_eq!(errors.len(), 1);
        // assert_eq!(errors[0], "Password must contain at least one of special characters: @ $ ! % * ? &".to_string());
    }
}
