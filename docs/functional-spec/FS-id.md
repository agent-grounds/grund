# FS-id: grund proposes IDs for new declarations

The `id` subcommand emits one conflict-free ID for a new declaration — `<KIND>-<NNN>-<slug>` under the default format (§2.1). The name is deliberate: `id` is the pure allocator, while `new` is reserved for a future command that would create a declaration stub ([§DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub](../decisions/functional/DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub.md#df-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-creation)). Authors, agents and editor "new declaration" actions all call this one primitive (§8), so the next number for a kind and the canonical slug for a title are computed in exactly one place. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (no human picks the next number by reading a directory listing) and [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) (proposed IDs cannot collide with existing declarations).

## 1. Inputs

```
grund id <KIND> "<title>" [<path>] [--width <N>] [--explain] [--format text|json]
```

- `<KIND>` — required: a configured *citable* `[[kinds]]` entry ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)); an unknown or non-citable kind is refused (§1.1).
- `<title>` — required: a free-form title for the new declaration, slugged per §3.
- `<path>` — the directory whose tree is scanned for the next free number; defaults to the current directory and is resolved as by every `grund` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): config discovery walks up to `grund.toml`, else takes the defaults.
- `--width <N>` — minimum digit width of the number, default `3` ([§DF-id-number-width](../decisions/functional/DF-id-number-width.md#df-id-number-width-grund-id-zero-pads-minted-numbers-to-a-default-width-of-3)), a floor rather than a cap (§1.2).
- `--explain` — text only: also print a one-line `next:` hint on stderr (§2.3). No effect in `--format json`, which already carries the `folder`.
- `--format text|json` — output shape (§2). Default `text`.

`id` is non-interactive — no prompt, no confirmation ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). Text `stdout` is always exactly the proposed ID, so `$(grund id …)` is safe.

### 1.1 Unknown and non-citable kinds

An unknown kind is a CLI-level error, exit `2` (§6): an `error:` line naming the kind, then a `known kinds: …` line listing the citable kinds, on stderr — the shape `grund list --kind <unknown>` produces, so a typo never reads as a clean run. A configured **non-citable** kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is refused in the same shape with the reason in place of "unknown" — `error: kind `skill` declares no IDs — skills/ is not a citable home` — because it has no ID to allocate.

### 1.2 Number width

The default `3` matches the canonical form's `-NNN-`; [§DF-id-number-width](../decisions/functional/DF-id-number-width.md#df-id-number-width-grund-id-zero-pads-minted-numbers-to-a-default-width-of-3) says why 3, and why a per-invocation flag rather than an `[id]` config key (for now). The number is zero-padded to at least `--width` digits; a number that already has more is emitted as-is (`FS-1000`), so the default is a floor, not a cap.

## 2. Outputs

### 2.1 `--format text` (default)

One line on stdout: the proposed ID, with no marker prefix, then a newline. Its shape follows the kind's effective format — its `[[kinds]].format` override when present, otherwise `[id].format` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — whose placeholders decide what is allocated (§4.1 for number-less forms):

```
$ grund id FS "User can log in with email"        # default [id] format = {kind}-{number}-{slug}
FS-008-user-can-log-in-with-email
$ grund id FS "User can log in with email"        # a repo whose [id] format = {kind}-{slug}, like grund itself
FS-user-can-log-in-with-email
```

`id` never invokes a configured fetcher ([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)). Stderr is empty on success unless `--explain` was passed (§2.3). The `path:line:` prefix of [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) does not apply: `id` synthesizes; it points at no source location.

### 2.2 `--format json`

```json
{"id":"FS-008-user-can-log-in-with-email","kind":"FS","number":8,"slug":"user-can-log-in-with-email","folder":"","file":"requirements.md"}
```

`folder` and `file` are the kind's configured `[[kinds]]` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)); usually exactly one is non-empty. They are there so an editor "create new declaration" action can place the declaration without a second lookup. Under a number-less effective format, `number` is `null` (§4.1).

### 2.3 `--explain` (text only)

Stdout is unchanged — still the bare ID — and stderr carries one more line: where to put the declaration and how to start it. For a kind with a configured `file`, the hint names that file and an H2 by convention, or the H1 where the file holds the kind's single declaration ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), as `GRUND`'s does:

```
$ grund id FS "User can log in with email" --explain
FS-008-user-can-log-in-with-email
next: add the declaration to requirements.md  (H2: `## FS-008-user-can-log-in-with-email: <one-line statement>`), then cite it as §FS-008-user-can-log-in-with-email
```

For a kind with a configured `folder`, the hint names the new declaration file under that folder and uses an H1; for a kind with neither, it names the H1 and the citation but no path. `E2E` is the exception to the folder form: its declaration is a case directory, not a Markdown file ([§FS-config.3.4.4.3](FS-config.md#3443-e2e-is-configured-not-a-default)), so its hint names the case directory to create under that folder, with `expected.exit` and fixtures. The bare ID still composes in `$(…)`, while a person who ran `grund id` by hand gets the next step without recalling the layout. It creates no file (§7).

## 3. Slug derivation

The title becomes a slug deterministically: the same title and the same configured `slug_pattern` give the same slug on every platform, in every locale ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

1. Unicode-normalize the title to NFKD and strip combining marks (`Café log-in` → `Cafe log-in`).
2. Lower-case, ASCII only; a non-ASCII letter that survives step 1 passes to step 3 unchanged and is filtered there.
3. Replace every run of characters outside the repeating character class of the configured `slug_pattern` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — its last `[...]` bracket expression, or `[a-z0-9-]` when it has none — with a single `-`. Under the default pattern, spaces, punctuation and quotes all become `-`.
4. Trim leading and trailing `-`.
5. Collapse runs of two or more `-` into one.
6. Truncate to 60 characters at the nearest preceding `-`, so a slug never ends mid-word.

An empty slug is refused (§3.1).

### 3.1 An empty slug

When no character of the title is a slug character, the slug is empty and `id` exits `1` with a bare query-failure line on stderr, no `error:` prefix (§6):

```
title produces empty slug after normalization: "<original title>"
```

`id` invents no fallback — a meaningless slug is worse than an error — so the author supplies a title with at least one slug character.

## 4. Next-number derivation

`id` scans exactly what `check` walks ([§FS-check.1](FS-check.md#1-inputs), [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)) and collects every declaration of the requested `<KIND>`. The proposed number is `max(existing numbers) + 1`, or `1` if the kind has none.

Holes below the maximum are **not** filled: with `FS-001`, `FS-002` and `FS-004` declared, `FS-003` is never issued. Numbers are issued strictly above the maximum, because an ID once removed may still be cited from outside the tree — PRs, chat, mirrored repos — and filling its hole would silently change what those citations point at. This is [§FS-non-goals.4](FS-non-goals.md#4-cross-workspace-id-renaming) (no rename) applied to allocation.

A failed scan (I/O, a malformed file) exits `2` with the underlying error and never falls back to a guess: a number allocated against an incomplete view of the tree could collide.

### 4.1 Number-less ID formats

When the kind's effective format (§2.1) has no `{number}` — `{kind}-{slug}`, the form `grund` itself uses — there is nothing to derive: the proposed ID is the format with `{kind}` and `{slug}` substituted. `--width` is accepted and has no effect, and the JSON `number` is `null`. The collision check (§5) still runs and carries more weight: with no number to tell them apart, two declarations sharing a kind and a slug are the same ID.

When the format has no `{slug}` — `{kind}-{number}` — the title is still required: it still produces the slug, which is refused when empty (§3.1) and reported as the JSON `slug`, and has no part in the ID. The proposed ID is the kind and the next number.

Neither one-component format has the number-only citation shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)), so neither can produce a [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) finding.

## 5. Collision check

After deriving slug and number, `id` verifies that the full proposed ID is not already declared in the scanned tree — belt-and-suspenders against:

- a configured `slug_pattern` that admits ambiguity, e.g. one loosened after declarations were written under the strict default;
- a `--width` change that re-zero-pads existing IDs into the candidate.

The declarations checked include JSON value declarations from opted-in kind homes: `id` may reject a candidate already declared in JSON, but stays a Markdown-oriented allocator that never creates or edits JSON ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)).

A collision exits `1` with a bare query-failure line on stderr, no `error:` prefix (§6):

```
proposed ID `FS-user-login` already declared at docs/functional-spec/FS-user-login.md:1
```

Authors disambiguate by editing the title.

## 6. Exit codes

- `0` — proposed ID emitted.
- `1` — empty slug (§3.1) or collision (§5). These are query failures — the request was well-formed but has no ID to return — so they print a bare stderr line with no `error:` prefix, as ID queries do for `ID not found` ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)).
- `2` — a scan or I/O error, an unknown kind, an unknown `--format`, or any other CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). These print `error: <message>` on stderr — the prefix CI scripts grep for to tell a launch-time failure from a clean run.

## 7. What `id` does **not** do

- It does not create the file. Writing a stub is the caller's job — an `$EDITOR` invocation for an author, the `New file` action for an IDE plugin, a follow-up `Write` for an agent — and `id` stays a pure function from `(kind, title, tree)` to `id`: three callers create files three ways, and baking one in shrinks the surface.
- It does not modify `grund.toml`, the scanned tree, or any existing declaration.
- It reserves no number. Two parallel `grund id FS …` calls on one tree may both propose `FS-008-…`; the loser sees the collision when their declaration is committed and `grund check` runs. A reservation is state, and `grund` is stateless ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)).

## 8. Why this exists

Three callers, one source of truth:

1. **Authors.** Picking the next free number by listing a directory invites a typo that creates a duplicate `grund check` catches hours later; `id` removes the typo class.
2. **Agents.** An LLM cannot reliably read a directory listing and increment the right number, and even a right answer drifts with the next file added. `grund id` is cheap, deterministic, and committed to the same regex grammar as the checker.
3. **The optional LSP server.** A "new declaration" code action in [§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server) would need the number `grund id` would compute; sharing the engine through `grund-core` makes exactly one allocator, not three subtly different ones.
