use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),

    #[error(transparent)]
    Http(#[from] hyper::Error),

    #[error(transparent)]
    Certificate(#[from] shared::CertificateError),

    #[error(transparent)]
    Shutdown(#[from] shared::ShutdownError),

    #[error(transparent)]
    Tls(#[from] rustls::Error),

    #[error(transparent)]
    QuicConfig(#[from] quinn::crypto::rustls::NoInitialCipherSuite),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("internal server error: {0}")]
    Internal(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Json(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        tracing::error!(error = %self, "API error occurred");

        let body = serde_json::json!({
            "error": message,
            "status": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}
