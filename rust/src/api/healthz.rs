use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr, SocketAddr};

pub use client::{HealthResponse, QuicClient, collect_body};

// To mirror an external struct, you need to define a placeholder type with the same definition
#[flutter_rust_bridge::frb(mirror(HealthResponse))]
pub struct _HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Parse(#[from] AddrParseError),
    #[error(transparent)]
    Client(#[from] client::ClientError),
}

#[flutter_rust_bridge::frb(opaque)]
pub struct Client(QuicClient);

impl Client {
    pub async fn connect(
        server_addr: String,
        server_name: String,
        cert_pem: String,
    ) -> Result<Self, ClientError> {
        let addr: SocketAddr = server_addr.parse()?;
        let bind = match addr {
            SocketAddr::V4(_) => SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            SocketAddr::V6(_) => SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
        };
        QuicClient::connect(bind, addr, &server_name, &cert_pem)
            .await
            .map(Self)
            .map_err(Into::into)
    }

    pub async fn healthz(&self) -> Result<HealthResponse, ClientError> {
        let resp = self.0.get("/healthz").await?;
        Ok(collect_body(resp).await?)
    }

    pub async fn close(self) {
        self.0.close().await;
    }
}
