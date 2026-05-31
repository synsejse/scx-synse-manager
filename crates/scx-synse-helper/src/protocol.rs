use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use rkyv::rancor::Error as RkyvError;
use scx_synse_ipc::{read_frame, write_frame, Request, Response};

use crate::executor::Executor;

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// Run with the default 5-minute idle timeout.
pub async fn run<R, W, E>(input: R, output: W, executor: Arc<E>) -> Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
    E: Executor + ?Sized + 'static,
{
    run_with_timeout(input, output, executor, DEFAULT_IDLE_TIMEOUT).await
}

/// Run the helper protocol. Returns Ok when stdin reaches EOF or the idle
/// timeout fires; returns Err on I/O failure.
pub async fn run_with_timeout<R, W, E>(
    mut input: R,
    mut output: W,
    executor: Arc<E>,
    idle_timeout: Duration,
) -> Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
    E: Executor + ?Sized + 'static,
{
    loop {
        let frame = match tokio::time::timeout(idle_timeout, read_frame(&mut input)).await {
            Ok(Ok(Some(frame))) => frame,
            Ok(Ok(None)) => return Ok(()),       // EOF
            Ok(Err(io)) => return Err(io.into()),
            Err(_elapsed) => return Ok(()),      // watchdog
        };
        let response = match rkyv::from_bytes::<Request, RkyvError>(&frame) {
            Ok(req) => handle(&*executor, req).await,
            Err(err) => Response::Err { message: format!("invalid request: {err}") },
        };
        let payload = rkyv::to_bytes::<RkyvError>(&response)
            .map_err(|e| anyhow!("encoding response: {e}"))?;
        write_frame(&mut output, &payload).await?;
    }
}

async fn handle<E: Executor + ?Sized>(executor: &E, req: Request) -> Response {
    let outcome: Result<()> = match req {
        Request::Ping => Ok(()),
        Request::Apply { scheduler, mode } => executor.apply(&scheduler, mode).await,
        Request::Disable => executor.disable().await,
    };
    match outcome {
        Ok(()) => Response::Ok,
        Err(err) => Response::Err { message: format!("{err:#}") },
    }
}
