use axum::{Router, routing::get};
use futures::{FutureExt, pin_mut, select};
use quinn::Endpoint;
use tower_http::trace::TraceLayer;

mod api;
mod error;
mod handler;

pub use api::{HealthResponse, healthz};
pub use error::{ApiError, ServerError};
pub use handler::handle;

#[derive(Clone)]
pub struct AppState {
    pub version: &'static str,
}

pub async fn serve(endpoint: Endpoint, version: &'static str) -> Result<(), ServerError> {
    let state = AppState { version };
    let router = create_router(state);

    tracing::info!(
        local_addr = %endpoint.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into()),
        "server started, accepting QUIC connections"
    );

    let shutdown = shared::signal().fuse();
    let accept_loop = accept_connections(&endpoint, router).fuse();

    pin_mut!(shutdown);
    pin_mut!(accept_loop);

    select! {
        result = shutdown => {
            let signal = result?;
            tracing::info!(?signal, "shutdown signal received");
        },
        result = accept_loop => result?,
    }

    tracing::info!("initiating graceful shutdown");
    endpoint.close(0u32.into(), b"server shutdown");
    endpoint.wait_idle().await;
    tracing::info!("all connections closed, server shut down");

    Ok(())
}

async fn accept_connections(endpoint: &Endpoint, router: Router) -> Result<(), ServerError> {
    while let Some(incoming) = endpoint.accept().await {
        let router = router.clone();

        smol::spawn(async move {
            if let Err(e) = handle(incoming, router).await {
                tracing::error!(error = %e, "connection handler failed");
            }
        })
        .detach();
    }

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
