use anyhow::{Error, anyhow};
use std::fmt::Write;
use std::thread;

/// Utility function for keeping track of errors that originate in worker threads.
///
/// # Panics
///
/// Panics in threads joined by this function will cause panics here.
#[must_use]
pub fn join_and_collect_thread_errors<T>(handles: Vec<thread::JoinHandle<T>>) -> Option<Error> {
    let mut error_str = String::new();
    for handle in handles {
        let thread_name = handle.thread().name().unwrap_or("unnamed").to_owned();
        if let Err(thread_panic) = handle.join() {
            if let Ok(thread_panic_anyhow) = thread_panic.downcast::<Error>() {
                let _ = write!(
                    error_str,
                    "Error returned from thread {thread_name}:\n{thread_panic_anyhow:?}\n----------\n",
                );
            } else {
                panic!(
                    "Failed downcast to anyhow::Error. This should not happen. Threads in this application should always return anyhow::Error."
                );
            }
        }
    }
    if error_str.is_empty() {
        return None;
    }
    Some(anyhow!("Error in one or more threads.").context(error_str))
}
