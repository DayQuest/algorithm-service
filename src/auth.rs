use axum::body::Body;
use axum::http::header::AUTHORIZATION;
use axum::http::Request;
use axum::http::Response;
use axum::http::StatusCode;
use axum::middleware::Next;
use chrono::Duration;
use chrono::Utc;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use jsonwebtoken::errors::Error;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::net::SocketAddr;

use crate::config::INTERNAL_SECRET_KEY;
use crate::config::JWT_SECRET_KEY;

pub fn gen_token(username: String) -> Result<String, Error> {
    let encoding_key = EncodingKey::from_secret(
        env::var(JWT_SECRET_KEY)
            .expect("Failed to get jwt secret")
            .as_ref(),
    );
    let claims = Claims {
        sub: username,
        iat: Utc::now().timestamp() as usize,
        exp: (Utc::now().timestamp() + Duration::days(365 * 200).num_seconds()) as usize,
        roles: Vec::new(),
    };

    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key)
}

pub fn extract_claims(token: &str) -> Result<Claims, Error> {
    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(
            env::var(JWT_SECRET_KEY)
                .expect("Failed to get jwt secret")
                .as_ref(),
        ),
        &Validation::new(Algorithm::HS256),
    )?
    .claims;

    Ok(claims)
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub iat: usize,
    pub exp: usize,
}

pub async fn jwt_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let auth_header = extract_auth_header(&request).ok_or_else(|| {
        warn_failed_auth(&request, AuthError::InvalidHeader);
        StatusCode::UNAUTHORIZED
    })?;

    let claims = match extract_claims(auth_header) {
        Ok(claims) => claims,
        Err(why) => {
            warn_failed_auth(&request, AuthError::ClaimExtractionError(why));
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub async fn internal_secret_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let auth_header = extract_auth_header(&request).ok_or_else(|| {
        warn_failed_auth(&request, AuthError::InvalidHeader);
        StatusCode::UNAUTHORIZED
    })?;

    if !auth_header.eq(&env::var(INTERNAL_SECRET_KEY).unwrap()) {
        warn_failed_auth(&request, AuthError::WrongInternalSecret(auth_header.to_owned()));
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

fn extract_auth_header<'a>(request: &'a Request<Body>) -> Option<&'a str> {
    request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
}

enum AuthError {
    InvalidHeader,
    ClaimExtractionError(Error),
    WrongInternalSecret(String),
}

fn warn_failed_auth(request: &Request<Body>, error: AuthError) {
    let err_msg = match error {
        AuthError::InvalidHeader => "Invalid auth header".to_string(),
        AuthError::ClaimExtractionError(why) => format!("Failed claim extraction: {}", why),
        AuthError::WrongInternalSecret(used_pw) => format!("Wrong internal secret: `{}`", used_pw),
    };

    match request.extensions().get::<SocketAddr>() {
        Some(socket_addr) => {
            warn!(
                "`{}` failed authentication, err: {}",
                socket_addr.ip(),
                err_msg
            );
        }

        None => {
            warn!(
                "(Unknown Socket Addr) failed authentication, err: {}",
                err_msg
            )
        }
    }
}
