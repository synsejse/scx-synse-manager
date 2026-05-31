//! Privileged helper for scx-synse-manager.

use anyhow::Result;
use scx_synse_helper::protocol::run;
use scx_synse_helper::real_executor::RealExecutor;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let executor = RealExecutor::new().await?;
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    run(stdin, stdout, executor).await
}
