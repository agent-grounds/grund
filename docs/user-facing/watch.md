# Terminal feedback with `grund check --watch`

`grund check --watch [PATH]` checks immediately and remains in the terminal,
checking again after effective local inputs change. All ordinary check options
work, including `--format`, `--full`, `--suggestions`, grounding, trial rules and
finding selection. A path narrows the report while resolution still observes the
whole project ([§FS-check.6.2](../functional-spec/FS-check.md#62-each-run-is-a-plain-grund-check)).

```bash
grund check --watch
grund check --watch src/ --only dangling
grund check --watch --format json > findings.ndjson
```

Save a file to update the report. A clean text run prints `success`. JSON contains
only the ordinary finding NDJSON: clean runs produce no bytes, and the concatenated
stream does not reveal every run boundary or confirm that a clean run completed.
Redirected stdout and stderr append each run's ordinary bytes; neither receives
screen clearing controls ([§FS-check.6.2.1](../functional-spec/FS-check.md#621-exact-stream-contract)).

**Terminal screen.** Eligible text output uses an alternate screen when stdout
is a capable terminal and stderr shares it or is redirected. Each publication
clears that screen; Ctrl-C restores the content from before watching. Redirected
stderr keeps appending even while stdout clears. Distinct terminals, dumb terminals
and redirected stdout append without controls. JSON never clears; switching a
config default from text to JSON restores the screen, and switching back can
acquire it again. An explicit format always wins over config defaults
([§FS-check.6.2.2](../functional-spec/FS-check.md#622-owned-terminal-screen)).

**Detection.** Linux, macOS and Windows use native `notify` backends. Watch observes
both config names, ancestor workspace claims, members and member-glob parents,
resolution-wide source roots and kind homes, catalogs, indexes, checker probes,
followed file targets and effective ignore/Git discovery inputs. Missing paths
keep parent anchors so creation or replacement can restore coverage. Narrowing
the report does not narrow these necessary inputs
([§FS-check.6.1.3](../functional-spec/FS-check.md#613-effective-input-inventory)).

The idle debounce waits for 100 ms of quiet, with a maximum of 500 ms from the
first pending event. Write bursts and atomic saves can coalesce; a save lasting
longer can expose intermediate states. Scans and publication are serialized,
and changes during work arrange another run. Read/access events do not cause
runs ([§FS-check.6.1.2](../functional-spec/FS-check.md#612-bounded-debounce-and-serialized-work)).

There is no polling fallback or polling setting. Network mounts and pseudo-filesystems
that do not deliver notifications are unsupported. Backend permissions and native
watch-resource limits can prevent setup; broad recursive roots can consume watches
for ignored descendants. Reported lost events, overflow and queue saturation force
full rediscovery and rescanning. Replacement reattaches subscriptions
([§FS-check.6.1](../functional-spec/FS-check.md#61-change-detection),
[§FS-check.6.1.4](../functional-spec/FS-check.md#614-uncertain-notifications-and-recovery)).

**Repair and shutdown.** Ordinary findings, invalid configuration, read failures
and config-dependent trial refusals remain resident. Fix the input and watch
checks again, retaining usable discovery coverage during the error. Static CLI
errors terminate before watching. Fatal native setup, subscription or recovery
failures restore the screen, identify the watching operation/input on stderr and
exit 2 ([§FS-check.6.3.1](../functional-spec/FS-check.md#631-recoverable-runs),
[§FS-check.6.3.2](../functional-spec/FS-check.md#632-fatal-watcher-failures)).

Ctrl-C returns the last completed check's status: 0 for the selected report with
no errors, 1 for selected findings, or 2 for an operational failure. Before any
check completes, interruption prints `error: interrupted before the first check completed`
and returns 2. An active scan finishes privately and its unpublished result is
discarded; publication already begun finishes both streams and flushes. Shutdown
can therefore wait for that work. Pending work is discarded and native subscriptions
are released before exit ([§FS-check.6.3.3](../functional-spec/FS-check.md#633-interrupt-and-completion)).
