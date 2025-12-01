use hyper::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Certificate(#[from] shared::CertificateError),
    #[error(transparent)]
    Pem(#[from] pem::PemError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),
    #[error(transparent)]
    Tls(#[from] rustls::Error),
    #[error(transparent)]
    QuicConfig(#[from] quinn::crypto::rustls::NoInitialCipherSuite),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("request failed with status {status}: {body}")]
    Status { status: StatusCode, body: String },
}
