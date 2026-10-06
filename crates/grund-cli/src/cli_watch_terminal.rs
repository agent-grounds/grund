// Alternate-screen ownership preserves independent streams (§FS-check.6.2.2).
struct WatchScreen {
    eligible: bool,
    owned: bool,
    #[cfg(windows)]
    console_mode: Option<u32>,
}

impl WatchScreen {
    fn new() -> Self {
        use std::io::IsTerminal;
        let terminal = std::io::stdout().is_terminal()
            && (!std::io::stderr().is_terminal() || watch_streams_share_terminal());
        #[cfg(not(windows))]
        let eligible = terminal
            && std::env::var_os("TERM").is_some_and(|term| !term.is_empty() && term != "dumb");
        #[cfg(windows)]
        let console_mode = watch_enable_console_controls(terminal);
        #[cfg(windows)]
        let eligible = console_mode.is_some();
        #[cfg(feature = "test-watch")]
        let eligible = watch_test_terminal().unwrap_or(eligible);
        Self {
            eligible,
            owned: false,
            #[cfg(windows)]
            console_mode,
        }
    }

    fn before_publication(&mut self, format: &str) -> std::io::Result<()> {
        if format != "text" || !self.eligible {
            return self.restore();
        }
        if !self.owned {
            Self::control(b"\x1b[?1049h")?;
            self.owned = true;
        }
        Self::control(b"\x1b[H\x1b[2J")
    }

    fn restore(&mut self) -> std::io::Result<()> {
        if self.owned {
            self.owned = false;
            Self::control(b"\x1b[?1049l")?;
        }
        Ok(())
    }

    fn control(bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        out.write_all(bytes)?;
        out.flush()?;
        #[cfg(feature = "test-watch")]
        watch_capture_write(0, bytes);
        Ok(())
    }
}

impl Drop for WatchScreen {
    fn drop(&mut self) {
        let _ = self.restore();
        #[cfg(windows)]
        if let Some(mode) = self.console_mode {
            use windows_sys::Win32::System::Console::{
                GetStdHandle, STD_OUTPUT_HANDLE, SetConsoleMode,
            };
            unsafe {
                SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), mode);
            }
        }
    }
}

#[cfg(unix)]
fn watch_streams_share_terminal() -> bool {
    let mut out: libc::stat = unsafe { std::mem::zeroed() };
    let mut err: libc::stat = unsafe { std::mem::zeroed() };
    (unsafe { libc::fstat(1, &mut out) == 0 && libc::fstat(2, &mut err) == 0 })
        && out.st_dev == err.st_dev
        && out.st_ino == err.st_ino
        && out.st_rdev == err.st_rdev
}

#[cfg(windows)]
fn watch_streams_share_terminal() -> bool {
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
    unsafe {
        windows_sys::Win32::Foundation::CompareObjectHandles(
            GetStdHandle(STD_OUTPUT_HANDLE),
            GetStdHandle(STD_ERROR_HANDLE),
        ) != 0
    }
}

#[cfg(windows)]
fn watch_enable_console_controls(terminal: bool) -> Option<u32> {
    use windows_sys::Win32::System::Console::{
        ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle,
        STD_OUTPUT_HANDLE, SetConsoleMode,
    };
    if !terminal || std::env::var_os("TERM").is_some_and(|term| term == "dumb") {
        return None;
    }
    let output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut mode = 0;
    if unsafe { GetConsoleMode(output, &mut mode) } == 0 {
        return None;
    }
    if unsafe {
        SetConsoleMode(
            output,
            mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT,
        )
    } == 0
    {
        return None;
    }
    Some(mode)
}

#[cfg(not(any(unix, windows)))]
fn watch_streams_share_terminal() -> bool {
    false
}
