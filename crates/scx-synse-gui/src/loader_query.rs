use anyhow::{Context, Result};
use scx_loader::dbus::LoaderClientProxy;
use scx_synse_ipc::SchedMode;
use zbus::Connection;

/// Fetch the list of schedulers scx_loader knows how to launch. Returns an
/// empty Vec if scx_loader is unreachable — the UI is responsible for
/// surfacing that as a friendly error.
pub async fn supported_schedulers() -> Result<Vec<String>> {
    let conn = Connection::system().await.context("system bus")?;
    let proxy = LoaderClientProxy::new(&conn).await.context("proxy")?;
    Ok(proxy.supported_schedulers().await?)
}

/// The running scheduler together with its active mode, or None when nothing
/// is loaded. scx_loader reports "unknown" (or empty) for an idle system.
pub async fn current_state() -> Result<Option<(String, SchedMode)>> {
    let conn = Connection::system().await?;
    let proxy = LoaderClientProxy::new(&conn).await?;
    let name = proxy.current_scheduler().await?;
    match name.as_str() {
        "" | "unknown" => Ok(None),
        _ => {
            let mode = mode_from_upstream(proxy.scheduler_mode().await?);
            Ok(Some((name, mode)))
        }
    }
}

fn mode_from_upstream(mode: scx_loader::SchedMode) -> SchedMode {
    match mode {
        scx_loader::SchedMode::Auto => SchedMode::Auto,
        scx_loader::SchedMode::Gaming => SchedMode::Gaming,
        scx_loader::SchedMode::PowerSave => SchedMode::PowerSave,
        scx_loader::SchedMode::LowLatency => SchedMode::LowLatency,
        scx_loader::SchedMode::Server => SchedMode::Server,
    }
}
