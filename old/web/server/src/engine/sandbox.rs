use std::{
    ffi::OsStr,
    io,
    path::Path,
    pin::Pin,
    process::Stdio,
    task::{Context, Poll},
    time::Duration,
};

use futures::stream::StreamExt;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{Child, ChildStdin, ChildStdout, Command},
    time,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::warn;

use crate::error::{Error, ErrorContext};
use pin_project::pin_project;

#[pin_project]
pub struct Sandbox {
    #[pin]
    pub stdin: ChildStdin,
    #[pin]
    pub stdout: ChildStdout,
    child: Child,
}

impl AsyncWrite for Sandbox {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();
        this.stdin.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        this.stdin.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        this.stdin.poll_shutdown(cx)
    }
}

impl AsyncRead for Sandbox {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        this.stdout.poll_read(cx, buf)
    }
}

impl Sandbox {
    pub async fn from_executable_path(path: &Path) -> Result<Self, Error> {
        let path = tokio::fs::canonicalize(path)
            .await
            .with_context(|| format!("could not canonicalize engine path {}", path.display()))?;

        let mut child = create_docker_cmd(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn sandbox command")?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stderr = FramedRead::new(stderr, LinesCodec::new());

        tokio::spawn(stderr.for_each(|x| {
            if let Ok(e) = x {
                warn!("engine stderr: {}", e);
            }
            futures::future::ready(())
        }));

        Ok(Sandbox {
            stdin,
            stdout,
            child,
        })
    }

    pub async fn stop(mut self) -> Result<(), Error> {
        match time::timeout(Duration::from_secs(1000), self.child.wait()).await {
            Err(_) => {
                warn!("sandbox process did not quit before timeout, killing process");

                self.child.kill().await?;
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e.into()),
        }
        Ok(())
    }
}

fn create_docker_cmd(executable_path: &Path) -> Command {
    let mut volume_arg = executable_path.as_os_str().to_owned();
    volume_arg.push(":/exec/engine:ro");
    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .args(["--network", "none"])
        .args(["-a", "stdin"])
        .args(["-a", "stdout"])
        .args(["-a", "stderr"])
        .arg("-i")
        .arg("--rm")
        .args(["--memory", "512M"])
        .args(["--cpus", "2"])
        .args(["--pids-limit", "512"])
        .arg("--security-opt=no-new-privileges")
        .arg("--cap-drop=ALL")
        .arg("--cap-add=DAC_OVERRIDE")
        .args([OsStr::new("-v"), &volume_arg])
        .args(["-w", "/exec"])
        .arg("ubuntu:latest")
        .arg("./engine");

    cmd.kill_on_drop(true);

    cmd
}
