use std::net::SocketAddr;

use clap::Parser;
use client::{ClientError, HealthResponse, QuicClient, collect_body};
use shared::decode_b64_pem;

#[derive(Parser)]
#[command(name = "healthz", about = "Health check client")]
struct Args {
    #[arg(short, long, env = "CLIENT_BIND_ADDR", default_value = "[::]:0")]
    bind_addr: SocketAddr,
    #[arg(short, long, env = "CLIENT_SERVER_ADDR", default_value = "[::1]:8443")]
    server_addr: SocketAddr,
    #[arg(long, env = "CLIENT_CERT_PEM_B64")]
    cert_pem_b64: String,
}

fn main() -> Result<(), ClientError> {
    shared::logging::init();
    smol::block_on(run(Args::parse()))
}

async fn run(args: Args) -> Result<(), ClientError> {
    tracing::info!(server = %args.server_addr, "connecting");
    let cert_pem = decode_b64_pem(&args.cert_pem_b64)?;
    let client =
        QuicClient::connect(args.bind_addr, args.server_addr, "localhost", &cert_pem).await?;
    let response = client.get("/healthz").await?;
    let body: HealthResponse = collect_body(response).await?;
    tracing::info!(body = ?body, "health check ok");
    client.close().await;
    Ok(())
}
