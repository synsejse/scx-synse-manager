//! Shared request/response types for scx-synse-manager IPC, plus the binary
//! wire framing.
//!
//! Messages are rkyv-encoded and sent as length-prefixed frames: a
//! little-endian `u32` byte length followed by that many bytes. Encode with
//! [`rkyv::to_bytes`] and decode with [`rkyv::from_bytes`]; move the bytes with
//! [`write_frame`] / [`read_frame`].

use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Apply { scheduler: String, mode: SchedMode },
    Disable,
}

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Err { message: String },
}

/// Write `payload` as one length-prefixed frame and flush.
pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> std::io::Result<()> {
    w.write_u32_le(payload.len() as u32).await?;
    w.write_all(payload).await?;
    w.flush().await
}

/// Read one length-prefixed frame into an aligned buffer (so rkyv can read it
/// directly). Returns `None` on a clean EOF before a frame begins.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Option<AlignedVec>> {
    let len = match r.read_u32_le().await {
        Ok(len) => len as usize,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut raw = vec![0u8; len];
    r.read_exact(&mut raw).await?;
    let mut buf = AlignedVec::<16>::new();
    buf.extend_from_slice(&raw);
    Ok(Some(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rkyv::rancor::Error;

    #[test]
    fn requests_round_trip() {
        for req in [
            Request::Ping,
            Request::Disable,
            Request::Apply { scheduler: "scx_bpfland".into(), mode: SchedMode::Gaming },
        ] {
            let bytes = rkyv::to_bytes::<Error>(&req).unwrap();
            assert_eq!(rkyv::from_bytes::<Request, Error>(&bytes).unwrap(), req);
        }
    }

    #[test]
    fn responses_round_trip() {
        for resp in [Response::Ok, Response::Err { message: "boom".into() }] {
            let bytes = rkyv::to_bytes::<Error>(&resp).unwrap();
            assert_eq!(rkyv::from_bytes::<Response, Error>(&bytes).unwrap(), resp);
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
