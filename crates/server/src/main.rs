use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use quinn::{Endpoint, crypto::rustls::QuicServerConfig};
use server::{ServerError, serve};
use shared::{ALPN_QUIC, decode_b64_pem, parse_rustls_from_pem};

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[arg(short, long, env = "SERVER_BIND_ADDR", default_value = "[::1]:8443")]
    bind_addr: SocketAddr,
    #[arg(long, env = "SERVER_CERT_PEM_B64")]
    cert_pem_b64: String,
    #[arg(long, env = "SERVER_KEY_PEM_B64")]
    key_pem_b64: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), ServerError> {
    shared::logging::init();
    smol::block_on(run(Args::parse()))
}

async fn run(args: Args) -> Result<(), ServerError> {
    let (key, cert) = parse_rustls_from_pem(
        &decode_b64_pem(&args.cert_pem_b64)?,
        &decode_b64_pem(&args.key_pem_b64)?,
    )?;
    let mut tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)?;
    tls.alpn_protocols = ALPN_QUIC.iter().map(|&x| x.into()).collect();
    serve(
        Endpoint::server(
            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls)?)),
            args.bind_addr,
        )?,
        VERSION,
    )
    .await
}
