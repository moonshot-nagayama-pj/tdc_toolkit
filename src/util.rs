use anyhow::{Result, bail};
use std::fmt::Write;
use std::thread;

/// Utility function for keeping track of errors that originate in worker threads.
///
/// # Panics
///
/// Panics in threads joined by this function will cause panics here.
pub fn join_and_collect_thread_errors(handles: Vec<thread::JoinHandle<Result<()>>>) -> Result<()> {
    let mut error_str = String::new();
    let mut unexpected_panic: Option<Box<dyn std::any::Any + Send>> = None;

    for handle in handles {
        let thread_name = handle.thread().name().unwrap_or("unnamed").to_owned();
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let _ = write!(error_str, "Error in thread {thread_name}:\n{e:?}\n---\n");
            }
            Err(payload) => {
                // Store first unexpected panic; keep joining remaining threads.
                if unexpected_panic.is_none() {
                    unexpected_panic = Some(payload);
                }
            }
        }

        if let Some(payload) = unexpected_panic {
            std::panic::resume_unwind(payload);
        }
    }
    if error_str.is_empty() {
        return Ok(());
    }
    bail!("Errors in one or more threads: {error_str}")
}
