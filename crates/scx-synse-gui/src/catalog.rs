/// A short use-case tag shown as a colored pill next to each scheduler, so a
/// newcomer can tell at a glance what a scheduler is *for* without parsing the
/// description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Badge {
    /// General-purpose, good for typical desktop use.
    Everyday,
    /// Optimized for games — responsiveness and frame pacing.
    Gaming,
    /// Tight, predictable timing for audio and real-time work.
    LowLatency,
    /// Throughput and isolation for servers / virtualization.
    Server,
    /// Research / unstable — interesting, but not a daily driver.
    Experimental,
    /// Minimal reference design, useful as a baseline.
    Baseline,
    /// Unknown scheduler with no catalog entry yet.
    Other,
}

impl Badge {
    /// Every badge's CSS class, for clearing stale styling when a widget is
    /// reused (e.g. the hero card re-pointed at a different scheduler).
    pub const ALL: [Badge; 7] = [
        Badge::Everyday,
        Badge::Gaming,
        Badge::LowLatency,
        Badge::Server,
        Badge::Experimental,
        Badge::Baseline,
        Badge::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Badge::Everyday => "Everyday",
            Badge::Gaming => "Gaming",
            Badge::LowLatency => "Low latency",
            Badge::Server => "Server",
            Badge::Experimental => "Experimental",
            Badge::Baseline => "Baseline",
            Badge::Other => "Other",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Badge::Everyday => "synse-badge-everyday",
            Badge::Gaming => "synse-badge-gaming",
            Badge::LowLatency => "synse-badge-lowlatency",
            Badge::Server => "synse-badge-server",
            Badge::Experimental => "synse-badge-experimental",
            Badge::Baseline => "synse-badge-baseline",
            Badge::Other => "synse-badge-other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerInfo {
    pub display_title: String,
    pub blurb: String,
    pub icon_name: &'static str,
    pub badge: Badge,
    pub supports_profiles: bool,
}

/// Look up a descriptor for a scheduler name. Unknown schedulers fall back
/// to a descriptor that uses their own name as the title so the UI stays
/// useful when scx_loader reports something new.
///
/// Descriptions are paraphrased from each scheduler's own crates.io / upstream
/// summary; see <https://github.com/sched-ext/scx>.
pub fn describe(scheduler: &str) -> SchedulerInfo {
    match scheduler {
        // ── Schedulers shipped by scx_loader on CachyOS ──────────────────
        "scx_bpfland" => SchedulerInfo {
            display_title: "scx_bpfland".into(),
            blurb: "All-rounder that keeps interactive apps snappy with low input lag. Great for everyday desktop use and gaming.".into(),
            icon_name: "gamepad-symbolic",
            badge: Badge::Everyday,
            supports_profiles: true,
        },
        "scx_lavd" => SchedulerInfo {
            display_title: "scx_lavd".into(),
            blurb: "Latency-aware deadline scheduling. A favorite for gaming, esports, and real-time audio.".into(),
            icon_name: "headphones-symbolic",
            badge: Badge::Gaming,
            supports_profiles: true,
        },
        "scx_rusty" => SchedulerInfo {
            display_title: "scx_rusty".into(),
            blurb: "Multi-domain scheduler that scales smoothly across large multi-core and NUMA systems.".into(),
            icon_name: "sitemap-symbolic",
            badge: Badge::Everyday,
            supports_profiles: false,
        },
        "scx_p2dq" => SchedulerInfo {
            display_title: "scx_p2dq".into(),
            blurb: "Versatile \u{201c}pick-two\u{201d} load balancer that adapts well to mixed workloads.".into(),
            icon_name: "scale-balanced-symbolic",
            badge: Badge::Everyday,
            supports_profiles: true,
        },
        "scx_tickless" => SchedulerInfo {
            display_title: "scx_tickless".into(),
            blurb: "Server and virtualization scheduler that cuts timer-tick noise for steadier, isolated performance.".into(),
            icon_name: "server-symbolic",
            badge: Badge::Server,
            supports_profiles: true,
        },
        "scx_cosmos" => SchedulerInfo {
            display_title: "scx_cosmos".into(),
            blurb: "Lightweight scheduler that keeps tasks on warm CPUs to minimize overhead.".into(),
            icon_name: "moon-symbolic",
            badge: Badge::Everyday,
            supports_profiles: true,
        },
        "scx_cake" => SchedulerInfo {
            display_title: "scx_cake".into(),
            blurb: "Borrows CAKE anti-bufferbloat ideas to keep the system responsive under heavy load.".into(),
            icon_name: "cake-candles-symbolic",
            badge: Badge::LowLatency,
            supports_profiles: true,
        },
        "scx_flash" => SchedulerInfo {
            display_title: "scx_flash".into(),
            blurb: "Tuned for multimedia and real-time audio with consistently low latency.".into(),
            icon_name: "bolt-symbolic",
            badge: Badge::LowLatency,
            supports_profiles: true,
        },
        "scx_flow" => SchedulerInfo {
            display_title: "scx_flow".into(),
            blurb: "Multi-lane, budget-based scheduler balancing snappy response with general throughput.".into(),
            icon_name: "wind-symbolic",
            badge: Badge::Everyday,
            supports_profiles: false,
        },
        "scx_beerland" => SchedulerInfo {
            display_title: "scx_beerland".into(),
            blurb: "Prioritizes cache locality and scalability \u{2014} handy on many-core machines and surprisingly good for gaming.".into(),
            icon_name: "beer-mug-empty-symbolic",
            badge: Badge::Everyday,
            supports_profiles: false,
        },
        "scx_pandemonium" => SchedulerInfo {
            display_title: "scx_pandemonium".into(),
            blurb: "Adaptive scheduler that learns and classifies task behavior on the fly. Newer and still maturing.".into(),
            icon_name: "brain-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_rustland" => SchedulerInfo {
            display_title: "scx_rustland".into(),
            blurb: "Runs its scheduling policy in user-space Rust \u{2014} mainly a research and prototyping playground.".into(),
            icon_name: "flask-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },

        // ── Other upstream scx schedulers (may appear on other systems) ──
        "scx_layered" => SchedulerInfo {
            display_title: "scx_layered".into(),
            blurb: "Configurable layered scheduling tuned for specific server and workstation workloads.".into(),
            icon_name: "layer-group-symbolic",
            badge: Badge::Server,
            supports_profiles: false,
        },
        "scx_chaos" => SchedulerInfo {
            display_title: "scx_chaos".into(),
            blurb: "Injects randomized scheduling delays to stress-test software and surface timing bugs.".into(),
            icon_name: "dice-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_mitosis" => SchedulerInfo {
            display_title: "scx_mitosis".into(),
            blurb: "Splits the machine into dynamic cells for isolated scheduling domains.".into(),
            icon_name: "clone-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_simple" => SchedulerInfo {
            display_title: "scx_simple".into(),
            blurb: "Minimal global vtime scheduler. A clean baseline rather than a daily driver.".into(),
            icon_name: "cube-symbolic",
            badge: Badge::Baseline,
            supports_profiles: false,
        },
        "scx_central" => SchedulerInfo {
            display_title: "scx_central".into(),
            blurb: "Routes all scheduling decisions through a single CPU \u{2014} a specialized baseline design.".into(),
            icon_name: "bullseye-symbolic",
            badge: Badge::Baseline,
            supports_profiles: false,
        },
        "scx_userland" => SchedulerInfo {
            display_title: "scx_userland".into(),
            blurb: "Example scheduler that runs entirely in user space. For experimentation, not daily use.".into(),
            icon_name: "terminal-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_nest" => SchedulerInfo {
            display_title: "scx_nest".into(),
            blurb: "Research scheduler that re-uses recently active (warm) cores to keep frequencies high.".into(),
            icon_name: "fire-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_pair" => SchedulerInfo {
            display_title: "scx_pair".into(),
            blurb: "Research scheduler that co-schedules tasks onto SMT sibling threads.".into(),
            icon_name: "link-symbolic",
            badge: Badge::Experimental,
            supports_profiles: false,
        },
        "scx_prev" => SchedulerInfo {
            display_title: "scx_prev".into(),
            blurb: "Simple scheduler that favors a task's previous CPU to keep caches warm.".into(),
            icon_name: "arrow-rotate-left-symbolic",
            badge: Badge::Baseline,
            supports_profiles: false,
        },

        unknown => SchedulerInfo {
            display_title: unknown.to_owned(),
            blurb: "Sched-ext scheduler. No description in the catalog yet.".into(),
            icon_name: "microchip-symbolic",
            badge: Badge::Other,
            supports_profiles: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_scheduler_has_non_empty_fields() {
        for name in [
            "scx_bpfland",
            "scx_lavd",
            "scx_rusty",
            "scx_p2dq",
            "scx_tickless",
            "scx_cosmos",
            "scx_cake",
            "scx_flash",
            "scx_flow",
            "scx_beerland",
            "scx_pandemonium",
            "scx_rustland",
            "scx_layered",
            "scx_chaos",
            "scx_mitosis",
            "scx_simple",
            "scx_central",
            "scx_userland",
            "scx_nest",
            "scx_pair",
            "scx_prev",
        ] {
            let info = describe(name);
            assert_eq!(info.display_title, name, "title for {name}");
            assert!(!info.blurb.is_empty(), "blurb for {name}");
            assert!(!info.icon_name.is_empty(), "icon for {name}");
            assert_ne!(info.badge, Badge::Other, "{name} should have a real badge");
        }
    }

    #[test]
    fn unknown_scheduler_uses_its_own_name_as_title() {
        let info = describe("scx_nonsense");
        assert_eq!(info.display_title, "scx_nonsense");
        assert_eq!(info.badge, Badge::Other);
        assert!(!info.supports_profiles);
        assert!(!info.blurb.is_empty());
    }

    #[test]
    fn badge_css_classes_are_unique() {
        let mut classes: Vec<&str> = Badge::ALL.iter().map(|b| b.css_class()).collect();
        let count = classes.len();
        classes.sort_unstable();
        classes.dedup();
        assert_eq!(classes.len(), count, "every badge needs a distinct css class");
    }
}
