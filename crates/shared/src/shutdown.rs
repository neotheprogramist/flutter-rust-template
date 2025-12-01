use async_signal::{Signal, Signals};
use smol::stream::StreamExt;

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("signal stream closed unexpectedly")]
    StreamClosed,
}

pub async fn signal() -> Result<Signal, ShutdownError> {
    let mut signals = Signals::new([Signal::Term, Signal::Quit, Signal::Int])?;

    let signal = signals.next().await.ok_or(ShutdownError::StreamClosed)??;

    Ok(signal)
}
