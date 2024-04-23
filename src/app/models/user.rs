use sqlx::FromRow;
use core::fmt;

use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;


// Database fields

// Name
#[derive(Debug, Serialize, Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Name(String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl Name {
    pub fn parse(name: String) -> Result<Self,Vec<String>> {
        let mut validation_errors: Vec<String> = Vec::<String>::new();

        // Length
        {
            let re = r"^.{3,}$";
            let regex = Regex::new(re).unwrap();
            if !regex.is_match(&name) {
                validation_errors.push("Name must be minimum 3 charaters long".to_string());
            };
        }

        // Digits
        {
            let re = r"\d";
            let regex = Regex::new(re).unwrap();
            if regex.is_match(&name) {
                validation_errors.push("Name must not contain digits".to_string());
            };
        }

        if validation_errors.is_empty() {
            Ok(Name(name))
        } else {
            Err(validation_errors)
        }

    }
}

impl sqlx::Type<sqlx::Postgres> for Name {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// Login
#[derive(Debug, Serialize,Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Login(String);

impl fmt::Display for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl Login {
    pub fn parse(login: String) -> Result<Self,Vec<String>> {
        let mut validation_errors: Vec<String> = Vec::<String>::new();

        // Min length
        {
            let re = r"^.{3,}$";
            let regex = Regex::new(re).unwrap();
            if !regex.is_match(&login) {
                validation_errors.push("Login must be minimum 3 charaters long".to_string());
            };
        }


        // Max length
        {
            let re = r"^.{0,20}$";
            let regex = Regex::new(re).unwrap();
            if !regex.is_match(&login) {
                validation_errors.push("Login must be maximum 20 charaters long".to_string());
            };
        }

        // Contain
        {
            let re = r"^[a-zA-Z0-9_.]*$";
            let regex = Regex::new(re).unwrap();
            if !regex.is_match(&login) {
                validation_errors.push("Login can contain only letters, numbers, underscores, and periods.".to_string());
            };
        }

        if validation_errors.is_empty() {
            Ok(Login(login))
        } else {
            Err(validation_errors)
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for Login {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// Password
#[derive(Debug, Serialize,Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Password(String);

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl Password {
    pub fn parse(password: String) -> Result<Self,Vec<String>> {
        let mut validation_errors: Vec<String> = Vec::<String>::new();

        {
            let re = r#"^(^.*[a-z]$)
                        (^.*[A-Z]$)
                        (^?=.*\d$)
                        (^.*[@$!%*?&]$)
                        [A-Za-z\d@$!%*?&]
                        {8,20}$"#;
            let regex = Regex::new(re).unwrap();

            if let Some(captures) = regex.captures(&password) {
                // Rule 1: At least one lowercase character
                if captures.get(1).is_none() {
                   validation_errors.push("Password must contain minimum one lowercase letter".to_string());
                }

                // Rule 2: At least one uppercase character
                if captures.get(2).is_none() {
                   validation_errors.push("Password must contain minimum one uppercate character".to_string());
                }

                // Rule 3: At least one digit
                if captures.get(3).is_none() {
                    validation_errors.push("Password must contain at least one digit".to_string());
                }

                // Rule 4: At least one of special characters
                if captures.get(4).is_none() {
                    validation_errors.push("Password must contain at least one of special characters: @ $ ! % * ? &".to_string());
                }
            }
        }

        if validation_errors.is_empty() {
            Ok(Password(password))
        } else {
            Err(validation_errors)
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for Password {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}


// Database type
#[derive(Serialize, Deserialize, FromRow)]
pub(crate) struct User {
    id: Uuid,
    name: Name,
    login: Login,
    password: Password
}

trait UserRepository<T: sqlx::Database> {
    async fn fetch_all_users(db: &Pool<T>) -> Result<Vec<User>,sqlx::Error>;
    async fn fetch_user(db: &Pool<T>, user_id: Uuid) -> Result<User,sqlx::Error>;

    async fn create_user(db: &Pool<T>, user: User) -> Result<User,sqlx::Error>;
    async fn create_many_users(db: &Pool<T>, users: Vec<User>) -> Result<Vec<User>,sqlx::Error>;

    async fn delete_user(db: &Pool<T>, user_id: Uuid) -> Result<(),sqlx::Error>;
    async fn delete_many_users(db: &Pool<T>, users_id: Vec<Uuid>) -> Result<(),sqlx::Error>;

    async fn patch_user(db: &Pool<T>, user: User) -> Result<User,sqlx::Error>;
    async fn patch_many_users(db: &Pool<T>, users: Vec<User>) -> Result<Vec<User>,sqlx::Error>;

}

impl UserRepository<Postgres> for User {
    async fn fetch_all_users(db: &Pool<Postgres>) -> Result<Vec<User>,sqlx::Error> {
        sqlx::query_as::<_,User>("SELECT id, first_name, last_name FROM users")
            .fetch_all(db)
            .await
    }

    async fn create_user(db : &Pool<Postgres>, user: User) -> Result<User,sqlx::Error> {
        sqlx::query_as::<_,User>("INSERT INTO users (id, name, login, password) VALUES $1, $2, $3, $4")
            .bind(user.id)
            .bind(user.name)
            .bind(user.login)
            .bind(user.password)
            .fetch_one(db)
            .await
    }

    async fn fetch_user(db: &Pool<Postgres>, user_id: Uuid) -> Result<User,sqlx::Error> {
        sqlx::query_as::<_,User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await
    }

    async fn create_many_users(db: &Pool<Postgres>, users: Vec<User>) -> Result<Vec<User>,sqlx::Error> {
        let mut created_users = Vec::<User>::new();

        let mut tx = db.begin().await?;

        for user in users {
            let result = sqlx::query("INSERT INTO users (id, name, login, password) VALUES $1, $2, $3, $4")
                .bind(user.id)
                .bind(&user.name)
                .bind(&user.login)
                .bind(&user.password)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => {
                    created_users.push(user);
                },
                Err(err) => {
                    tx.rollback().await?;
                    return Err(err);
                },
            }

        }

        tx.commit().await?;

        Ok(created_users)
    }

    async fn delete_user(db: &Pool<Postgres>, user_id: Uuid) -> Result<(),sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(db)
            .await?;
        Ok(())
    }

    async fn delete_many_users(db: &Pool<Postgres>, users_id: Vec<Uuid>) -> Result<(),sqlx::Error> {
        let mut tx = db.begin().await?;

        for user_id in users_id {
            let result = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => {},
                Err(err) => {
                    tx.rollback().await?;
                    return Err(err);
                },
            }

        }

        tx.commit().await?;

        Ok(())
    }

    async fn patch_user(db: &Pool<Postgres>, user: User) -> Result<User,sqlx::Error> {
        sqlx::query_as::<_,User>("UPDATE users SET name = $2, login = $3, password = $4 WHERE id = $1")
                .bind(user.id)
                .bind(&user.name)
                .bind(&user.login)
                .bind(&user.password)
                .fetch_one(db)
                .await
    }

    async fn patch_many_users(db: &Pool<Postgres>, users: Vec<User>) -> Result<Vec<User>,sqlx::Error> {
        let mut patched_users = Vec::<User>::new();

        let mut tx = db.begin().await?;

        for user in users {
            let result = sqlx::query("UPDATE users SET name = $2, login = $3, password = $4 WHERE id = $1")
                .bind(user.id)
                .bind(&user.name)
                .bind(&user.login)
                .bind(&user.password)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => {
                    patched_users.push(user)
                },
                Err(err) => {
                    tx.rollback().await?;
                    return Err(err);
                },
            }
        }

        tx.commit().await?;

        Ok(patched_users)
    }


}


