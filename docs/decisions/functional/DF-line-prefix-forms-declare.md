# DF-line-prefix-forms-declare: Lua, Haskell and Lisp doc-comment forms declare, and a Lisp doc block is not a note

**Status:** Accepted
**Date:** 2026-10-13

## 1. Context

Each language below documents a definition in a comment form of its own, and grund read
none of them as a declaration host. Reproduced on `main` at 40d7a50887
([#703](https://github.com/agent-grounds/grund/issues/703)), with `FS-entry` citing each ID:

```text
src/lua/router.lua      --- AR-lua: Lua router
src/haskell/Router.hs   -- | AR-haskell: Haskell router
                          -- ^ AR-haskell-caret: the request path
src/elisp/router.el     ;; AR-elisp: elisp router
                        ;; Routes requests, per FS-entry.1.   (a seven-line block above a defun)
```

`grund list` lists none of them and `grund check` reports `unknown reference` on every
citation of them, while `-- AR-sql:` in SQL and `/// AR-rust:` in Rust, in the same run,
declare and resolve. The `;;` blocks in `.clj`, `.el` and `.lisp` also fail as inline
notes over the three-line cap.

A source declaration is found behind a configured prefix followed by whitespace. `--` and
`;` are default prefixes, but in `--- AR-lua`, `-- | AR-haskell` and `;; AR-elisp` the
character after the prefix is a third `-`, a `|` or a second `;`. `//` was widened long ago
to `///` and `//!`, and `/*` to `/**` and `/*!` in 0.17
([§DF-bare-block-comment-line-declares](DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers)); `--` and `;` never were. Meanwhile
[§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages) already called the Lua and Haddock forms doc comments,
and gave Lisp no row at all, so every `;;` block was a note by specification.

## 2. Decision

### 2.1 A prefix stands for its doc-comment forms

`--` stands for `---`, `-- |`, `--|`, `-- ^` and `--^`, and `;` for any run of semicolons
([§FS-languages.prefixes.forms](../../functional-spec/FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)). A line behind any of them is in declaration position
([§FS-declarations.line](../../functional-spec/FS-declarations.md#line-what-a-line-in-declaration-position-declares)), a `## 1. Sub` behind one is a section heading where the bare
prefix's would be, and the wrapper `show` removes is the whole form
([§FS-languages.comment-wrapper](../../functional-spec/FS-languages.md#comment-wrapper-the-comment-wrapper)). `----` stays a rule line. The forms are per prefix, not
per language, as `///` already is.

### 2.2 Lisp is a position language

Lisp, Scheme, Clojure and Emacs Lisp write `;;` above a definition and inside a body alike,
so the run cannot say "documentation". Position can, as for Go, Ruby, shell and SQL: a `;`
block is a doc comment when the line under it opens a form whose head starts with `def` or
`cl-def`, or is `ns`, or when it is the file's leading comment
([§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages)). A `;;` note inside a body is still measured.

### 2.3 Haskell's block comment is not a host

`{-| … -}` declares nothing ([§FS-languages.prefixes.haskell-block](../../functional-spec/FS-languages.md#prefixeshaskell-block-a-haskell-block-comment-hosts-no-declaration)). The line form
`-- |` documents everything it does and now declares, and a citation inside the block is
already read and checked, so nothing written there is missed.

### 2.4 Every Lisp extension is read by default

The Lisp row names `lisp cl el scm ss rkt clj cljs cljc`. `lisp`, `scm` and `clj` were
already in the `[scan] extensions` default; `cl`, `el`, `ss`, `rkt`, `cljs` and `cljc` join
it ([§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned)), appended after the entries already there, so every extension a
recognizer classifies is read by a project that wrote no `extensions`, and each has its
dangling-citation proof ([§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)). `.cl` is OpenCL's extension too,
which is harmless: only lines led by a configured prefix are read.

## grounding: A Lisp doc block that cites nothing warns first

Under `grounding_level` 2 or higher a doc-comment block is a grounding unit
([§FS-check.3.6.2.2](../../functional-spec/FS-check.md#3622-in-a-source-file)), so the Lisp row makes units that did not exist, and one that
cites and declares nothing would fail a tree that passed. That is a new policy reaching
existing code rather than a false verdict corrected, so it takes the deprecation path of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): through 0.18.x an ungrounded Lisp doc-comment is a
`warning:` whose message ends `this warning becomes an error in grund 0.19.0`, and from
0.19.0 it is the error every other ungrounded doc-comment is.

## 3. Rejected alternatives

- **A doc-comment row keyed on `;;`**, prefix-style like Lua's `---`. It would stop
  measuring the `;;` notes inside function bodies too, the one place the cap is for.
- **`{- … -}` as a second block-comment pair.** The one block-comment reader is C's
  `/* … */`; a nesting pair would generalize it and add a default prefix that changes every
  `grund init` output, for a form whose line twin works.
- **Widening per language rather than per prefix.** `//` is not, and two rules for one
  question would let the declaration gate and the note stripper disagree.
- **A warning release before the declarations count.** It keeps a false
  `unknown reference` alive for a release, which [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids.

## 4. Consequences

- **Fail to pass.** A citation of a declaration written behind one of the forms resolves,
  and `show`, `list`, `refs`, `cover` and the editor find it. A Lisp doc block above a
  definition or at the top of a file is no longer measured as a note.
- **Pass to fail.** A newly found declaration is judged like any other: a duplicate (most
  likely a workaround copy in Markdown), a misplaced declaration, a missing index entry
  (`grund fmt --write` adds it), a `declaration-near-miss` for `--- FS-Bad:` or
  `;; FS-Bad:` as `-- FS-Bad:` already gets. A file under `[scan] include` with one of the
  six new extensions now has its citations checked. A declaration found today behind a
  single `;` or `--` keeps today's reading of its body until 0.19.0
  ([§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)).
- **Untouched.** A project that removed `--` or `;` from `comment_prefixes`, or wrote its
  own `extensions`, sees no change from the forms or the default. Markdown, docstrings,
  `//`, `/* */` and `#` read as before.

## 5. The correction route

The declarations and the Lisp row move verdicts under
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), and each of its five conditions holds.

1. *Prior prohibition.* A citation of a declaration its author wrote was reported as an
   `unknown reference`, which [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids; the default prefixes
   have carried `--` and `;` since 0.1.0.
2. *Accepted proof.* This record. Deprecation keeps the false error alive a release longer,
   and there is nothing to migrate: the comments are right as written.
3. *Named release.* The release note below.
4. *Actionable findings.* Every finding that replaces the old verdict is one the bare
   prefix's form already gets, at the declaring line, naming the edit.
5. *No new licence.* No rule or finding is new. The six new default extensions take
   [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise): those files had no scan meaning, as `lisp`, `scm` and `clj`
   had none before 0.10.1. The new grounding units take the warning of
   [§DF-line-prefix-forms-declare.grounding](DF-line-prefix-forms-declare.md#grounding-a-lisp-doc-block-that-cites-nothing-warns-first).

## release-note: Release note

- [§DF-line-prefix-forms-declare](DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note), [§FS-languages.prefixes.forms](../../functional-spec/FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for),
  [§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): **Lua `---`, Haskell
  `-- |` and `-- ^`, and Lisp `;;` and `;;;` comments now host declarations, and a Lisp
  doc block is no longer measured as a note.** Their declarations appear in `list`,
  `refs` and `show`, and the false `unknown reference` on every citation of them goes.
  A `;;` block directly above a `def…`, `cl-def…` or `ns` form, or at the top of the file,
  is documentation; one inside a body is still a note. Haskell's `{-| … -}` hosts no
  declaration. The `[scan] extensions` default gains `cl`, `el`, `ss`, `rkt`, `cljs` and
  `cljc`. **Who this breaks:** a repository that declared such an ID a second time
  elsewhere to work around this (now a duplicate at both sites), one with `--- FS-Bad:` or
  `;; FS-Bad:` lines (now a `declaration-near-miss`), and a project with no `extensions`
  key that keeps a stale `§` citation in a file of one of the six new extensions (now an
  `unknown reference`). Under `grounding_level` 2 or higher a Lisp doc block that cites
  nothing is a warning in this release and an error from grund 0.19.0. Closes
  [#703](https://github.com/agent-grounds/grund/issues/703).
