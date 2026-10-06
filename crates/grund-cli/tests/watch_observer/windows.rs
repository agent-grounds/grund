//! Real console interruption in an isolated process group (§FS-check.6.3.3).
#![cfg(windows)]
use crate::support::*;
use grund::{
    WatchObservation,
    watch_test::{self, Control},
};
use grund_core::CheckFindingSelection;

#[test]
fn watch_windows_native_console_interruption() {
    use std::os::windows::process::CommandExt;
    let _serial = crate::support::serial();
    let output = bounded_child(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "windows::watch_windows_console_child",
                "--nocapture",
            ])
            .env("GRUND_WATCH_CONSOLE_CHILD", "1")
            .creation_flags(0x00000200),
    ); // CREATE_NEW_PROCESS_GROUP: never signal Cargo.
    assert!(output.status.success(), "console child: {output:?}");
}

#[test]
fn watch_windows_console_child() {
    if std::env::var_os("GRUND_WATCH_CONSOLE_CHILD").is_none() {
        return;
    }
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    let control = Control::new(|event| {
        if matches!(event, WatchObservation::Completed(..)) {
            use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent};
            assert_ne!(
                unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, std::process::id()) },
                0
            );
        }
    });
    assert_eq!(
        watch_test::run(f.opts(), CheckFindingSelection::default(), None, control),
        1
    );
}
