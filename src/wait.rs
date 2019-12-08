use libc;
use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};

/// Wait for until a condition `cond` is `true` or the `timeout` is reached.
/// If the `timeout` is `None` it will wait an infinite time.
/// The condition is checked when the `file` has changed.
///
/// # Arguments
/// * `file` - Listen to changes in this file
/// * `cond` - Condition that should become true
/// * `timeout` - Maximal timeout to wait for the condition or file changes
///
/// # Example
/// ```
/// let fd = File::open(...).unwrap().as_raw_fd();
/// let cond = || {
///     ...
///     true
/// };
/// let timeout = Duration::from_millis(2000);
///
/// wait::wait(fd, cond, timeout);
/// ```
pub fn wait<F>(fd: RawFd, cond: F, timeout: Option<Duration>) -> bool
where
    F: Fn() -> bool,
{
    if cond() {
        return true;
    }

    let start = Instant::now();

    let mut t = timeout;

    loop {
        let wait_timeout = match t {
            Some(duration) => duration.as_millis() as i32,
            None => -1,
        };
        wait_file_changes(fd, wait_timeout);

        if let Some(duration) = timeout {
            let elapsed = start.elapsed();
            if elapsed >= duration {
                return false;
            }
            t = Some(duration - elapsed);
        }

        if cond() {
            return true;
        }
    }
}

/// Wrapper for `libc::epoll_wait`
fn wait_file_changes(fd: RawFd, timeout: i32) -> bool {
    let mut buf: [libc::epoll_event; 10] = [libc::epoll_event { events: 0, u64: 0 }; 10];

    let result = unsafe {
        libc::epoll_wait(
            fd,
            buf.as_mut_ptr() as *mut libc::epoll_event,
            buf.len() as i32,
            timeout,
        ) as i32
    };

    result > 0
}
