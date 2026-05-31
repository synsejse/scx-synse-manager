use std::path::PathBuf;

/// Where to look for sched_ext kernel state. Injectable for tests.
#[derive(Debug, Clone)]
pub struct SysfsRoot(pub PathBuf);

impl Default for SysfsRoot {
    fn default() -> Self {
        Self(PathBuf::from("/sys/kernel/sched_ext"))
    }
}

/// Whether the running kernel exposes sched_ext at all. When false, the app
/// shows its "unsupported kernel" guard screen. Which scheduler is loaded (and
/// its mode) comes from scx_loader, not sysfs.
pub fn is_supported(root: &SysfsRoot) -> bool {
    root.0.join("state").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sysfs_is_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_supported(&SysfsRoot(dir.path().to_owned())));
    }

    #[test]
    fn present_state_file_is_supported() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("state"), "disabled\n").unwrap();
        assert!(is_supported(&SysfsRoot(dir.path().to_owned())));
    }
}
