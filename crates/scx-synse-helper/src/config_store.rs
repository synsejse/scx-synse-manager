use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use scx_loader::config::Config;
use scx_loader::{SchedMode, SupportedSched};

/// Owns the on-disk scx_loader configuration and supports atomic writes.
pub struct ConfigStore {
    path: PathBuf,
    config: Config,
}

impl ConfigStore {
    /// Load `path`. If missing, returns the upstream default config.
    /// If present but malformed, returns Err.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = if path.exists() {
            scx_loader::config::parse_config_file(path.to_str().context("non-UTF-8 path")?)
                .with_context(|| format!("parsing {}", path.display()))?
        } else {
            scx_loader::config::get_default_config()
        };
        Ok(Self { path, config })
    }

    /// Like `open` but silently falls back to the default config if the file
    /// is malformed. Used by `save_is_atomic_via_tmp_rename` and by helper
    /// recovery paths so a corrupt file never wedges the GUI.
    pub fn open_or_default(path: impl AsRef<Path>) -> Self {
        Self::open(&path).unwrap_or_else(|_| Self {
            path: path.as_ref().to_path_buf(),
            config: scx_loader::config::get_default_config(),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_default_sched(&mut self, sched: Option<SupportedSched>) {
        self.config.default_sched = sched;
    }

    pub fn set_default_mode(&mut self, mode: Option<SchedMode>) {
        self.config.default_mode = mode;
    }

    /// Persist to disk via write-temp + rename.
    pub fn save(&self) -> Result<()> {
        let toml_str = toml::to_string(&self.config).context("serializing scx_loader config")?;
        let mut tmp = self.path.clone();
        let mut tmp_name = tmp.file_name().context("path has no file name")?.to_os_string();
        tmp_name.push(".tmp");
        tmp.set_file_name(tmp_name);

        {
            let mut f = fs::File::create(&tmp)
                .with_context(|| format!("creating {}", tmp.display()))?;
            f.write_all(toml_str.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &self.path).with_context(|| {
            format!("renaming {} -> {}", tmp.display(), self.path.display())
        })?;
        Ok(())
    }
}
