use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use scx_synse_helper::executor::Executor;
use scx_synse_helper::protocol::run_with_timeout;
use scx_synse_ipc::SchedMode;

struct NoopExecutor;

#[async_trait::async_trait]
impl Executor for NoopExecutor {
    async fn apply(&self, _: &str, _: SchedMode) -> Result<()> { Ok(()) }
    async fn disable(&self) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn exits_after_idle_timeout() {
    // Empty input: stdin stays open but no lines arrive.
    let (input_writer, input_reader) = tokio::io::duplex(64);
    let (output_writer, _output_reader) = tokio::io::duplex(64);
    let exec = Arc::new(NoopExecutor);

    let start = std::time::Instant::now();
    let result = run_with_timeout(
        input_reader,
        output_writer,
        exec,
        Duration::from_millis(150),
    ).await;
    drop(input_writer);

    assert!(result.is_ok(), "watchdog exit should be Ok, got {result:?}");
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(150), "should wait at least the timeout");
    assert!(elapsed < Duration::from_secs(2), "should not block long after timeout");
}
