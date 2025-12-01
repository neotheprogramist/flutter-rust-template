mod error;

use std::{net::SocketAddr, sync::Arc};

use axum::body::Bytes;
pub use error::ClientError;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Response, StatusCode, body::Incoming};
use hyper_util::rt::TokioIo;
use quinn::{Connection, Endpoint, crypto::rustls::QuicClientConfig};
use serde::de::DeserializeOwned;
pub use server::HealthResponse;
use shared::ALPN_QUIC;
use tokio::io::join;

pub struct QuicClient {
    endpoint: Endpoint,
    conn: Connection,
}

impl QuicClient {
    pub async fn connect(
        bind: SocketAddr,
        server: SocketAddr,
        name: &str,
        cert_pem: &str,
    ) -> Result<Self, ClientError> {
        let cert_der = pem::parse(cert_pem)?.contents().to_vec();
        let crypto = Arc::new(rustls::crypto::ring::default_provider());
        let mut roots = rustls::RootCertStore::empty();
        roots.add(cert_der.into())?;
        let mut tls = rustls::ClientConfig::builder_with_provider(crypto)
            .with_safe_default_protocol_versions()?
            .with_root_certificates(roots)
            .with_no_client_auth();
        tls.alpn_protocols = ALPN_QUIC.iter().map(|&x| x.into()).collect();
        let mut endpoint = Endpoint::client(bind)?;
        endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(tls)?,
        )));
        let conn = endpoint.connect(server, name)?.await?;
        Ok(Self { endpoint, conn })
    }

    pub async fn get(&self, path: impl AsRef<str>) -> Result<Response<Incoming>, ClientError> {
        self.request(Method::GET, path.as_ref(), Bytes::new()).await
    }

    pub async fn post(
        &self,
        path: impl AsRef<str>,
        body: Bytes,
    ) -> Result<Response<Incoming>, ClientError> {
        self.request(Method::POST, path.as_ref(), body).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Bytes,
    ) -> Result<Response<Incoming>, ClientError> {
        let (tx, rx) = self.conn.open_bi().await?;
        let (mut sender, conn) =
            hyper::client::conn::http1::handshake(TokioIo::new(join(rx, tx))).await?;
        let req = hyper::Request::builder()
            .method(method)
            .uri(path)
            .header("Connection", "close")
            .header("content-type", "application/json")
            .body(Full::new(body))?;
        let (r, resp) = futures::join!(conn, sender.send_request(req));
        r?;
        Ok(resp?)
    }

    pub async fn close(self) {
        self.conn.close(0u32.into(), b"done");
        self.endpoint.wait_idle().await;
    }
}

pub async fn collect_body<T: DeserializeOwned>(resp: Response<Incoming>) -> Result<T, ClientError> {
    let (status, bytes) = collect_body_bytes(resp).await?;
    if !status.is_success() {
        return Err(ClientError::Status {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        });
    }
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn collect_body_bytes(
    resp: Response<Incoming>,
) -> Result<(StatusCode, Vec<u8>), ClientError> {
    Ok((
        resp.status(),
        resp.into_body().collect().await?.to_bytes().to_vec(),
    ))
}
