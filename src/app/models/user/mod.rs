mod auth;

use crate::repository::db::SqlxError;
use core::fmt;
use lib_utils::validation::{self, validate_rules, Rules, Validate};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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
}

// Uuid
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Decode, sqlx::Encode)]
pub struct Uuid(uuid::Uuid);

impl sqlx::Type<sqlx::Postgres> for Uuid {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <uuid::Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<String> for Uuid {
    fn from(value: String) -> Self {
        let bytes: [u8; 16] = value
            .as_bytes()
            .try_into()
            .expect("Error converting from String to Uuid: UUID must be 16 bytes");

        Uuid(uuid::Builder::from_bytes(bytes).into_uuid())
    }
}

impl Uuid {
    pub fn parse(uuid: uuid::Uuid) -> Self {
        Uuid(uuid)
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

pub trait UserRepository<T: sqlx::Database> {
    async fn fetch_all_users(db: &Pool<T>) -> Result<Vec<User>, SqlxError>;
    async fn fetch_user(db: &Pool<T>, user_id: Uuid) -> Result<User, SqlxError>;

    async fn create_user(db: &Pool<T>, user: User) -> Result<(), SqlxError>;
    async fn create_many_users(db: &Pool<T>, users: Vec<User>) -> Result<(), SqlxError>;

    async fn delete_user(db: &Pool<T>, user_id: Uuid) -> Result<(), SqlxError>;
    async fn delete_many_users(db: &Pool<T>, users_id: Vec<Uuid>) -> Result<(), SqlxError>;

    async fn patch_user(db: &Pool<T>, user: User) -> Result<(), SqlxError>;
    async fn patch_many_users(db: &Pool<T>, users: Vec<User>) -> Result<(), SqlxError>;


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
            &user.id.to_string(),
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
                &user.id.to_string(),
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
        sqlx::query!("DELETE FROM users WHERE id = $1", &user_id.to_string())
            .execute(db)
            .await?;
        Ok(())
    }

    async fn delete_many_users(db: &Pool<Postgres>, users_id: Vec<Uuid>) -> Result<(), SqlxError> {
        let mut tx = db.begin().await?;

        for user_id in users_id {
            sqlx::query!("DELETE FROM users WHERE id = $1", &user_id.to_string())
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn patch_user(db: &Pool<Postgres>, user: User) -> Result<(), SqlxError> {
        sqlx::query!(
            "UPDATE users SET name = $2, login = $3, password = $4 WHERE id = $1",
            &user.id.to_string(),
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
                &user.id.to_string(),
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

}
