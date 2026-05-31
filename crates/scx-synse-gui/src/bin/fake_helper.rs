//! Test fixture: a fake privileged helper that replies `Response::Ok` to every
//! framed request. The `helper_client` integration tests spawn it (via
//! `CARGO_BIN_EXE_fake_helper`) to exercise process spawning and child reuse
//! without needing root or D-Bus. Not installed by the packaging.

use scx_synse_ipc::{read_frame, write_frame, Response};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let ok = rkyv::to_bytes::<rkyv::rancor::Error>(&Response::Ok).expect("encode ok");
    while let Ok(Some(_)) = read_frame(&mut stdin).await {
        if write_frame(&mut stdout, &ok).await.is_err() {
            break;
        }
    }
}
