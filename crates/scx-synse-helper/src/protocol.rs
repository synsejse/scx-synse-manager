use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use scx_synse_ipc::{Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
    input: R,
    mut output: W,
    executor: Arc<E>,
    idle_timeout: Duration,
) -> Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
    E: Executor + ?Sized + 'static,
{
    let mut lines = BufReader::new(input).lines();
    loop {
        let next = tokio::time::timeout(idle_timeout, lines.next_line()).await;
        let line = match next {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => return Ok(()),     // EOF
            Ok(Err(io)) => return Err(io.into()),
            Err(_elapsed) => return Ok(()),    // watchdog
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handle(&*executor, req).await,
            Err(err) => Response::Err { message: format!("invalid request: {err}") },
        };
        let mut encoded = serde_json::to_string(&response)?;
        encoded.push('\n');
        output.write_all(encoded.as_bytes()).await?;
        output.flush().await?;
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
