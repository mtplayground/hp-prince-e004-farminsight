#![allow(dead_code)]

use jsonwebtoken::{
    decode, decode_header,
    jwk::Jwk,
    Algorithm, DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use thiserror::Error;

use crate::{
    config::{AuthSettings, Settings},
    models::user::UserIdentity,
};

pub const SESSION_COOKIE_NAME: &str = "mctai_session";

#[derive(Clone)]
pub struct AuthService {
    db: PgPool,
    client: reqwest::Client,
    settings: AuthSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionClaims {
    pub sub: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub identity: UserIdentity,
    pub is_first_seen: bool,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("auth service configuration is missing")]
    MissingConfig,
    #[error("session cookie is missing")]
    MissingSessionCookie,
    #[error("session token header is missing a key id")]
    MissingKeyId,
    #[error("session signing key was not found")]
    SigningKeyNotFound,
    #[error("session token uses an unsupported signing algorithm")]
    UnsupportedAlgorithm,
    #[error("session claims are missing an email address")]
    MissingEmail,
    #[error("failed to fetch auth service JWKS")]
    JwksFetch(#[from] reqwest::Error),
    #[error("failed to parse auth service JWKS")]
    JwksParse(#[from] serde_json::Error),
    #[error("failed to verify session token")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("failed to persist authenticated user")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Value>,
}

impl AuthService {
    pub fn from_settings(db: PgPool, settings: &Settings) -> Result<Self, AuthError> {
        let auth_settings = settings.auth.clone().ok_or(AuthError::MissingConfig)?;

        Ok(Self {
            db,
            client: reqwest::Client::new(),
            settings: auth_settings,
        })
    }

    pub fn session_token_from_cookie_header<'a>(&self, cookie_header: &'a str) -> Option<&'a str> {
        session_token_from_cookie_header(cookie_header)
    }

    pub fn auth_url(&self) -> &str {
        self.settings.auth_url.as_str()
    }

    pub fn app_token(&self) -> &str {
        self.settings.app_token.as_str()
    }

    pub async fn verify_session_cookie(
        &self,
        cookie_header: &str,
    ) -> Result<SessionClaims, AuthError> {
        let token = self
            .session_token_from_cookie_header(cookie_header)
            .ok_or(AuthError::MissingSessionCookie)?;

        self.verify_session_token(token).await
    }

    pub async fn verify_session_token(&self, token: &str) -> Result<SessionClaims, AuthError> {
        let header = decode_header(token)?;
        let allowed_algorithms = [
            Algorithm::RS256,
            Algorithm::RS384,
            Algorithm::RS512,
            Algorithm::ES256,
            Algorithm::ES384,
        ];
        if !allowed_algorithms.contains(&header.alg) {
            return Err(AuthError::UnsupportedAlgorithm);
        }

        let key_id = header.kid.as_deref().ok_or(AuthError::MissingKeyId)?;
        let jwk = self.fetch_jwk(key_id).await?;
        let decoding_key = DecodingKey::from_jwk(&jwk)?;
        let mut validation = Validation::new(header.alg);
        validation.algorithms = allowed_algorithms.to_vec();
        validation.set_audience(&[self.settings.app_token.as_str()]);
        validation.set_issuer(&[self.settings.auth_url.as_str()]);

        let TokenData { claims, .. } = decode::<SessionClaims>(token, &decoding_key, &validation)?;
        Ok(claims)
    }

    pub async fn upsert_user_from_claims(
        &self,
        claims: &SessionClaims,
    ) -> Result<AuthenticatedUser, AuthError> {
        let email = non_empty(claims.email.as_str()).ok_or(AuthError::MissingEmail)?;
        let name = claims.name.as_deref().and_then(non_empty);
        let picture_url = claims.picture.as_deref().and_then(non_empty);

        let row = sqlx::query(
            r#"
            INSERT INTO users (sub, email, name, picture_url, last_seen_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (sub) DO UPDATE SET
              email = EXCLUDED.email,
              name = EXCLUDED.name,
              picture_url = EXCLUDED.picture_url,
              last_seen_at = NOW()
            RETURNING (xmax = 0) AS is_first_seen
            "#,
        )
        .bind(claims.sub.as_str())
        .bind(email)
        .bind(name)
        .bind(picture_url)
        .fetch_one(&self.db)
        .await?;

        Ok(AuthenticatedUser {
            identity: UserIdentity {
                sub: claims.sub.clone(),
                email: email.to_owned(),
                name: name.map(str::to_owned),
                picture_url: picture_url.map(str::to_owned),
            },
            is_first_seen: row.try_get("is_first_seen")?,
        })
    }

    async fn fetch_jwk(&self, key_id: &str) -> Result<Jwk, AuthError> {
        let jwks = self
            .client
            .get(self.settings.jwks_url.as_str())
            .send()
            .await?
            .error_for_status()?
            .json::<JwksResponse>()
            .await?;

        let key = jwks
            .keys
            .into_iter()
            .find(|key| key.get("kid").and_then(Value::as_str) == Some(key_id))
            .ok_or(AuthError::SigningKeyNotFound)?;

        Ok(serde_json::from_value(key)?)
    }
}

pub fn session_token_from_cookie_header(cookie_header: &str) -> Option<&str> {
    cookie_header.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == SESSION_COOKIE_NAME && !value.is_empty()).then_some(value)
    })
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
