# DF-declarations-in-doc-comments: a v2 source declares in documentation, not in a note

**Status:** Accepted
**Date:** 2026-10-06

## 1. Context

Every comment in declaration position declares an ID in v1 ([§FS-declarations.line](../../functional-spec/FS-declarations.md#line-what-a-line-in-declaration-position-declares)). So a note inside a function that opens with a root ID and a colon is a second declaration of that ID. Reproduced on `main` at 5fd1400ce9 with `FS` at the folder `docs`:

```rust
/// FS-existing: Real documentation
pub fn example() {
    // FS-existing: Ordinary root-ID note
}
```

`grund list --format json` lists `FS-existing` twice, both `duplicate: true`, and `grund check` reports `duplicate declaration of FS-existing`. The section-suffix guard of [§FS-declarations.line.section-suffix](../../functional-spec/FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing) already stops `// FS-existing.3 / …` from declaring, which was the roadmap's original example, but a bare root ID still declares.

[§DF-doc-comments-are-not-notes](DF-doc-comments-are-not-notes.md#df-doc-comments-are-not-notes-a-doc-comment-is-documentation-not-a-note-and-is-never-an-inline-citation-site) built the classifier that tells a doc comment from a note, for measuring notes, and [§AR-scanner.4.2.3](../../architecture/AR-scanner.md#423-the-declaration-gate-reuses-it) reserved it for this gate. A v1 config's meaning cannot change ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)), so the gate needs a version that spells it. agent-grounds/grund#456 fused the gate into v2's source model, because the language that picks a file's comment hosts must also pick its doc classifier. Its proposal was approved by the repository owner on 2026-10-06.

## 2. Decision

### 2.1 A v2 key with a v2 default, and no v1 key

`[schema.sources] declarations` takes `"doc-comments"` or `"any-comment"`, and v2's default, for new and migrated projects alike, is `"doc-comments"` ([§FS-config-v2.schema.sources.declarations](../../functional-spec/FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare)). `"any-comment"` is v1's recognition, and it is what a migration writes ([§FS-config-v2.schema.sources.migration](../../functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). v1 gains no key. The roadmap's `[scan] declarations_in_doc_comments`, default on, would have changed every v1 file's meaning by default, so it is not adopted.

### 2.2 Applicability is the classifier's, frozen per profile

A block is eligible when its language's doc profile classifies it as documentation, or, for a language whose profile is `none`, when it is any comment behind the language's hosts ([§FS-config-v2.schema.sources.declarations.eligible](../../functional-spec/FS-config-v2.md#schemasourcesdeclarationseligible-which-blocks-are-eligible)). The profile's corners become stated rules for declarations: a dangling dedicated doc marker qualifies, a Go local `var` qualifies, a Ruby `private def` does not, and blank lines and lookalike markers break eligibility ([§FS-inline-citation-style.1.1.4](../../functional-spec/FS-inline-citation-style.md#114-declaration-applicability)). A module doc such as Rust `//!`, and a position language's leading block, qualify, so the roadmap's *immediately following definition* does not discard file documentation.

These limits are fixed at the v2 epoch. A widening that was harmless for measurement, where a miss only decides whether a note is measured, would change which IDs a tree declares, so a profile that needs to move is a new profile or a new version.

### 2.3 One gate, before every recognizer

Eligibility is decided before canonical, near-miss and legacy fallback recognition, and with or without a citation in the block ([§FS-config-v2.schema.sources.declarations.gate](../../functional-spec/FS-config-v2.md#schemasourcesdeclarationsgate-one-gate-before-every-recognizer)). Bodies, anchors, stubs, the formatter and the editor read the same answer. An ineligible line therefore cannot come back as an off-grammar declaration or a near miss.

### 2.4 No new diagnostic

A declaration-shaped line in an ineligible block is prose and draws nothing ([§FS-config-v2.schema.sources.declarations.prose](../../functional-spec/FS-config-v2.md#schemasourcesdeclarationsprose-an-ineligible-declaration-shaped-line-draws-no-new-finding)). A warning would fire on every note that names the point it grounds, which is the note the inline style asks for. A marked citation on that line is still a citation.

## 3. Alternatives rejected

- **An AST parser per host language.** Exact attachment would need a parser per language, and [§FS-non-goals.3](../../functional-spec/FS-non-goals.md#3-code-ast-parsing) excludes that. The bounded recognizer is enough, because a miss is visible: a declaration that vanishes is a dangling citation.
- **A doc-only gate in v1, default on.** It changes every existing tree's catalog on upgrade.
- **Making declarations impossible in a language with no doc notation.** Lisp, Ada and prefix-only definitions would lose every inline declaration. Their configured comments declare instead.

## 4. Consequences

A v2 tree has no phantom duplicates from notes. A v2 tree that declared in a plain comment moves that declaration into a doc comment, or writes `declarations = "any-comment"`. v1 trees see no change.

## release-note: Release note

- [§FS-config-v2.schema.sources.declarations](../../functional-spec/FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare), [§FS-config-v2.schema.sources](../../functional-spec/FS-config-v2.md#schemasources-schemasources): **a v2 project declares source IDs only in doc comments by default.** A `// FS-x: note` inside a function no longer declares `FS-x` in a `grund_config_version = 2` project, so it is no longer reported as a duplicate. Its marked citations still resolve. Write `declarations = "any-comment"` under `[schema.sources]` to keep v1's recognition. Version-1 configurations are unchanged.
