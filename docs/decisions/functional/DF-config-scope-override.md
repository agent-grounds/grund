# DF-config-scope-override: the committed scopes are one relation, stated once

**Status:** Accepted
**Date:** 2026-09-24

## 1. Context

`grund.toml` is written at three committed scopes — the built-in default, the project's top-level tables, and one `[[kinds]]` row — and until now this specification never said once how they relate. It said it five times, partially, and once wrongly.

The partial statements are real and each is correct where it stands: [§FS-config.3.4.8.3](../../functional-spec/FS-config.md#3483-precedence-is-row-over-global) says the row's `require_grounding` beats the `[reference]` default and the flag that sets it, [§FS-config.3.4.7.2](../../functional-spec/FS-config.md#3472-not-walked-however-the-walk-arrives) says the narrower `scan` key is the config's answer where the two disagree, [§FS-config.3.9.3](../../functional-spec/FS-config.md#393-namespace-matching) says a workspace member inherits no section from the root, [§FS-config.3.9.4](../../functional-spec/FS-config.md#394-defaults-and-precedence) states a third precedence ladder for `[citations]`, and [§FS-config.3.4.2.4](../../functional-spec/FS-config.md#3424-the-default-is-per-kind-name) states a default keyed on a kind's name rather than on a scope. A reader who wants the general rule has to induce it from five special cases, and a key added tomorrow has no rule to be held to at all.

The wrong statement was [§FS-config.2](../../functional-spec/FS-config.md#2-precedence)'s, and it contradicted [§FS-config.3](../../functional-spec/FS-config.md#3-schema) six lines below it. The first said *Layering is shallow: a value present in `grund.toml` overrides the entire corresponding default*; the second says *Every key is optional; omitted keys take the default value*. [§FS-config.3](../../functional-spec/FS-config.md#3-schema) is the true one, and the implementation has never done anything else: the loader fills a complete `Config` from the defaults before the file is opened and then assigns one field per recognized key (`config/record.rs`, `config/parse.rs`). [§FS-config.2](../../functional-spec/FS-config.md#2-precedence)'s second clause, *CLI flags override individual leaf values*, is wrong in the other direction — `--require-grounding` loses to a row that writes `require_grounding = false`, which is exactly what [§FS-config.3.4.8.3](../../functional-spec/FS-config.md#3483-precedence-is-row-over-global) already said.

Three doc-comments in `grund-core` cited [§FS-config.2](../../functional-spec/FS-config.md#2-precedence) for the phrase "merged over the built-in defaults" — the opposite of what [§FS-config.2](../../functional-spec/FS-config.md#2-precedence) said. The implementation had been citing the section for the claim it refuted.

Serves [§GOAL-configurable](../../goals.md#goal-configurable-every-default-is-overridable): *can I set this per kind?* becomes one read rather than a hunt through ten table sections.

## 2. Decision

### 2.1 One relation, stated once, and stated as a relation

[§FS-config.principle](../../functional-spec/FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins) carries the whole rule, and every other point keeps its own words and is cited from there rather than restated. Its children are **named**, not numbered: a chapter cited from inside `grund-core`'s source tree cannot afford positional children, because inserting one silently re-points every citation of the ones after it while `<§>FS-config.principle.4` still resolves and simply means something else.

The rule is stated as a relation over scopes rather than as an enumeration of three, so that a scope introduced later — finer than a kind, or between the project and the kind — takes its place without the chapter being rewritten. No such scope exists today.

[§FS-config.requirements](../../functional-spec/FS-config.md#requirements-what-the-config-contract-holds-to) carries the config contract as eight **marked** points, because a direction that binds the next key and a statement about how `grund` behaves today are different kinds of sentence and reading one as the other is the mistake the chapter exists to prevent.

### 2.2 The inventory is derived in two stages

[§FS-config.principle.inventory](../../functional-spec/FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) claims a *complete* list of the settings admitted at both the project and the kind scope. A claim of completeness is only worth what its derivation is worth, so the derivation is recorded here to be re-run rather than trusted.

**Stage 1 — the candidate set, from the parse sites.** Every committed parse site is enumerated: `config/parse.rs` (the project tables), `config/kind_table.rs` (a `[[kinds]]` row), and `config/citations.rs` (`[citations]` and `[citations.<KIND>]`). This is complete by construction: a key no parser accepts cannot be written at that scope.

**Stage 2 — the criterion, from the resolution sites.** A pair exists if and only if one place in the code chooses between a narrower-scope value and a wider-scope value **of the same setting**. Stage 1 bounds the search; stage 2 decides.

The four resolution sites, and the pairs they establish:

| Resolution site | Pair |
|---|---|
| `config/grounding.rs:207` | `[reference] require_grounding` / a row's `require_grounding` |
| `config/grounding.rs:208` | `[reference] grounding_level` / a row's `grounding_level` |
| `config/kind.rs:92` | `[id] format` / a row's `format` |
| `checker/citations.rs:443-446` | `[citations] default` / `[citations.<KIND>] default` |

Stage 2 is what rejects `[output] format` against `[id] format` — one name, two settings, and no site chooses between them — and what rejects `[scan]` against a row's `scan` for the same reason. It accepts `[citations] default` whatever shape the fallback is written in. A fifth key added later is caught by stage 2 regardless of what it is named.

### 2.3 Both clauses of [§FS-config.2](../../functional-spec/FS-config.md#2-precedence) are withdrawn, and the section survives

[§FS-config.2](../../functional-spec/FS-config.md#2-precedence) keeps its number and its heading and states the corrected source rule in one paragraph. It is not deleted, because the sections numbered 3 through 6 in that file must not renumber under the ten sites that cite it.

**The first clause, with the run behind it.** In an empty directory, a `grund.toml` whose `[reference]` table names one key:

```toml
grund_config_version = 1

[reference]
strict = false
```

```
$ grund config show
grund_config_version = 1

[reference]
marker = "§"
trigger = "$$"
strict = false
shorthand = "canonical"
…
```

`marker` and every other `[reference]` key kept its default. Writing one key did not override "the entire corresponding default".

**The second clause, with the run behind it.** A row that writes `require_grounding = false`, under the flag that is supposed to override it:

```
$ grund check --require-grounding
docs/functional-spec/FS-001-alpha.md:1: warning: declared but never cited: FS-001-alpha

$ sed -i '/require_grounding = false/d' grund.toml && grund check --require-grounding
docs/functional-spec/notes.rs:1: error: ungrounded source file: no § citation to a declared ID
docs/functional-spec/FS-001-alpha.md:1: warning: declared but never cited: FS-001-alpha
```

The one row line is the whole difference. The flag enters at the project scope it spells and the row is narrower, per [§FS-config.principle.cli](../../functional-spec/FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells).

The withdrawal is recorded here rather than performed silently in `FS`, because a live sentence under a declaration this heavily cited is retracted with its evidence attached.

## 3. Alternatives rejected

- **A declaration of its own, `FS-config-scopes`.** Citations are the interface: a rule that moves out of `FS-config` later rewrites every site that cites it. Both chapters stay in `FS-config.md`.
- **Enumerating three scopes instead of stating a relation.** A fourth scope would then be a rewrite of the chapter rather than an entry in it.
- **Numbered children, `principle.1` … `principle.9`.** Positional beneath a named handle: inserting a point renumbers its siblings and re-points every citation of them without failing anything, because the old coordinate still resolves.
- **Deriving the inventory by a parser-surface name intersection alone.** It yields one name, and that name is ambiguous:

  ```
  $ comm -12 \
      <(grep -oE "^\s+\"[a-z_]+\" =>" crates/grund-core/src/config/kind_table.rs \
        | grep -oE "[a-z_]+" | sort -u) \
      <(grep -oE "\(\"[a-z_.]+\", \"?[a-z_]*\"?" crates/grund-core/src/config/parse.rs \
        | sed -E "s/.*, \"?([a-z_]+)\"?.*/\1/" | sort -u)
  format
  ```

  It misses three of the four pairs — `require_grounding` and `grounding_level` are dispatched by a `("kinds", key @ (…))` arm whose table name is `kinds` rather than `reference`, and `[citations] default` is parsed in `config/citations.rs` and appears in neither surface — and its one hit is ambiguous, because `[id] format` and `[output] format` are both project keys of that name and only one of them is a pair.
- **Deriving it by an effective-value fallback grep alone.** It finds three of the four and admits one non-pair:

  ```
  $ grep -rn 'unwrap_or(&config\.\|unwrap_or(self\.' crates/grund-core/src
  crates/grund-core/src/config/grounding.rs:207:            kind.require_grounding.unwrap_or(self.require_grounding),
  crates/grund-core/src/config/grounding.rs:208:            kind.grounding_level.unwrap_or(self.grounding_level),
  crates/grund-core/src/config/kind.rs:92:        self.format.as_deref().unwrap_or(&config.id_format)
  ```

  It misses `checker/citations.rs:443-446`, which is written as an `if let Some(default) { return … }` with the global default as the trailing expression, and a grep widened to catch that shape admits `config/grounding.rs:197`, the homeless kind's fallback, which is not a scope pair at all. A shape-matching grep cannot be the criterion; what a site *chooses between* is.
- **Sweeping all ten table sections with project-only prose.** The obligation is discharged centrally by [§FS-config.principle.inventory](../../functional-spec/FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) and is forward-looking; repeating it per section is ten places to drift.
- **Renaming `[scan]` or a row's `scan` so the principle appears to apply.** They are two settings that share a spelling, [§FS-config.3.4.7.2](../../functional-spec/FS-config.md#3472-not-walked-however-the-walk-arrives) already reconciles them, and renaming a config key is a compatibility change bought for a cosmetic consistency.

## 4. Consequences

No behavior changes. No key is renamed, no key gains a scope it did not have, `grund_config_version` stays 1, and every existing `grund.toml` means exactly what it meant. What changes is what a reader is owed: a key admitted at more than one scope says so at its own point, and [§FS-config.principle.inventory](../../functional-spec/FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) is the list that must be added to in the same change that admits a fifth.

One reading of the specification breaks: a reader who relied on [§FS-config.2](../../functional-spec/FS-config.md#2-precedence)'s withdrawn clause and expected `strict = true` to discard the other `[reference]` defaults was reading a description that never matched `grund`.

Four follow-ups stay follow-ups, one ticket each: per-kind values for `inline_style` and the note budgets; an error that names a key written at a scope that does not admit it instead of calling it unknown; provenance in `grund config show`; and a user-facing `docs/user-facing/config.md`.
