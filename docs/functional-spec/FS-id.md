# FS-id: grund proposes IDs for new declarations

The `id` subcommand prints one conflict-free ID for a new declaration in the kind's effective format ([§FS-id.2.1](FS-id.md#21---format-text-default)): `<KIND>-<NNN>-<slug>` under version 1's default, `<KIND>-<slug>` under version 2's ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) and [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker,
citation, shorthand, canonical form), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, fetcher).

## requirements: What every proposed ID holds to

What `id` holds to with every other command is stated where they share it: the same request over the same tree prints the same bytes ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes), which names `id`), it reads the declaration set every command reads ([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set)), it invokes no fetcher ([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)), and its exits keep [§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api)'s mapping.

### requirements.check-accepts: The ID `id` proposes is one `check` accepts as a new declaration

Declared as proposed, the ID neither collides nor fails the grammar: where its format has a number, it is numbered strictly above every declaration of its kind in the project, and refused rather than guessed where the scan is incomplete ([§FS-id.4](FS-id.md#4-next-number-derivation)); it is refused rather than printed where it is already declared, value-source declarations included ([§FS-id.5](FS-id.md#5-collision-check)); and it is spelled in the kind's effective format with a slug drawn from the configured `slug_pattern`, refused rather than printed where the pattern rejects that slug ([§FS-id.3](FS-id.md#3-slug-derivation)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Inputs

```
grund id <KIND> "<title>" [<path>] [--width <N>] [--explain] [--format text|json]
```

- `<KIND>` — required: a configured *citable* kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) of the project the run loads ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)).
- `<title>` — required: the new declaration's title, slugged per [§FS-id.3](FS-id.md#3-slug-derivation).
- `<path>` — which project it numbers from is [§FS-scan.path.id](FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers).
- `--width <N>` — minimum digit width of the number ([§FS-id.1.2](FS-id.md#12-number-width), [§DF-id-number-width](../decisions/functional/DF-id-number-width.md#df-id-number-width-grund-id-zero-pads-minted-numbers-to-a-default-width-of-3)).
- `--explain` — also print a `next:` hint on stderr ([§FS-id.2.3](FS-id.md#23---explain-text-only)).
- `--format text|json` — output shape ([§FS-id.2](FS-id.md#2-outputs)). Default `text`.

An unknown or non-citable `<KIND>` is a CLI-level error, exit `2`, in the shape `grund list --kind` refuses with ([§FS-list.1.1](FS-list.md#11---kind)): an `error:` line naming the kind as unknown, or with [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s reason where it is configured and declares no IDs, then a `known kinds:` line naming the citable kinds, or [§FS-list.1.1.1](FS-list.md#111-a-scope-with-no-citable-kind)'s line where there are none, both on stderr.

```
error: unknown kind `NOPE`
known kinds: FS
```

### 1.2 Number width

The number is zero-padded to at least `--width` digits, default `3`; a number that already has more is printed as it is (`FS-1000`), so the width is a floor, not a cap. It is a flag only, and no config key sets it ([§DF-id-number-width](../decisions/functional/DF-id-number-width.md#df-id-number-width-grund-id-zero-pads-minted-numbers-to-a-default-width-of-3)).

## 2. Outputs

### 2.1 `--format text` (default)

One line on stdout: the proposed ID, with no marker, then a newline, so `ID=$(grund id …)` captures exactly the ID. Its shape is the kind's effective format ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), whose placeholders decide what is allocated ([§FS-id.4](FS-id.md#4-next-number-derivation)):

```
$ grund id FS "Password reset"      # effective format {kind}-{number}-{slug}
FS-002-password-reset
$ grund id FS "Printing in 3D"      # effective format {kind}-{slug}
FS-printing-in-3d
```

On success, stderr is empty unless `--explain` asks for its line ([§FS-id.2.3](FS-id.md#23---explain-text-only)).

### 2.2 `--format json`

One object on stdout, its keys in this order:

```json
{"id":"FS-003-user-can-reset-password","kind":"FS","number":3,"slug":"user-can-reset-password","folder":"docs/functional-spec","file":""}
```

`number` is `null` under a format with no `{number}` ([§FS-id.4](FS-id.md#4-next-number-derivation)). `folder` and `file` are the kind's configured home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). In a version-2 config, where a kind may have several folders ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), `folder` is the first listed and the object adds `"folders"`, every folder in written order, right after it; a version-1 object has no `folders`.

### 2.3 `--explain` (text only)

Stdout is still the bare ID, and stderr carries one more line: where to put the declaration and how to start and cite it.

```
$ grund id FS "Password reset" --explain
FS-002-password-reset
next: add the declaration to requirements.md  (H2: `## FS-002-password-reset: <one-line statement>`), then cite it as §FS-002-password-reset
```

Where the line points follows the kind's home:

- A configured file: the first one its row lists, with an H1 where the kind's first declaration in that file is an H1, or, with none there, where the file's H1 title is followed by headings of its own; otherwise an H2.
- A configured `folder`: the new declaration file under it, with an H1. `E2E` names instead the case directory to create under its folder, with `expected.exit` and fixtures, because its declaration is a case directory ([§FS-config.3.4.4.3](FS-config.md#3443-e2e-is-configured-not-a-default)).
- Neither: the H1 and the citation, with no path.

Under `--format json` the flag has no effect.

## 3. Slug derivation

The same title and the same configured `slug_pattern` give the same slug on every platform and in every locale ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)):

1. Unicode-normalize the title to NFKD and strip combining marks (`Müller café` → `Muller cafe`).
2. Lower-case ASCII letters; a non-ASCII letter that survives step 1 is left for step 3 to filter.
3. Replace every run of characters outside the repeating character class of the configured `slug_pattern` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — its last `[...]` bracket expression, or `[a-z0-9-]` when it has none — with a single `-`.
4. Trim leading and trailing `-`, even where the class admits it: `---login---` becomes `login`.
5. Collapse runs of two or more `-` into one.
6. Where the slug is longer than 60 characters, cut it before the last `-` among its first 60, or at 60 where they hold none.

The whole proposed ID (kind, number and slug in the kind's effective format) is then matched against the compiled `[id]` grammar `check` accepts a new declaration by ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), so `id` prints only an ID `check` would accept. A slug that comes out empty, and a candidate that grammar refuses, are query failures: `id` exits `1` with one bare line on stderr ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)) and invents no other slug:

```
title produces empty slug after normalization: "<original title>"
title produces slug "<slug>" that slug_pattern "<slug_pattern>" does not accept: "<original title>"
```

Step 3 reads only the pattern's class, so a pattern that requires a leading letter, or whose class lacks `-`, can still refuse what the steps yield.

## 4. Next-number derivation

`id` collects the declarations of `<KIND>` the project declares, read as `check` reads them ([§FS-scan.path.id](FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers)), and proposes one past their highest number, or `1` if the kind has none. A hole below the highest is never filled, because a removed ID may still be cited from outside the tree and reissuing it would change what those citations name ([§FS-non-goals.4](FS-non-goals.md#4-cross-workspace-id-renaming)). Where the scan cannot read or parse a file, `id` exits `2` with that failure ([§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)) and never falls back to a guess, since a number counted on part of the tree could collide.

A format with no `{number}`, such as `{kind}-{slug}`, has nothing to number: the proposed ID is the format with `{kind}` and `{slug}` filled in, and `--width` is accepted and has no effect. A format with no `{slug}`, such as `{kind}-{number}`, still requires the title: it produces the slug, which is refused when empty ([§FS-id.3](FS-id.md#3-slug-derivation)) and reported as the JSON `slug`, and has no part in the ID.

## 5. Collision check

`id` refuses a proposal already declared in the project ([§FS-scan.path.id](FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers)), value-source declarations included ([§FS-values.6.2](FS-values.md#62-value-source-declarations-in-the-catalog)). Under a number-less format this check is what keeps a proposal new, since two declarations sharing a kind and a slug are one ID ([§FS-id.4](FS-id.md#4-next-number-derivation)). The refusal exits `1` with a bare query-failure line on stderr ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)):

```
proposed ID `FS-existing` already declared at docs/functional-spec/FS-existing.md:1
```

## 6. Exit codes

- `0` — the proposed ID was printed.
- `1` — a well-formed request with no ID to return, one bare line on stderr ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)): a slug that is empty or that the pattern refuses ([§FS-id.3](FS-id.md#3-slug-derivation)), or a collision ([§FS-id.5](FS-id.md#5-collision-check)).
- `2` — a CLI-level error, `error: <message>` on stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)): an unknown or non-citable kind ([§FS-id.1](FS-id.md#1-inputs)), an unknown `--format`, a failed scan ([§FS-id.4](FS-id.md#4-next-number-derivation)), or any other CLI-level error.

## 7. What `id` does **not** do

- It writes nothing: no declaration file, and no change to `grund.toml`, the scanned tree or any existing declaration. Creating the declaration is the caller's job ([§DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub](../decisions/functional/DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub.md#df-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-creation)).
- It reserves no number: two runs on one tree may both propose the same number, and the loser sees the collision once both are declared and `check` runs; a reservation would be state, which `grund` does not keep ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)).

## 8. Why this exists

Picking the next number by listing a directory invites a typo that makes a duplicate, which `check` reports only later, and an agent cannot reliably read a listing and increment the right number, while even a right answer drifts with the next file added. Authors, agents and editor actions all ask `grund id`, so the next number of a kind and the slug of a title are computed in one place, cheaply, deterministically and from the grammar `check` reads ([§FS-id.3](FS-id.md#3-slug-derivation)).
