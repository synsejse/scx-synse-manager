use anyhow::Result;
use scx_synse_ipc::SchedMode;

/// Performs the actual privileged side-effects.
///
/// Abstracted as a trait so unit/integration tests can drive the protocol
/// without a live D-Bus bus or root filesystem.
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    async fn apply(&self, scheduler: &str, mode: SchedMode) -> Result<()>;
    async fn disable(&self) -> Result<()>;
}
