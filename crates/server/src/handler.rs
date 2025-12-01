use axum::Router;
use hyper::{Request, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite, join};
use tower::Service;

use crate::ServerError;

pub async fn handle(incoming: quinn::Incoming, router: Router) -> Result<(), ServerError> {
    let remote_addr = incoming.remote_address();
    tracing::debug!(%remote_addr, "accepting new QUIC connection");

    let connection = incoming.await?;

    let connection_id = connection.stable_id();
    tracing::info!(
        %remote_addr,
        connection_id,
        "QUIC connection established"
    );

    loop {
        let (send, recv) = match connection.accept_bi().await {
            Ok(streams) => streams,
            Err(quinn::ConnectionError::ApplicationClosed(info)) => {
                tracing::debug!(
                    connection_id,
                    code = ?info.error_code,
                    reason = %String::from_utf8_lossy(&info.reason),
                    "connection closed by application"
                );
                break;
            }
            Err(quinn::ConnectionError::ConnectionClosed(info)) => {
                tracing::debug!(
                    connection_id,
                    code = ?info.error_code,
                    reason = %String::from_utf8_lossy(&info.reason),
                    "connection closed by peer"
                );
                break;
            }
            Err(quinn::ConnectionError::LocallyClosed) => {
                tracing::debug!(connection_id, "connection closed locally");
                break;
            }
            Err(quinn::ConnectionError::TimedOut) => {
                tracing::warn!(connection_id, "connection timed out");
                break;
            }
            Err(e) => {
                tracing::error!(connection_id, error = %e, "connection error");
                return Err(e.into());
            }
        };

        let stream = join(recv, send);
        let router_clone = router.clone();

        smol::spawn(async move {
            if let Err(e) = handle_stream(stream, router_clone, connection_id).await {
                tracing::error!(
                    connection_id,
                    error = %e,
                    "stream handler error"
                );
            }
        })
        .detach();
    }

    tracing::info!(connection_id, "connection handler finished");
    Ok(())
}

async fn handle_stream<IO>(
    stream: IO,
    tower_service: Router,
    connection_id: usize,
) -> Result<(), ServerError>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let stream = TokioIo::new(stream);

    let service = hyper::service::service_fn(move |request: Request<Incoming>| {
        tower_service.clone().call(request)
    });

    tracing::trace!(connection_id, "serving HTTP/1.1 connection");

    hyper::server::conn::http1::Builder::new()
        .serve_connection(stream, service)
        .with_upgrades()
        .await?;

    Ok(())
}
