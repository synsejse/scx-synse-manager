//! Shared request/response types for scx-synse-manager IPC.
//!
//! Wire format: newline-delimited JSON (NDJSON). One [`Request`] per line
//! from GUI to helper; one [`Response`] per line from helper to GUI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchedMode {
    Auto,
    Gaming,
    PowerSave,
    LowLatency,
    Server,
}

impl SchedMode {
    pub fn as_raw(self) -> u32 {
        match self {
            SchedMode::Auto => 0,
            SchedMode::Gaming => 1,
            SchedMode::PowerSave => 2,
            SchedMode::LowLatency => 3,
            SchedMode::Server => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Request {
    Ping,
    Apply { scheduler: String, mode: SchedMode },
    Disable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Response {
    Ok,
    Err { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_apply_round_trip() {
        let req = Request::Apply {
            scheduler: "scx_bpfland".into(),
            mode: SchedMode::Gaming,
        };
        let line = serde_json::to_string(&req).unwrap();
        assert!(!line.contains('\n'));
        let parsed: Request = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed, req);
    }

    #[test]
    fn request_simple_variants_round_trip() {
        for req in [Request::Ping, Request::Disable] {
            let line = serde_json::to_string(&req).unwrap();
            let parsed: Request = serde_json::from_str(&line).unwrap();
            assert_eq!(parsed, req);
        }
    }

    #[test]
    fn response_round_trip() {
        let cases = [
            Response::Ok,
            Response::Err { message: "boom".into() },
        ];
        for resp in cases {
            let line = serde_json::to_string(&resp).unwrap();
            let parsed: Response = serde_json::from_str(&line).unwrap();
            assert_eq!(parsed, resp);
        }
    }

    #[test]
    fn sched_mode_raw_matches_upstream_numbering() {
        assert_eq!(SchedMode::Auto.as_raw(), 0);
        assert_eq!(SchedMode::Gaming.as_raw(), 1);
        assert_eq!(SchedMode::PowerSave.as_raw(), 2);
        assert_eq!(SchedMode::LowLatency.as_raw(), 3);
        assert_eq!(SchedMode::Server.as_raw(), 4);
    }
}
