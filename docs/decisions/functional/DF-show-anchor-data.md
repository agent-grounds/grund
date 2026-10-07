# DF-show-anchor-data: show JSON carries the heading anchor grund already derives

**Status:** Accepted
**Date:** 2026-10-07

## 1. Context

[§DF-neural-link-generation](DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command) made composing a clickable citation the writing agent's job and gave it a recipe: read the declaration's path from `grund <ID> --format json`, then slugify the heading's rendered text by hand ([§DF-neural-link-generation.5](DF-neural-link-generation.md#5-recipe)). It recorded a fallback for the case where that proved unreliable: revisit a *data* surface first, such as the anchor as a field in the ID query's JSON, before any command ([§DF-neural-link-generation.2](DF-neural-link-generation.md#2-why), item 4).

It proved unreliable. The JSON object carried `path` and `line` but no fragment, so an agent had to read the heading at that line and re-implement the `github` profile's slugger, the same algorithm [§DF-github-anchor-fidelity](DF-github-anchor-fidelity.md#df-github-anchor-fidelity-the-github-anchor-profile-reproduces-github-slugger-exactly) exists because grund's own specification once got wrong. Writing one issue comment that cited about forty points took a dedicated helper script and a second `show` call per citation, all to rebuild a fragment `grund fmt --cross-refs` already derives. An agent without such a helper either drops the links its instructions ask for or guesses, and a guessed fragment is a silent dead link, which the recipe's own fallback ladder forbids. Reported as [issue #497](https://github.com/agent-grounds/grund/issues/497).

## 2. Decision

### 2.1 The anchor is a field of the read, not a link

Every successful `show --format=json` object carries `anchor`: the fragment, without its leading `#`, that `grund fmt --cross-refs` derives for the selected coordinate, and each `--toc` section entry carries its own ([§FS-show.3.1.3.1](../../functional-spec/FS-show.md#3131-the-heading-anchor), [§FS-output-shapes.4](../../functional-spec/FS-output-shapes.md#4-show---formatjson)). It sits after `kind_title` and before `path`, because `path`, `line` must stay the closing pair installed `grund-open` copies read. A successful batch result is the same object ([§FS-output-shapes.4.1](../../functional-spec/FS-output-shapes.md#41-show---batch---formatjson)).

This takes the data-surface fallback and nothing more. `show` renders no URL, ships no `--link` flag, and no `grund link` command appears: [§DF-neural-link-generation.1](DF-neural-link-generation.md#1-decision) stands, `grund fmt --cross-refs` stays the only link emitter, and the caller still chooses the base and ref and joins `<base>/blob/<ref>/<path>#<anchor>` itself.

### 2.2 One derivation, the formatter's

The value is the formatter's fragment for the same heading, under the `anchor_format` profile of the project that declares the target, so it cannot drift from the links `fmt` writes into the repository. Profile authority stays with [§DF-md-link-anchor-strategy.2.3](DF-md-link-anchor-strategy.md#23-renderer-profiles) and its recorded `github` correction; this decision adds no slugger and changes none.

### 2.3 `null` means there is no heading anchor

`anchor` is always present. It is `null` under the `none` profile, for a declaration whose home is a source file (read directly or through a stub), for a JSON value declaration, for a `--toc` entry inside a source declaration, and for an E2E manifest ([§FS-show.2.4.2](../../functional-spec/FS-show.md#242-the-manifest-as-json)). One presence rule is cheaper to read than a key that is sometimes missing, and a caller that sees `null` links the `line` as `#L<line>`, as the recipe already said for source homes. `[fmt.cross_refs] enabled = false` and the formatter's exclusions decide what `fmt` writes, not what a heading's anchor is, so they leave `anchor` set.

## 3. Alternatives considered

**Keep hand-slugging.** The status quo, and what this decision ends: it costs a second read per link and fails silently when the slug is guessed wrong.

**A `--link <base>` flag or a `grund link` command.** Removes the most work from the caller, but reverses [§DF-neural-link-generation.1](DF-neural-link-generation.md#1-decision) and adds permanent surface for a presentation concern. The field gets nearly all of the benefit.

**Omit the key where there is no anchor.** Rejected for 2.3's reason: one shape with a `null` is easier to decode than two shapes.

## 4. What this costs, and why it may be taken

The JSON bytes of every successful `show --format=json` read change, and so do whole-output snapshots and strict decoders that reject unknown keys, against [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) and [§GOAL-no-silent-breakage.1](../../goals.md#1-what-counts-as-user-visible). No verdict moves and no field changes meaning: a reader that ignores unknown keys, and the installed location-tail and E2E-prefix readers, keep working. Carrying the old shape beside the new one would mean a second JSON schema and a mode to select it, for a single additive key, so this uses the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) and discloses the change in the release note below.

## release-note: Release note

- [§FS-show.3.1.3.1](../../functional-spec/FS-show.md#3131-the-heading-anchor), [§FS-output-shapes.4](../../functional-spec/FS-output-shapes.md#4-show---formatjson), [§DF-show-anchor-data](DF-show-anchor-data.md#df-show-anchor-data-show-json-carries-the-heading-anchor-grund-already-derives): every successful `show --format=json` object, single or `--batch`, now carries `anchor`, the heading fragment `grund fmt --cross-refs` derives for it, between `kind_title` and the closing `path`, `line` pair, and each `--toc` section entry carries its own after `depth`. It is `null` where there is no heading anchor: the `none` profile, source homes, JSON values and E2E manifests. Readers that ignore unknown keys are unaffected; a strict decoder or a whole-output snapshot must accept the new key. Closes [issue #497](https://github.com/agent-grounds/grund/issues/497). (PR #500)
