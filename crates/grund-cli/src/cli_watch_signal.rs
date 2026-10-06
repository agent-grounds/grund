// Native interruption without a detached signal worker (§FS-check.6.3.3).
static WATCH_INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn watch_sigint(_: libc::c_int) {
    WATCH_INTERRUPTED.store(true, std::sync::atomic::Ordering::Release);
}

#[cfg(windows)]
unsafe extern "system" fn watch_console_interrupt(event: u32) -> i32 {
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT};
    if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
        WATCH_INTERRUPTED.store(true, std::sync::atomic::Ordering::Release);
        1
    } else {
        0
    }
}

struct WatchSignal {
    #[cfg(unix)]
    previous: libc::sigaction,
}

impl WatchSignal {
    fn install() -> Result<Self, String> {
        WATCH_INTERRUPTED.store(false, std::sync::atomic::Ordering::Release);
        #[cfg(unix)]
        {
            // Only the atomic store runs in the handler. Restore the caller's
            // disposition after watcher shutdown (§FS-check.6.3.3).
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            let mut previous: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = watch_sigint as *const () as usize;
            unsafe {
                libc::sigemptyset(&mut action.sa_mask);
            }
            if unsafe { libc::sigaction(libc::SIGINT, &action, &mut previous) } != 0 {
                return Err(format!(
                    "installing SIGINT handler: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(Self { previous })
        }
        #[cfg(windows)]
        {
            if unsafe {
                windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
                    Some(watch_console_interrupt),
                    1,
                )
            } == 0
            {
                return Err(format!(
                    "installing console interruption handler: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(Self {})
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err("native watch interruption is unsupported on this platform".to_string())
        }
    }

    fn interrupted(&self) -> bool {
        WATCH_INTERRUPTED.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Drop for WatchSignal {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::sigaction(libc::SIGINT, &self.previous, std::ptr::null_mut());
        }
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
                Some(watch_console_interrupt),
                0,
            );
        }
    }
}
