//! Private boundary evidence for §FS-check.6.1–§FS-check.6.3 and §AR-bindings.3.
//! Uses the native backend on Linux/macOS/Windows; no filesystem polling.

#[path = "watch_observer/aliases.rs"]
mod aliases;
#[path = "watch_observer/coordination.rs"]
mod coordination;
#[path = "watch_observer/global_ignore.rs"]
mod global_ignore;
#[path = "watch_observer/inventory.rs"]
mod inventory;
#[path = "watch_observer/lifecycle.rs"]
mod lifecycle;
#[path = "watch_observer/parity.rs"]
mod parity;
#[path = "watch_observer/probes.rs"]
mod probes;
#[path = "support/watch_observer.rs"]
mod support;
#[path = "watch_observer/terminal.rs"]
mod terminal;
#[path = "watch_observer/windows.rs"]
mod windows;
