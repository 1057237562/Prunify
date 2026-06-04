use std::sync::atomic::{AtomicU32, Ordering};

static CHILD_PID: AtomicU32 = AtomicU32::new(0);

/// Register signal handlers for SIGINT and SIGTERM.
/// On Unix, uses libc::kill/libc::signal to forward signals to the child.
/// On Windows, Ctrl+C is handled by the console runtime; no-op.
#[cfg(unix)]
pub fn register_handler() {
    extern "C" fn forward_sigterm(_sig: i32) {
        let pid = CHILD_PID.load(Ordering::SeqCst);
        if pid != 0 {
            // SAFETY: pid is a valid child PID we spawned, and kill with SIGTERM
            // is safe to call even if the child has already exited (returns -1 with
            // ESRCH, which we ignore).
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }

    let _ = ctrlc::set_handler(move || {
        let pid = CHILD_PID.load(Ordering::SeqCst);
        if pid != 0 {
            // SAFETY: pid is a valid child PID we spawned, and kill with SIGINT
            // is safe to call even if the child has already exited (returns -1 with
            // ESRCH, which we ignore).
            unsafe {
                libc::kill(pid as i32, libc::SIGINT);
            }
        }
    });

    // SAFETY: libc::signal is safe to call here. The handler function
    // (forward_sigterm) only performs async-signal-safe operations:
    // atomic load and libc::kill.
    unsafe {
        libc::signal(
            libc::SIGTERM,
            forward_sigterm as extern "C" fn(i32) as usize,
        );
    }
}

/// Windows stub: no Unix-style signal forwarding available.
/// Child process cleanup is handled by the OS when the parent exits.
#[cfg(windows)]
pub fn register_handler() {
    // No-op. On Windows, Ctrl+C generates a console event that the
    // child process receives directly via its own console handler.
}

/// Register the child PID so the signal handler can forward signals to it.
pub fn set_child_pid(pid: u32) {
    CHILD_PID.store(pid, Ordering::SeqCst);
}

/// Clear the child PID (child exited normally).
pub fn clear_child_pid() {
    CHILD_PID.store(0, Ordering::SeqCst);
}
