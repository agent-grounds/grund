# REQ-no-data-loss: grund never eats user content

A wrong verdict can be re-run; destroyed content cannot. `grund` writes only what it owns, only when it was asked to — a `grund` invocation must never be the reason a repository lost work. The tool that organizes a project's long-term memory ([§GRUND-schema](../grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep)) is the last thing that should be able to destroy it.

## 1. The query surface never writes

`check`, ID queries, `refs`, `list`, `cover`, `config show` / `validate`, `config migrate` without `--write`, `completions`, `agent-setup-instructions`, LSP requests, and `integrations` without `--write` open files read-only. No caches, no lock files, no PID files, no "fixed it for you" rewrites — their only output is the streams ([§FS-errors.1](../functional-spec/FS-errors.md#1-streams)).

## 2. Writers touch only what they own

Five invocations write, each bounded by a stated ownership marker: `fmt --write` rewrites the citation tokens it names, and the cross-reference wrapper it owns around such a token ([§FS-fmt.6](../functional-spec/FS-fmt.md#6-cross-reference-emission)), and no other byte ([§FS-fmt.2.3](../functional-spec/FS-fmt.md#23-what-is-never-rewritten)), plus the section lock files under `.grund/sections/` and nothing else under `.grund/` ([§FS-section-lock.write](../functional-spec/FS-section-lock.md#write-who-writes-the-lock)); `fetch` atomically replaces or inserts only its validated named declaration in the configured snapshot home ([§FS-fetch.6](../functional-spec/FS-fetch.md#6-stability-and-ownership)); `init` owns the managed entrypoint block, delimited by its markers, plus the files it scaffolds ([§FS-init.3](../functional-spec/FS-init.md#3-non-intrusive-guarantees)); `integrations --write` owns four marked things and nothing else ([§FS-integrations.4](../functional-spec/FS-integrations.md#4-managed-writes---write)) — the comment-delimited block it writes into each editor and terminal configuration, its fixed resolver path and its own extension directory, the keys it records in the user's `~/.config/grund/config.toml` ([§FS-integrations.4.3](../functional-spec/FS-integrations.md#43-user-preference-and-global-agent-instructions)), and the marked `## Grund citation rendering` block in each agent's global instruction file ([§FS-integrations.4.3.8](../functional-spec/FS-integrations.md#438-the-global-instruction-files)). `config migrate --to 2 --write` owns the whole config file it migrates, on the terms of [§REQ-no-data-loss.4](REQ-no-data-loss.md#4-one-migration-may-replace-the-config) and no others. The editor transform is held to the same rule in the user's buffer ([§FS-lsp.1.4](../functional-spec/FS-lsp.md#14-live-trigger-transform)). Ownership is what makes a re-run safe: everything outside the marker survives, however many times the command runs.

## 3. Destructive is opt-in and never a side effect

`init --force` is one of the two paths that replace whole files — the canonical `AGENTS.md` and the starter files `--scaffold` lays, which `init` owns end to end ([§FS-init.3](../functional-spec/FS-init.md#3-non-intrusive-guarantees)). That is the user asking for it, and it stays bounded: the config is never overwritten, save by the one command [§REQ-no-data-loss.4](REQ-no-data-loss.md#4-one-migration-may-replace-the-config) names, and companion entrypoints keep their unrelated content. That command is the other path. No other flag, and no default run of any command, replaces a file whose content `grund` did not write. Where a spec describes a rule that `--force` reverses, it says so at that point — an unqualified "left unchanged" that is only true without the flag is the wording that loses somebody's `docs/goals.md`.

## 4. One migration may replace the config

`grund config migrate --to 2 --write` is the one command that replaces an authored `grund.toml`, and it is bounded. The owner ruled it so on agent-grounds/grund#461 on 2026-10-14, as the single named exception to "the config is never overwritten" in [§REQ-no-data-loss.3](REQ-no-data-loss.md#3-destructive-is-opt-in-and-never-a-side-effect):

- It writes only when `--write` is given.
- It writes only after its comparison of the old and new file passes for the file as it stands.
- It writes only by an atomic replace in the file's own directory, so any failure leaves the original byte for byte and no partial file.
- It loses no comment unless `--drop-comments` is given. That flag drops exactly the comments the preview listed, and the command names them again.
- It writes no other file.

After writing it says what it replaced, and how to get it back: from version control, and by previewing again without `--write`. A second exception is a change to this point, never a flag of another command. The command's contract is [§FS-config-migrate](../functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does); it serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path), whose deprecation path from version 1 would otherwise be a hand rewrite nothing checks.
