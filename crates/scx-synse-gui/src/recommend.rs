use scx_synse_ipc::SchedMode;

#[derive(Debug, Clone, Default)]
pub struct SystemProbe {
    pub has_battery: bool,
    pub has_dgpu: bool,
    pub cpu_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recommendation {
    pub scheduler: &'static str,
    pub mode: SchedMode,
    /// One-line rationale shown next to the "Recommended" badge.
    pub reason: &'static str,
}

pub fn recommend(probe: &SystemProbe) -> Recommendation {
    if probe.has_battery {
        return Recommendation {
            scheduler: "scx_bpfland",
            mode: SchedMode::Auto,
            reason: "Balanced choice on battery-powered hardware.",
        };
    }
    if probe.has_dgpu {
        return Recommendation {
            scheduler: "scx_bpfland",
            mode: SchedMode::Gaming,
            reason: "Discrete GPU detected — favors input responsiveness.",
        };
    }
    if probe.cpu_count >= 16 {
        return Recommendation {
            scheduler: "scx_layered",
            mode: SchedMode::Server,
            reason: "Many-core system — layered scheduling fits well.",
        };
    }
    Recommendation {
        scheduler: "scx_bpfland",
        mode: SchedMode::Auto,
        reason: "Safe default for desktops.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_wins_over_other_signals() {
        let probe = SystemProbe { has_battery: true, has_dgpu: true, cpu_count: 32 };
        let rec = recommend(&probe);
        assert_eq!(rec.scheduler, "scx_bpfland");
        assert_eq!(rec.mode, SchedMode::Auto);
    }

    #[test]
    fn dgpu_without_battery_picks_gaming() {
        let probe = SystemProbe { has_battery: false, has_dgpu: true, cpu_count: 8 };
        let rec = recommend(&probe);
        assert_eq!(rec.scheduler, "scx_bpfland");
        assert_eq!(rec.mode, SchedMode::Gaming);
    }

    #[test]
    fn many_cores_picks_layered() {
        let probe = SystemProbe { has_battery: false, has_dgpu: false, cpu_count: 32 };
        let rec = recommend(&probe);
        assert_eq!(rec.scheduler, "scx_layered");
        assert_eq!(rec.mode, SchedMode::Server);
    }

    #[test]
    fn default_falls_back_to_bpfland_auto() {
        let probe = SystemProbe::default();
        let rec = recommend(&probe);
        assert_eq!(rec.scheduler, "scx_bpfland");
        assert_eq!(rec.mode, SchedMode::Auto);
    }
}

impl SystemProbe {
    /// Detect hardware signals from the running system. Best-effort; any
    /// I/O failure just leaves the field at its default.
    pub fn detect() -> Self {
        Self {
            has_battery: detect_battery(),
            has_dgpu: detect_dgpu(),
            cpu_count: detect_cpu_count(),
        }
    }
}

fn detect_battery() -> bool {
    std::fs::read_dir("/sys/class/power_supply")
        .map(|entries| {
            entries.flatten().any(|e| {
                e.file_name()
                    .to_string_lossy()
                    .to_ascii_uppercase()
                    .starts_with("BAT")
            })
        })
        .unwrap_or(false)
}

fn detect_dgpu() -> bool {
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else { return false; };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let driver_link = entry.path().join("device/driver");
        if let Ok(target) = std::fs::read_link(&driver_link) {
            let driver = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            // amdgpu, nouveau, nvidia, i915 → GPU drivers. simple-framebuffer is not.
            if driver != "simple-framebuffer" && !driver.is_empty() {
                return true;
            }
        }
    }
    false
}

fn detect_cpu_count() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
}
