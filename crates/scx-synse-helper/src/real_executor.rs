use std::sync::Arc;

use anyhow::{Context, Result};
use scx_loader::dbus::LoaderClientProxy;
use scx_loader::{SchedMode as UpstreamMode, SupportedSched};
use tokio::sync::Mutex;
use zbus::Connection;

use crate::config_store::ConfigStore;
use crate::executor::Executor;
use scx_synse_ipc::SchedMode;

const DEFAULT_CONFIG_PATH: &str = "/etc/scx_loader.toml";

pub struct RealExecutor {
    state: Mutex<ConfigStore>,
    conn: Connection,
}

impl RealExecutor {
    pub async fn new() -> Result<Arc<Self>> {
        Self::with_path(DEFAULT_CONFIG_PATH).await
    }

    pub async fn with_path(path: impl Into<std::path::PathBuf>) -> Result<Arc<Self>> {
        let store = ConfigStore::open_or_default(path.into());
        let conn = Connection::system()
            .await
            .context("connecting to system D-Bus")?;
        Ok(Arc::new(Self { state: Mutex::new(store), conn }))
    }

    async fn loader(&self) -> Result<LoaderClientProxy<'_>> {
        LoaderClientProxy::new(&self.conn)
            .await
            .context("creating scx_loader proxy")
    }
}

#[async_trait::async_trait]
impl Executor for RealExecutor {
    async fn apply(&self, scheduler: &str, mode: SchedMode) -> Result<()> {
        let sched: SupportedSched = scheduler
            .parse()
            .with_context(|| format!("unknown scheduler {scheduler:?}"))?;
        let mode = to_upstream_mode(mode);

        self.loader()
            .await?
            .switch_scheduler(sched.clone(), mode)
            .await
            .context("switch_scheduler")?;

        // Persist the choice so scx_loader restores it on the next boot.
        let mut store = self.state.lock().await;
        store.set_default_sched(Some(sched));
        store.set_default_mode(Some(mode));
        store.save().context("saving config")?;
        Ok(())
    }

    async fn disable(&self) -> Result<()> {
        self.loader()
            .await?
            .stop_scheduler()
            .await
            .context("stop_scheduler")?;
        Ok(())
    }
}

fn to_upstream_mode(mode: SchedMode) -> UpstreamMode {
    match mode {
        SchedMode::Auto => UpstreamMode::Auto,
        SchedMode::Gaming => UpstreamMode::Gaming,
        SchedMode::PowerSave => UpstreamMode::PowerSave,
        SchedMode::LowLatency => UpstreamMode::LowLatency,
        SchedMode::Server => UpstreamMode::Server,
    }
}
