use axum::{
    Json,
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use std::convert::Infallible;

#[derive(Serialize)]
pub(crate) struct ApiResponse {
    pub status: u16,
    pub message: String,
}

pub(crate) type IoResult<T> = std::io::Result<T>;

pub(crate) fn api_err(status: StatusCode, message: &str) -> Response<Body> {
    (
        status,
        Json(ApiResponse {
            status: status.as_u16(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

pub(crate) fn api_response<T: Serialize>(status: StatusCode, body: T) -> Response<Body> {
    (status, Json(body)).into_response()
}

pub(crate) async fn render_404(_req: Request) -> Result<Response<Body>, Infallible> {
    let url = _req.uri().to_string();
    let message = format!("The requested resource at {url} could not be found.");

    Ok(api_err(StatusCode::NOT_FOUND, &message))
}
