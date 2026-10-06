# DF-watch-terminal-loop: a terminal watch loop preserves ordinary check reports

**Status:** Accepted
**Date:** 2026-10-06

## 1. Context

Issue #473 asks for the terminal save/check loop planned by [§FS-check.6](../../functional-spec/FS-check.md#6-watch-mode---watch) and [§GOAL-fast-feedback](../../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). The owner approved the full baseline proposal on [issue #473](https://github.com/agent-grounds/grund/issues/473), comment 6006042060, including release of specification and implementation. The earlier roadmap's silent clean text and self-contained JSON reports disagreed with the ordinary success and finding NDJSON contracts ([§FS-check.2.1.3](../../functional-spec/FS-check.md#213-the-success-line), [§FS-check.2.1.4](../../functional-spec/FS-check.md#214-json)).

## 2. Decision

### 2.1 Preserve reports and stream ownership

Each run uses ordinary checking, finding selection, order, output defaults/overrides, rendering and status. Clean text prints `success`; clean JSON is empty and has no observable boundary. There is no framing extension, timestamp or banner ([§FS-check.6.2.1](../../functional-spec/FS-check.md#621-exact-stream-contract)). Freezing config at startup or adding another checking engine would break one-shot parity.

### 2.2 React to all effective local inputs

Native notification backends on Linux/macOS/Windows observe the complete effective local-input inventory, including discovery candidates, global ignores and replacement anchors ([§FS-check.6.1.3](../../functional-spec/FS-check.md#613-effective-input-inventory)). Subscribe before reading; add and re-resolve before retiring coverage ([§FS-check.6.1.1](../../functional-spec/FS-check.md#611-subscribe-before-reading)). Narrow paths still need resolution-wide inputs. Unsupported notification-silent mounts receive no polling fallback.

Idle debounce is 100 ms quiet / 500 ms maximum. During scans retain one pending rerun; serialize scans/publication ([§FS-check.6.1.2](../../functional-spec/FS-check.md#612-bounded-debounce-and-serialized-work)). Full rediscovery/rescan handles uncertain/lost events; incremental graph work is unnecessary. Failed watcher setup/runtime subscriptions or recovery exit explicitly rather than leaving stale results ([§FS-check.6.1.4](../../functional-spec/FS-check.md#614-uncertain-notifications-and-recovery)).

### 2.3 Keep repairable failures resident and interruption precise

Ordinary 0/1/2 runs, including config/read failures and config-dependent trial refusals, remain resident with discovery/usable inventory retained ([§FS-check.6.3.1](../../functional-spec/FS-check.md#631-recoverable-runs)). Static invocation errors still terminate. Fatal watcher failures restore the screen, diagnose the failed operation on stderr and exit 2, overriding the last verdict ([§FS-check.6.3.2](../../functional-spec/FS-check.md#632-fatal-watcher-failures)).

SIGINT discards queued work and an unpublished in-flight result after its synchronous scan finishes privately. Publication already begun completes; completion includes both streams flushed. Return the last completed status, or 2 with `error: interrupted before the first check completed` if none exists. Restore the screen, release subscriptions and join workers ([§FS-check.6.3.3](../../functional-spec/FS-check.md#633-interrupt-and-completion)). Finishing scans privately avoids cancellation inside shared engine code; shutdown can wait for that scan.

### 2.4 Own an alternate screen only when streams permit it

Eligible terminal text uses stdout enter `\x1b[?1049h`, clear `\x1b[H\x1b[2J` and restore `\x1b[?1049l`. Stderr must share that terminal or be redirected. Shared stderr clears with the report; redirected stderr appends. Redirected stdout, distinct terminals and dumb terminals append without controls. Restore before JSON; JSON never enters/clears. Exit restores pre-watch content ([§FS-check.6.2.2](../../functional-spec/FS-check.md#622-owned-terminal-screen)). Cursor/line-count erasure would risk deleting unrelated terminal content or redirected bytes.

## 3. Consequences and proof

Full rescans cost one ordinary check per burst. Recursive watches can consume OS resources for ignored descendants and fail explicitly at limits. Parent anchors permit replacement/creation without a polling loop; shallow path-filtered ancestor/home coverage avoids watching unrelated home trees recursively. No network, daemon, binding watch API, TUI or LSP dependency is introduced ([§FS-check.6.4](../../functional-spec/FS-check.md#64-scope)).

Private test builds observe subscription and flushed completion without production markers, and inject clocks, barriers and backend failures ([§AR-bindings.3](../../architecture/AR-bindings.md#3-cratesgrund-cli-the-cli-binary)). Subprocess tests establish the missing CLI outcome first; completion/barrier tests and PTYs must pin the remaining lifecycle before the feature ships. Silence cannot prove a clean JSON run.

## release-note: Release note

`grund check --watch [<path>]` now checks immediately and stays resident, rechecking effective local inputs after saves. Ctrl-C returns the last completed status; interruption before any completion and fatal watcher errors return 2. Previously this flag was rejected. Ordinary one-shot reports and verdicts remain unchanged. Eligible text terminals use an alternate screen restored on exit; redirected output appends. JSON remains finding-only NDJSON with invisible clean runs. Native notifications are required; notification-silent mounts have no polling fallback ([§FS-check.6](../../functional-spec/FS-check.md#6-watch-mode---watch)).
