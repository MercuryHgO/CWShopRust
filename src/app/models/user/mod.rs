mod auth;

use crate::repository::db::SqlxError;
use core::fmt;
use std::any::Any;
use actix_web::cookie::time::error::Format;
use lib_utils::validation::{self, validate_rules, Rules, Validate};
use redis::FromRedisValue;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeKind;
use sqlx::{query_as, Decode, Execute, FromRow, QueryBuilder, Type};
use sqlx::{Pool, Postgres};

// Custom validation rules

#[derive(Debug)]
enum CustomRules {
    LoginCanContain,
}

impl std::fmt::Display for CustomRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self  {
            CustomRules::LoginCanContain => {
                write!(
                    f,
                    "Can contain only letters, numbers, underscores, and periods."
                )
            },
        }
    }
}

impl validation::Rule for CustomRules {}

impl Validate<String> for CustomRules {
    fn validate(&self, value: &String) -> validation::Result<&Self> {
        match self {
            CustomRules::LoginCanContain => {
                let re = r"^[a-zA-Z0-9_.]*$";
                let regex = Regex::new(re).unwrap();
                if regex.is_match(value) {
                    Ok(self)
                } else {
                    Err(validation::Error::RuleNotValidated(self))
                }
            },
        }
    }
}

// Database fields

// Name
#[derive(Debug, Serialize, Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Name(String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl Name {
    pub fn parse(name: String) -> Result<Self, Vec<validation::Error<'static>>> {
        let errors = validate_rules(
            &name,
            &[&Rules::MinLength(3), &Rules::ContainsDidgits(false)],
        )
        .to_vec();

        if errors.is_empty() {
            Ok(Name(name))
        } else {
            Err(errors)
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for Name {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

// Login
#[derive(Debug, Serialize, Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Login(String);

impl fmt::Display for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<String> for Login {
    fn from(value: String) -> Self {
        Login(value)
    }
}

impl Login {
    pub fn parse(login: String) -> Result<Self, Vec<validation::Error<'static>>> {
        let validation_errors = [
            validate_rules(&login, &[&Rules::MinLength(3), &Rules::MaxLength(20)]),
            validate_rules(&login, &[&CustomRules::LoginCanContain]),
        ]
        .concat();

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

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

// Password
#[derive(Debug, Serialize, Deserialize, sqlx::Decode, sqlx::Encode)]
pub struct Password(String);

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<String> for Password {
    // Converts to string WITHOUT VALIDATION
    fn from(value: String) -> Self {
        Password(value)
    }
}

impl Password {
    pub fn parse(password: String) -> Result<Self, Vec<validation::Error<'static>>> {
        let validation_errors = validate_rules(
            &password,
            &[
                &Rules::ContainsLowecaseCharacter(true),
                &Rules::ContainsUppercaseCharacter(true),
                &Rules::ContainsDidgits(true),
                &Rules::ContainsSpecialCharacters(true),
            ],
        )
        .to_vec();

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

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

// Uuid
#[derive(Debug, Deserialize, Clone, sqlx::Decode, sqlx::Encode)]
pub struct Uuid([u8; 16]);

impl sqlx::Type<sqlx::Postgres> for Uuid {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <Vec<u8> as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

// impl<'r,DB: sqlx::Database> sqlx::Decode<'r,DB> for Uuid
// where
//     [u8; 16]: sqlx::Decode<'r, DB>
//  {
//     fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
//         let value = <[u8; 16] as sqlx::Decode<DB>>::decode(value)?;

//         // let sliced_value: [u8; 16] = value
//         //     .as_bytes()
//         //     .try_into()
//         //     .expect("Error converting from String to Uuid: UUID must be 16 bytes");

//         Ok(Uuid(value))
//     }
// }

impl From<[u8; 16]> for Uuid {
    fn from(value: [u8; 16]) -> Self {
        Uuid(value)
    }
}

impl From<Uuid> for [u8; 16] {
    fn from(value: Uuid) -> Self {
        value.0
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(value: Uuid) -> Self {
        uuid::Uuid::from_bytes(value.0)
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &uuid::Uuid::from_bytes(self.0).to_string())
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::convert::From<String> for Uuid {
    fn from(value: String) -> Self {
        let bytes: [u8; 16] = value
            .as_bytes()
            .try_into()
            .expect("Error converting from String to Uuid: UUID must be 16 bytes");

        Uuid(bytes)
    }
}

impl From<Vec<u8>> for Uuid {
    fn from(value: Vec<u8>) -> Self {
        let bytes: [u8; 16] = value
            .try_into()
            .expect("Error converting from String to Uuid: UUID must be 16 bytes");
            
        Uuid(bytes) }
}

impl Uuid {
    pub fn parse(uuid: uuid::Uuid) -> Self {
        Uuid(uuid.into_bytes())
    }
}

// Database type
#[derive(Serialize, Deserialize, FromRow)]
pub(crate) struct User {
    pub id: Uuid,
    pub name: Name,
    pub login: Login,
    pub password: Password,
}

pub struct UserSearch {
    pub id: Option<Uuid>,
    pub name: Option<Name>,
    pub login: Option<Login>,
    pub password: Option<Password>,
    
}

pub trait UserRepository<T: sqlx::Database> {
    async fn fetch_all_users(db: &Pool<T>) -> Result<Vec<User>, SqlxError>;
    async fn fetch_user(db: &Pool<T>, user_id: Uuid) -> Result<User, SqlxError>;

    async fn create_user(db: &Pool<T>, user: User) -> Result<(), SqlxError>;
    async fn create_many_users(db: &Pool<T>, users: Vec<User>) -> Result<(), SqlxError>;

    async fn delete_user(db: &Pool<T>, user_id: Uuid) -> Result<(), SqlxError>;
    async fn delete_many_users(db: &Pool<T>, users_id: Vec<Uuid>) -> Result<(), SqlxError>;

    async fn patch_user(db: &Pool<T>, user: User) -> Result<(), SqlxError>;
    async fn patch_many_users(db: &Pool<T>, users: Vec<User>) -> Result<(), SqlxError>;

    async fn get_user(db: &Pool<T>, user: UserSearch) -> Result<User,SqlxError>;

}

pub struct Builder {
    id: Option<Uuid>,
    name: Option<Name>,
    login: Option<Login>,
    password: Option<Password>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            id: None,
            name: None,
            login: None,
            password: None,
        }
    }

    pub fn id(&mut self, id: Uuid) {
        self.id = Some(id);
    }
    pub fn name(&mut self, name: Name) { self.name = Some(name);
    }
    pub fn login(&mut self, login: Login) {
        self.login = Some(login);
    }
    pub fn password(&mut self, password: Password) {
        self.password = Some(password);
    }

    pub fn try_get(self) -> Option<User> {
        Some(User {
            name: self.name?,
            id: self.id?,
            login: self.login?,
            password: self.password?,
        })
    }
}

impl UserRepository<Postgres> for User {
    async fn fetch_all_users(db: &Pool<Postgres>) -> Result<Vec<User>, SqlxError> {
        // sqlx::query_as::<_,User>("SELECT id, first_name, last_name FROM users")
        //     .fetch_all(db)
        //     .await
        Ok(sqlx::query_as!(User, "SELECT * FROM users",)
            .fetch_all(db)
            .await?)
    }

    async fn create_user(db: &Pool<Postgres>, user: User) -> Result<(), SqlxError> {
        // sqlx::query_as::<_,User>("INSERT INTO users (id, name, login, password) VALUES $1, $2, $3, $4")
        //     .bind(user.id)
        //     .bind(user.name)
        //     .bind(user.login)
        //     .bind(user.password)
        //     .fetch_one(db)
        //     .await
        sqlx::query_as!(
            User,
            "INSERT INTO users (id, name, login, password) VALUES ($1, $2, $3, $4)",
            &user.id.0,
            &user.name.0,
            &user.login.0,
            &user.password.0
        )
        .execute(db)
        .await?;
        Ok(())
    }

    async fn fetch_user(db: &Pool<Postgres>, user_id: Uuid) -> Result<User, SqlxError> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(db)
                .await?,
        )
    }

    async fn create_many_users(db: &Pool<Postgres>, users: Vec<User>) -> Result<(), SqlxError> {
        let mut tx = db.begin().await?;

        for user in users {
            sqlx::query!(
                "INSERT INTO users (id, name, login, password) VALUES ($1, $2, $3, $4)",
                &user.id.0,
                &user.name.0,
                &user.login.0,
                &user.password.0,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_user(db: &Pool<Postgres>, user_id: Uuid) -> Result<(), SqlxError> {
        sqlx::query!("DELETE FROM users WHERE id = $1", &user_id.0)
            .execute(db)
            .await?;
        Ok(())
    }

    async fn delete_many_users(db: &Pool<Postgres>, users_id: Vec<Uuid>) -> Result<(), SqlxError> {
        let mut tx = db.begin().await?;

        for user_id in users_id {
            sqlx::query!("DELETE FROM users WHERE id = $1", &user_id.0)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn patch_user(db: &Pool<Postgres>, user: User) -> Result<(), SqlxError> {
        sqlx::query!(
            "UPDATE users SET name = $2, login = $3, password = $4 WHERE id = $1",
            &user.id.0,
            &user.name.0,
            &user.login.0,
            &user.password.0
        )
        .execute(db)
        .await?;
        Ok(())
    }

    async fn patch_many_users(db: &Pool<Postgres>, users: Vec<User>) -> Result<(), SqlxError> {
        let mut tx = db.begin().await?;

        for user in users {
            sqlx::query!(
                "UPDATE users SET name = $2, login = $3, password = $4 WHERE id = $1",
                &user.id.0,
                &user.name.0,
                &user.login.0,
                &user.password.0,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_user(db: &Pool<Postgres>, user: UserSearch) -> Result<User,SqlxError> {
        let mut query = QueryBuilder::new("SELECT id, name, login, password FROM users WHERE");

        let mut first = true;

        if let Some(id) = user.id {
            if !first { query.push(" AND"); } else { first = false }
            query.push(format!(" id = '{}'",id.to_string()));
        }

        if let Some(name) = user.name {
            if !first { query.push(" AND"); } else { first = false }
            query.push(format!(" name = '{}'",name.to_string()));
        }

        if let Some(login) = user.login {
            if !first { query.push(" AND"); } else { first = false }
            query.push(format!(" login = '{}'",login.to_string()));
        }

        if let Some(password) = user.password {
            if !first { query.push(" AND"); } else { first = false }
            query.push(format!(" password = '{}'",password.to_string()));
        }

        Ok(
            query
                .build_query_as::<User>()
                .fetch_one(db)
                .await?
        )
    }

}
