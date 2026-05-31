use std::sync::Arc;

use anyhow::Result;
use rkyv::rancor::Error as RkyvError;
use scx_synse_helper::executor::Executor;
use scx_synse_helper::protocol::run;
use scx_synse_ipc::{read_frame, write_frame, Request, Response, SchedMode};

struct NoopExecutor;

#[async_trait::async_trait]
impl Executor for NoopExecutor {
    async fn apply(&self, _: &str, _: SchedMode) -> Result<()> { Ok(()) }
    async fn disable(&self) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn ping_returns_ok() {
    let payload = rkyv::to_bytes::<RkyvError>(&Request::Ping).unwrap();

    let (mut input_writer, input_reader) = tokio::io::duplex(4096);
    let (output_writer, mut output_reader) = tokio::io::duplex(4096);
    let exec = Arc::new(NoopExecutor);

    let join = tokio::spawn(async move {
        run(input_reader, output_writer, exec).await.unwrap();
    });

    write_frame(&mut input_writer, &payload).await.unwrap();
    drop(input_writer); // signal EOF

    let frame = read_frame(&mut output_reader).await.unwrap().expect("a response frame");
    join.await.unwrap();

    let resp = rkyv::from_bytes::<Response, RkyvError>(&frame).unwrap();
    assert_eq!(resp, Response::Ok);
}
