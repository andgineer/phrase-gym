use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use super::StreamSummary;

#[derive(Default)]
struct Progress {
    bytes: AtomicU64,
    truncated: AtomicBool,
    closed: AtomicBool,
}

pub(super) struct Capture {
    progress: Arc<Progress>,
    task: Option<JoinHandle<()>>,
}

impl Capture {
    pub(super) fn start(
        stream: Option<impl AsyncRead + Unpin + Send + 'static>,
        file: tokio::fs::File,
        limit: u64,
        overflow: Arc<Notify>,
    ) -> Self {
        let progress = Arc::new(Progress::default());
        let task = stream.map(|stream| {
            tokio::spawn(drain(stream, file, limit, overflow, Arc::clone(&progress)))
        });
        Self { progress, task }
    }

    // A descendant that escaped the owned tree can keep the pipe open; stop waiting after `patience`.
    pub(super) async fn finish(mut self, patience: Duration) -> StreamSummary {
        if let Some(task) = self.task.take() {
            let abort = task.abort_handle();
            if tokio::time::timeout(patience, task).await.is_err() {
                abort.abort();
            }
        }
        StreamSummary {
            bytes: self.progress.bytes.load(Ordering::SeqCst),
            truncated: self.progress.truncated.load(Ordering::SeqCst),
            closed: self.progress.closed.load(Ordering::SeqCst),
        }
    }
}

async fn drain(
    mut stream: impl AsyncRead + Unpin,
    mut file: tokio::fs::File,
    limit: u64,
    overflow: Arc<Notify>,
    progress: Arc<Progress>,
) {
    let mut buffer = vec![0_u8; 16 * 1024];
    loop {
        let read = match stream.read(&mut buffer).await {
            Ok(0) => {
                progress.closed.store(true, Ordering::SeqCst);
                break;
            }
            Ok(read) => read,
            Err(_) => break,
        };
        let written = progress.bytes.load(Ordering::SeqCst);
        let room = usize::try_from(limit.saturating_sub(written)).unwrap_or(usize::MAX);
        let kept = read.min(room);
        if kept > 0 && file.write_all(&buffer[..kept]).await.is_err() {
            break;
        }
        progress
            .bytes
            .fetch_add(u64::try_from(kept).unwrap_or(u64::MAX), Ordering::SeqCst);
        if kept < read {
            progress.truncated.store(true, Ordering::SeqCst);
            overflow.notify_one();
            break;
        }
    }
    let _ = file.flush().await;
}
