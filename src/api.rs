use std::time::Duration;

use poise::serenity_prelude::Member;
use redis::AsyncTypedCommands;
use reqwest::StatusCode;

use crate::brainzbot::{BrainzContext, BrainzError};

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

#[derive(Debug)]
pub enum ApiError {
    Unexpected,
    ConnectionError(reqwest::Error),
    RateLimited(Duration),
    TokenInvalid,
    OtherStatus(StatusCode),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ConnectionError(err)
    }
}

pub async fn api_request(
    token: &str,
    endpoint: &str,
) -> Result<Option<serde_json::Value>, ApiError> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/{}", API_BASE_URL, endpoint))
        .header("Authorization", format!("Token {}", token))
        .send()
        .await?;

    println!("{}: {:?}", endpoint, res);

    match res.status() {
        StatusCode::OK => Ok(Some(res.json().await?)),
        StatusCode::TOO_MANY_REQUESTS => {
            let reset = res
                .headers()
                .get("X-RateLimit-Reset-In")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .ok_or(ApiError::Unexpected)?;

            Err(ApiError::RateLimited(reset))
        }
        StatusCode::NOT_FOUND => Ok(None),
        StatusCode::UNAUTHORIZED => Err(ApiError::TokenInvalid),
        status => Err(ApiError::OtherStatus(status)),
    }
}

pub async fn verify_token(token: &str) -> Result<String, ApiError> {
    let response = api_request(&token, "/validate-token")
        .await?
        .ok_or(ApiError::Unexpected)?;

    if !response["valid"].as_bool().ok_or(ApiError::Unexpected)? {
        Err(ApiError::TokenInvalid)
    } else {
        Ok(response["user_name"]
            .as_str()
            .ok_or(ApiError::Unexpected)?
            .to_string())
    }
}

pub struct User {
    pub token: String,
    pub username: String,
}

pub async fn get_user(ctx: BrainzContext<'_>, member: Option<Member>) -> Option<User> {
    let mut conn = ctx.data().conn().clone();
    let user_id = member
        .map(|m| m.user.id.get())
        .unwrap_or(ctx.author().id.get());

    let token = conn.get(format!("user:{}:token", user_id)).await.ok()??;
    let username = conn
        .get(format!("user:{}:username", user_id))
        .await
        .ok()??;

    Some(User { token, username })
}
