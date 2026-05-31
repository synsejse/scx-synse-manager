use std::process::Stdio;

use scx_synse_ipc::{Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// Long-lived privileged helper client. Spawns once on first `send()`, then
/// reuses the same child for every subsequent request — that's how a single
/// pkexec prompt covers a whole session.
pub struct HelperClient {
    program: String,
    args: Vec<String>,
    child: Option<HelperChild>,
}

struct HelperChild {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, thiserror::Error)]
pub enum HelperError {
    #[error("authorization canceled")]
    AuthCanceled,
    #[error("helper crashed: {0}")]
    Crashed(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol: {0}")]
    Protocol(String),
}

impl HelperClient {
    /// Default constructor for production use: runs the privileged helper via
    /// `pkexec`. The helper lives in libexecdir (e.g. `/usr/libexec`), which is
    /// not on `$PATH`, so it must be invoked by absolute path — otherwise
    /// pkexec can't find it (exit 127, surfaced as "Authorization cancelled")
    /// and wouldn't match the polkit action's annotated `exec.path`. The path
    /// is baked in at build time from meson's libexecdir, defaulting to the
    /// standard location for plain `cargo` builds.
    pub fn pkexec() -> Self {
        let helper = option_env!("SCX_SYNSE_HELPER_PATH")
            .unwrap_or("/usr/libexec/scx-synse-helper");
        Self::with_command("pkexec", &[helper.to_string()])
    }

    pub fn with_command(program: &str, args: &[String]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.to_vec(),
            child: None,
        }
    }

    /// PID of the live helper, or None if it hasn't been spawned yet.
    pub fn child_pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(|c| c.process.id())
    }

    pub async fn send(&mut self, req: Request) -> Result<Response, HelperError> {
        let child = self.ensure_started().await?;
        let mut encoded = serde_json::to_string(&req)
            .map_err(|e| HelperError::Protocol(e.to_string()))?;
        encoded.push('\n');
        child.stdin.write_all(encoded.as_bytes()).await?;
        child.stdin.flush().await?;

        let mut response_line = String::new();
        let n = child.stdout.read_line(&mut response_line).await?;
        if n == 0 {
            // EOF before any response — helper crashed or auth was canceled.
            let status = child.process.try_wait().ok().flatten();
            let stderr = drain_stderr(&mut child.process).await;
            self.child = None;
            return Err(match status {
                Some(s) if s.code() == Some(126) || s.code() == Some(127) => {
                    HelperError::AuthCanceled
                }
                _ => HelperError::Crashed(stderr),
            });
        }
        let trimmed = response_line.trim();
        serde_json::from_str::<Response>(trimmed)
            .map_err(|e| HelperError::Protocol(format!("decode {trimmed:?}: {e}")))
    }

    async fn ensure_started(&mut self) -> Result<&mut HelperChild, HelperError> {
        if self.child.is_none() {
            let mut cmd = Command::new(&self.program);
            cmd.args(&self.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let mut process = cmd.spawn().map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => HelperError::Crashed(format!(
                    "{} not found in PATH",
                    self.program
                )),
                _ => HelperError::Io(e),
            })?;
            let stdin = process.stdin.take().ok_or_else(|| {
                HelperError::Protocol("child has no stdin".into())
            })?;
            let stdout = process.stdout.take().ok_or_else(|| {
                HelperError::Protocol("child has no stdout".into())
            })?;
            self.child = Some(HelperChild {
                process,
                stdin,
                stdout: BufReader::new(stdout),
            });
        }
        Ok(self.child.as_mut().unwrap())
    }
}

async fn drain_stderr(child: &mut Child) -> String {
    let Some(mut stderr) = child.stderr.take() else {
        return String::new();
    };
    let mut buf = String::new();
    use tokio::io::AsyncReadExt;
    let _ = stderr.read_to_string(&mut buf).await;
    buf
}

// The child is spawned with `kill_on_drop(true)`, so dropping `HelperClient`
// tears the helper down — no explicit Drop impl needed.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkexec_invokes_helper_by_absolute_path() {
        // The helper installs to libexecdir (e.g. /usr/libexec), which is not
        // on $PATH. pkexec can only run it — and only matches our polkit
        // action's annotated exec.path — when given an absolute path. A bare
        // program name makes pkexec exit 127, surfacing as "Authorization
        // cancelled" with no prompt.
        let client = HelperClient::pkexec();
        assert_eq!(client.program, "pkexec");
        assert_eq!(client.args.len(), 1, "expected a single helper-path argument");
        assert!(
            client.args[0].starts_with('/'),
            "helper must be invoked by absolute path; got {:?}",
            client.args[0],
        );
    }
}
