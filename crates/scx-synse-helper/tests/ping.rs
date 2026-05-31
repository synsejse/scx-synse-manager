use std::sync::Arc;

use anyhow::Result;
use scx_synse_helper::executor::Executor;
use scx_synse_helper::protocol::run;
use scx_synse_ipc::{Request, Response, SchedMode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct NoopExecutor;

#[async_trait::async_trait]
impl Executor for NoopExecutor {
    async fn apply(&self, _: &str, _: SchedMode) -> Result<()> { Ok(()) }
    async fn disable(&self) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn ping_returns_ok() {
    let req = Request::Ping;
    let mut input_bytes = serde_json::to_vec(&req).unwrap();
    input_bytes.push(b'\n');

    let (mut input_writer, input_reader) = tokio::io::duplex(4096);
    let (output_writer, mut output_reader) = tokio::io::duplex(4096);
    let exec = Arc::new(NoopExecutor);

    let join = tokio::spawn(async move {
        run(input_reader, output_writer, exec).await.unwrap();
    });

    input_writer.write_all(&input_bytes).await.unwrap();
    drop(input_writer); // signal EOF

    let mut buf = String::new();
    output_reader.read_to_string(&mut buf).await.unwrap();
    join.await.unwrap();

    let line = buf.lines().next().unwrap();
    let resp: Response = serde_json::from_str(line).unwrap();
    assert_eq!(resp, Response::Ok);
}
