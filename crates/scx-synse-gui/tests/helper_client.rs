use std::path::PathBuf;

use scx_synse_gui::helper_client::HelperClient;
use scx_synse_ipc::{Request, Response, SchedMode};

fn fake_helper_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fake_helper.sh")
}

#[tokio::test]
async fn round_trip_ping() {
    let mut client = HelperClient::with_command(
        "/usr/bin/env",
        &["bash".to_string(), fake_helper_path().display().to_string()],
    );
    let resp = client.send(Request::Ping).await.unwrap();
    assert_eq!(resp, Response::Ok);
}

#[tokio::test]
async fn multiple_requests_reuse_same_child() {
    let mut client = HelperClient::with_command(
        "/usr/bin/env",
        &["bash".to_string(), fake_helper_path().display().to_string()],
    );
    let _ = client.send(Request::Ping).await.unwrap();
    let pid_first = client.child_pid().unwrap();
    let _ = client
        .send(Request::Apply {
            scheduler: "scx_bpfland".into(),
            mode: SchedMode::Auto,
        })
        .await
        .unwrap();
    let pid_second = client.child_pid().unwrap();
    assert_eq!(pid_first, pid_second, "client must reuse the spawned child");
}
