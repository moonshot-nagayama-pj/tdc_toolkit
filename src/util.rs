use anyhow::{Result, bail};
use std::thread;

/// Utility function for keeping track of errors that originate in worker threads.
///
/// # Panics
///
/// Panics in threads joined by this function will cause panics here.
pub fn join_and_collect_thread_errors(handles: Vec<thread::JoinHandle<Result<()>>>) -> Result<()> {
    let mut error_str = String::new();
    let mut panic_str = String::new();

    for handle in handles {
        let thread_name = handle.thread().name().unwrap_or("unnamed").to_owned();
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                error_str = format!("{error_str}Error in thread {thread_name}:\n{e:?}\n---\n");
            }
            Err(payload) => match payload.downcast_ref::<String>() {
                Some(payload_string) => {
                    panic_str = format!(
                        "{panic_str}Panic in thread {thread_name}:\n{payload_string}\n---\n"
                    );
                }
                None => match payload.downcast_ref::<&str>() {
                    Some(payload_string) => {
                        panic_str = format!(
                            "{panic_str}Panic in thread {thread_name}:\n{payload_string}\n---\n"
                        );
                    }
                    None => {
                        panic_str = format!(
                            "{panic_str}Panic in thread {thread_name}:\nPanic payload was not downcastable to String or &str\n---\n"
                        );
                    }
                },
            },
        }

        if !panic_str.is_empty() {
            if !error_str.is_empty() {
                panic_str =
                    format!("{panic_str}Additionally, threads returned errors:\n---\n{error_str}");
            }
            std::panic::resume_unwind(Box::new(panic_str));
        }
    }
    if error_str.is_empty() {
        return Ok(());
    }
    bail!("Errors in one or more threads: {}", error_str)
}
