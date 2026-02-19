use std::time::Duration;

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

pub enum ApiError {
    DatatypeMismatch,
    ConnectionError(reqwest::Error),
    RateLimited(Duration),
    TokenInvalid,
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ConnectionError(err)
    }
}

pub async fn verify_token(token: String) -> Result<String, ApiError> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/validate-token", API_BASE_URL))
        .header("Authorization", format!("Token {}", token))
        .send()
        .await?;

    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let reset = res
            .headers()
            .get("X-RateLimit-Reset-In")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .ok_or(ApiError::DatatypeMismatch)?;

        return Err(ApiError::RateLimited(reset));
    }

    let res: serde_json::Value = res.json().await?;

    if !res["valid"].as_bool().ok_or(ApiError::DatatypeMismatch)? {
        Err(ApiError::TokenInvalid)
    } else {
        Ok(res["user_name"]
            .as_str()
            .ok_or(ApiError::DatatypeMismatch)?
            .to_string())
    }
}
