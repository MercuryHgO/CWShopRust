use jsonwebtoken::{decode_header, DecodingKey, EncodingKey, Header, TokenData, Validation};
use redis::Commands;
use serde::{Deserialize, Serialize};


use crate::app;

use super::Uuid;

trait TokenBody {}

#[derive(Debug, Clone, Serialize)]
struct AccessToken(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenBody {
     user_id: Uuid,
}

impl AccessToken {
    pub async fn encode(cache: &mut redis::Connection,key_id: &Uuid, seconds_to_live: i64, body: AccessTokenBody)  ->  Result<Self> 
    {
        let headers: Header = Header {
            kid: Some(key_id.to_string().clone()),
            ..Default::default()
         };

        let key = std::env::var("ACCESS_TOKEN_SECRET")
            .expect("ACCESS_TOKEN_SECRET env must be provided");
        let encoded_key = EncodingKey::from_secret(key.as_bytes());

        let token = jsonwebtoken::encode(
            &headers,
            &body,
            &encoded_key
        )?;
        
        redis::pipe()
            .set(key_id.to_string(), "").ignore()
            .expire(key_id.to_string(),seconds_to_live)
            .query(cache)?;

        Ok(AccessToken(token))
    }

    pub fn decode(&self) -> Result<TokenData<AccessTokenBody>> {
        let key = std::env::var("ACCESS_TOKEN_SECRET")
            .expect("ACCESS_TOKEN_SECRET env must be provided");
        let encoded_key = DecodingKey::from_secret(key.as_bytes());
        
        let decoded_token = jsonwebtoken::decode(
            &self.0,
            &encoded_key,
            &Validation::default()
        )?;

        Ok(decoded_token)
    }

    pub async fn verify(&self, cache: &mut redis::Connection) -> Result<&Self> {
        let header = decode_header(&self.0)?;

        let is_exits: bool = cache
            .exists(header.kid)?;

        if is_exits {
            Ok(self)
        } else {
            Err(Error::InvalidSession)
        }
    }

    pub async fn destroy(&self, cache: &mut redis::Connection) -> Result<&Self> {
        let header = decode_header(&self.0)?;

        cache
            .del(header.kid)?;

        Ok(self)
    }

    pub async fn update(&self,cache: &mut redis::Connection, seconds_to_live: i64) -> Result<&Self> {
        let header = jsonwebtoken::decode_header(&self.0)?;

        cache
            .expire(header.kid, seconds_to_live)?;
        
        Ok(self)
    }

    pub fn parse(token_string: String) -> Result<Self> {
        jsonwebtoken::decode_header(&token_string)?;

        Ok(AccessToken(token_string))
    }
}

#[derive(Debug, Clone, Serialize)]
struct RefreshToken(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshTokenBody {
    user_id: Uuid
}

impl RefreshToken {
    pub async fn encode(cache: &mut redis::Connection, key_id: &Uuid, seconds_to_live: i64, body: RefreshTokenBody)  ->  Result<Self> 
    {
        let headers: Header = Header {
            kid: Some(key_id.to_string()),
            ..Default::default()
         };

        let key = std::env::var("REFRESH_TOKEN_SECRET")
            .expect("REFRESH_TOKEN_SECRET env must be provided");
        let encoded_key = EncodingKey::from_secret(key.as_bytes());

        let token = jsonwebtoken::encode(
            &headers,
            &body,
            &encoded_key
        )?;
        
        redis::pipe()
            .set(key_id.to_string(), "").ignore()
            .expire(key_id.to_string(),seconds_to_live)
            .query(cache)?;

        Ok(RefreshToken(token))
    }

    pub fn decode(&self) -> Result<TokenData<RefreshTokenBody>> {
        let key = std::env::var("REFRESH_TOKEN_SECRET")
            .expect("REFRESH_TOKEN_SECRET env must be provided");
        let encoded_key = DecodingKey::from_secret(key.as_bytes());
        
        let decoded_token = jsonwebtoken::decode(
            &self.0,
            &encoded_key,
            &Validation::default()
        )?;

        Ok(decoded_token)
    }

    pub async fn verify(&self, cache: &mut redis::Connection) -> Result<&Self> {
        let header = decode_header(&self.0)?;

        let is_exists: bool = cache
            .exists(header.kid)?;

        if is_exists {
            Ok(self)
        } else {
            Err(Error::InvalidSession)
        }
    }

    pub async fn destroy(&self, cache: &mut redis::Connection) -> Result<&Self> {
        let header = decode_header(&self.0)?;

        cache
            .del(header.kid)?;

        Ok(self)
    }

    pub async fn update(&self,cache: &mut redis::Connection, seconds_to_live: i64) -> Result<&Self> {
        let header = jsonwebtoken::decode_header(&self.0)?;

        cache
            .expire(header.kid, seconds_to_live)?;
        
        Ok(self)
    }

    pub fn parse(token_string: String) -> Result<Self> {
        jsonwebtoken::decode_header(&token_string)?;

        Ok(RefreshToken(token_string))
    }
}
struct Tokens {
    refresh: RefreshToken,
    access: AccessToken
}

pub trait Authorization<T> {
    async fn genrate_tokens(&self, cache: &mut T) -> Result<Tokens>;
    async fn refresh_tokens(&self, cache: &mut T, tokens: Tokens) -> Result<Tokens>;
}

impl Authorization<redis::Connection> for app::models::user::User {
    async fn genrate_tokens(&self, cache: &mut redis::Connection) -> Result<Tokens> {
        let tokens_id: Uuid = Uuid::parse(uuid::Uuid::new_v4());

        let access_token_body = AccessTokenBody { user_id: self.id.clone() };
        let refresh_token_body = RefreshTokenBody { user_id: self.id.clone() };

        let access = AccessToken::encode(
            cache,
            &tokens_id,
            60*60,
            access_token_body
        ).await?;

        let refresh = RefreshToken::encode(
            cache,
            &tokens_id,
            60*60*24,
            refresh_token_body
        ).await?;

        Ok(
            Tokens { refresh, access }
        )

    }

    async fn refresh_tokens(&self, cache: &mut redis::Connection, tokens: Tokens) -> Result<Tokens> {
        let access_validation_result = tokens.access.verify(cache).await;
        match access_validation_result {
            Ok(access) => {
                access.update(cache, 60*60 ).await?;
                tokens.refresh.update(cache, 60*60*24 ).await?;

                Ok(tokens)
            },
            Err(_) => {
                tokens.refresh
                    .verify(cache).await?;
                
                let tokens = self.genrate_tokens(cache).await?;

                Ok(tokens)
            },
        }
    }
}


// Error
type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub enum Error {
    InvalidToken(jsonwebtoken::errors::ErrorKind),
    InvalidSession,
    RedisError(redis::RedisError)
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        let error = value.into_kind();
        match error {
            _ => Error::InvalidToken(error),
        }
    }
}

impl From<redis::RedisError> for Error {
    fn from(value: redis::RedisError) -> Self {
        Error::RedisError(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidToken(_) => { write!(f, "Invalid token") },
            Error::InvalidSession => { write!(f, "Invalid session") },
            Error::RedisError(error) => { write!(f,"{}",error) },
        }
    }
}

impl std::error::Error for Error {}
