//! Test helper: run a future with its log output captured as JSON lines, so a
//! test can assert what was (and was not) logged. Compiled only for tests and
//! for crates that enable the `test-support` feature in `dev-dependencies`.
//!
//! One global subscriber is installed once and routes each line to the sink of
//! the future being polled on that thread. (Per-future scoped subscribers
//! race with `tracing`'s callsite interest cache when tests run in parallel.)

use std::{
    cell::RefCell,
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex, Once},
    task::{Context, Poll},
};

use tracing_subscriber::fmt;

type Sink = Arc<Mutex<Vec<u8>>>;

thread_local! {
    /// Where lines emitted on this thread go right now; `None` discards them.
    static CURRENT: RefCell<Option<Sink>> = const { RefCell::new(None) };
}

/// Writer that forwards to the thread's current sink.
struct Routed;

impl io::Write for Routed {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        CURRENT.with(|current| {
            if let Some(sink) = &*current.borrow() {
                sink.lock()
                    .expect("log sink poisoned")
                    .extend_from_slice(buf);
            }
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn install_once() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let subscriber = fmt()
            .json()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(|| Routed)
            .finish();
        // Another global subscriber would make captures come back empty, which
        // the asserting test reports; nothing to recover here.
        tracing::subscriber::set_global_default(subscriber).ok();
    });
}

/// Polls the inner future with this capture's sink current on the thread.
struct Capturing<F> {
    inner: Pin<Box<F>>,
    sink: Sink,
}

impl<F: Future> Future for Capturing<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        let sink = self.sink.clone();
        let previous = CURRENT.with(|c| c.replace(Some(sink)));
        let result = self.inner.as_mut().poll(cx);
        CURRENT.with(|c| *c.borrow_mut() = previous);
        result
    }
}

/// Runs `future`, returning its output and everything logged meanwhile (debug
/// and up, one JSON object per line, spans included under `spans`).
///
/// # Panics
/// If the log sink's lock was poisoned by a panic while logging.
pub async fn capture_logs<F: Future>(future: F) -> (F::Output, String) {
    install_once();
    let sink = Sink::default();
    let output = Capturing {
        inner: Box::pin(future),
        sink: sink.clone(),
    }
    .await;
    let bytes = sink.lock().expect("log sink poisoned").clone();
    (output, String::from_utf8_lossy(&bytes).into_owned())
}
