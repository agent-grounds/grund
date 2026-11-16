# DF-defined-language-string-forms: a file read through a source definition reads the multi-line strings and quotes its own suffix names

**Date:** 2026-11-16

## status: Accepted

## 1. Context

The multi-line string table of [§FS-languages.multi-line-string.forms](../../functional-spec/FS-languages.md#multi-line-stringforms-the-recognized-forms) keys each form by a file
extension: Rust raw strings in `rs`, Java text blocks in `java`, heredocs in `sh`, and so on. Two
suffixes also change how a `'` is read: `rs`, where a lifetime is no quote, and `hs`/`lhs`,
where a prime opens nothing ([§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)). Version 1 asks every file by
its extension. Version 2 asked by the file's language: a built-in read its own row and quote
reading, and a `[schema.sources.definitions]` language read none. No point of the specification
said so. [§FS-languages.multi-line-string](../../functional-spec/FS-languages.md#multi-line-string-a-string-literal-that-spans-lines) said a form is carried "in a file whose extension the
form names", and only the code's doc comment said a definition reads none.

Migration writes every version-1 suffix as a definition: `v1_c_family`, `v1_python`, `v1_go`,
`v1_shell` and the rest ([§FS-config-v2.schema.sources.migration](../../functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). So every migrated file lost
its row and its quote reading, and the records gate could not see it, because these were not among
the columns it compares ([§FS-config-migrate.5.1](../../functional-spec/FS-config-migrate.md#51-the-records-gate)). On a tree with no discriminating literal yet,
`config migrate --to 2` said `compared: the records are equal` and wrote the file. From then on,
the inside of every Java text block, Rust raw string and Go raw string was read as comment text,
and a citation after a `'static` was hidden ([issue #1406](https://github.com/agent-grounds/grund/issues/1406)). This repository's own
`grund.toml` is such a migrated file.

## 2. Decision

### 2.1 A source definition's files read their own suffix

A file read through a `source` definition reads the row and quote reading its own final
extension names, as a version-1 file does. A file read through a built-in reads that language's
own, over a mapped suffix too, because a mapping takes a built-in whole
([§FS-config-v2.schema.sources.extensions](../../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). Markdown files, and files read through a `document` or
`data` definition, read none. [§FS-languages.multi-line-string.suffix](../../functional-spec/FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads) is the one point that says
this, and both the scan and the records gate ask the one function that answers it.

That is what [§FS-languages.multi-line-string](../../functional-spec/FS-languages.md#multi-line-string-a-string-literal-that-spans-lines) already said for a definition's files, so the code
comes into line with the specification. It also gives every project that an earlier release
migrated its version-1 reading back without an edit. Migration writes the same file byte for
byte; what changes is what that file means.

### 2.2 The records gate compares the reading

Each suffix the records gate compares gains one column: the multi-line string and quote reading
a file of that suffix reads, worked out on both sides by the function the scan uses. It holds the
reading, not the suffix. So `.cc` under version 1 and `.cc` mapped to the built-in `cpp` compare
equal, because both read the one C++ row. `.h` under version 1, which reads C++ raw strings, and
`.h` mapped to the built-in `c`, which reads none, compare different. The quote reading is part
of the column because no row names `hs` or `lhs`. A column that held only the row would compare
a Haskell file read with primes and one read without them as equal.

For what migration writes, the column is always equal, because each side resolves the same
suffix. If it ever differs, the difference is a defect in migration rather than in the version-1
file, and it is shown on the `sources` line of `differs: the records` ([§FS-config-migrate.5.4](../../functional-spec/FS-config-migrate.md#54-the-result-and-a-difference)).

### 2.3 A definition cannot opt out of its suffix's row

A definition has no key for a row or a quote reading. So a hand-written `source` definition over
a suffix the table names, such as `java = "myjava"`, reads that suffix's row whatever its author
meant it for, and cannot opt out. That is acceptable for two reasons.
[§FS-languages.multi-line-string.bound](../../functional-spec/FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file) bounds what a misread opener can cost: an opener counts
only in code and only once its closer is found, so it can never start a span that hides a
citation. And version 1 had no opt-out either: a version-1 `.java` file has always read text
blocks. The one way to read such a suffix without its row is to map it to a built-in that has
none, which takes that built-in whole.

## 3. Alternatives considered

- **A forms key on definitions.** A definition holds one value, so migration would have to split
  `v1_c_family` into one definition per row: Rust, Java, C#, C++, JavaScript and TypeScript, and
  PHP. Worse, every project an earlier release migrated, this repository included, would keep
  the defect until someone edits its config.
- **Reusing an equal built-in per suffix.** This is part 1 of
  [issue #1384](https://github.com/agent-grounds/grund/issues/1384). It applies only where version 1's
  whole prefix list equals a built-in's, and version 1's default `["//", "#", ";", "--", "*",
  "/*"]` equals none of them. So it reaches neither the reported fixture nor this repository,
  and it leaves migrated projects as they are. It can still land on top of this decision: once
  the gate compares the reading, it can only pick a built-in the gate calls equal, which rules out
  `c` for `.h`.
- **A written exception in the records gate.** An exception written into
  [§FS-config-migrate.5.1](../../functional-spec/FS-config-migrate.md#51-the-records-gate) would have the gate call two different readings equal by decree. [§DF-config-migrate.2.1](DF-config-migrate.md#21-a-refusal-names-a-v1-repair-not-a-flag) rules that out: migration never changes a meaning.
- **Keying built-ins by suffix too, for one rule everywhere.** That would change `txt = "rust"`,
  which would lose raw strings, and `h = "c"`, which would gain C++ raw strings. Both follow the
  "whole language" rule of [§FS-config-v2.schema.sources.extensions](../../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language), which stays.

## 4. Compatibility

Only version-2 projects with a `source` definition mapped from a suffix that names a row or a
quote reading see a difference. That is every project `config migrate --to 2` wrote, and the
rare project that hand-wrote such a definition, since `grund init` writes built-ins. Version-1
projects and version-2 projects that read only built-ins are unchanged.

Verdicts move in both directions.

- **Fewer findings.** A qualified citation inside such a literal is no longer reported as
  `unknown project alias` or `unknown reference`, and `grund fmt --check` no longer offers to
  rewrite there. A false finding on runtime text is the false alarm [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)
  forbids, so this direction needs no licence.
- **New findings.** A dangling qualified citation in a comment was hidden when it came after a
  Rust lifetime or character literal, a Haskell prime, or a literal's closer. It is now reported
  at its line.

The second moves a verdict from passing to failing, so it uses [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids),
as [§DF-multi-line-string-literals.2.5](DF-multi-line-string-literals.md#25-the-closing-line-finding-is-a-correction-under-req-backwards-compatibility5) did for the same kind of finding. Its five conditions hold:

1. **Prior prohibition.** The old verdict was a blind spot that no section declared:
   [§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) reads a qualified citation after `'static` or `f'` in a
   source comment, and [§FS-languages.multi-line-string](../../functional-spec/FS-languages.md#multi-line-string-a-string-literal-that-spans-lines) reads a form in a file whose extension
   names it. Neither excepts a definition's files. [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) forbids such a
   blind spot, and it already applied when version-2 definitions shipped.
2. **Accepted proof.** This record. The deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit:
   a warning first would keep printing `success` for a whole release over a citation that
   points at nothing, which is the false negative the prohibition names. The mechanical
   migration of [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either: the repair is to correct or
   remove a citation, and only its author knows which.
3. **Named release.** The release note below names the change.
4. **Actionable findings.** Each new finding is the existing `unknown project alias` or
   `unknown reference`, at the citation's own line, with the action it always had: correct the
   citation or remove it.
5. **No new licence.** The prohibition is not invented here, and no rule gets stricter: a
   citation the specification already said was read is now read. This route is not a licence
   for any other verdict change.

## release-note: Release note

- In grund 0.23.0, a file read through a `source` definition in `[schema.sources.definitions]`
  reads the multi-line string literals and quotes its own suffix names, as version 1 and the
  built-ins do: Rust raw strings and lifetimes, Java, Kotlin, Scala, Swift and Dart
  triple-quoted strings, C# and C++ raw strings, Go raw strings, JavaScript and TypeScript
  template literals, Python triple-quoted strings, shell and PHP heredocs, and Haskell primes.
  In grund 0.22 it read none. Every project `grund config migrate --to 2` wrote returns to its
  version-1 reading without an edit. A hand-written `source` definition over such a suffix now
  reads that suffix's row and cannot opt out of it, as a version-1 file never could:
  [§FS-languages.multi-line-string.bound](../../functional-spec/FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file) keeps a misread opener from hiding a citation.
  **Fewer findings:** a qualified citation inside such a literal is no longer reported as
  `unknown project alias` or `unknown reference`, and `grund fmt --check` no longer offers to
  rewrite there. **Some `check` verdicts change from `0` to `1`:** a dangling qualified
  citation in a comment after a Rust lifetime or character literal, a Haskell prime, or a
  literal's closer was hidden and is now reported at its line; correct or remove the citation.
  Every finding that remains or appears keeps its location and its action.
  `grund config migrate --to 2` now compares each suffix's multi-line string and quote reading
  in its records gate and shows a difference on the `sources` line. This correction uses
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) because the prior verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)
  ([§DF-defined-language-string-forms](DF-defined-language-string-forms.md#df-defined-language-string-forms-a-file-read-through-a-source-definition-reads-the-multi-line-strings-and-quotes-its-own-suffix-names)).
