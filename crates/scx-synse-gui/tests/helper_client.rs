use scx_synse_gui::helper_client::HelperClient;
use scx_synse_ipc::{Request, Response, SchedMode};

/// The fake helper bin (replies Ok to every framed request).
fn fake_helper() -> String {
    env!("CARGO_BIN_EXE_fake_helper").to_string()
}

#[tokio::test]
async fn round_trip_ping() {
    let mut client = HelperClient::with_command(&fake_helper(), &[]);
    let resp = client.send(Request::Ping).await.unwrap();
    assert_eq!(resp, Response::Ok);
}

#[tokio::test]
async fn multiple_requests_reuse_same_child() {
    let mut client = HelperClient::with_command(&fake_helper(), &[]);
    client.send(Request::Ping).await.unwrap();
    let pid_first = client.child_pid().unwrap();
    client
        .send(Request::Apply { scheduler: "scx_bpfland".into(), mode: SchedMode::Auto })
        .await
        .unwrap();
    let pid_second = client.child_pid().unwrap();
    assert_eq!(pid_first, pid_second, "client must reuse the spawned child");
}
