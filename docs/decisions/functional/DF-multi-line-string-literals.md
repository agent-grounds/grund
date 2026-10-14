# DF-multi-line-string-literals: a string literal that spans lines is one literal, for a fixed set of forms

**Status:** Accepted
**Date:** 2026-10-13

## 1. Context

A qualified citation inside a source-file string literal is data shaped like a path, so it is skipped ([§FS-citations.recognized.string-literals](../../functional-spec/FS-citations.md#recognizedstring-literals-string-literals-in-source-files)). Whether a column is inside a string was decided one line at a time: the quote toggle of [§FS-fmt.2.3.1](../../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule) started every line neutral. So the skip reached the first line of a Rust raw string or a Java text block and none of the lines after it. A help text written as

```rust
r#"
Usage: see api/FS-gone
"#
```

with the marker before `api/FS-gone` failed `grund check` with `unknown reference`, while the same text on one line passed, and `grund fmt` offered to rewrite runtime text it does not own. Issue #702 reported it.

The closing line was wrong in the other direction. The toggle read the closer's `"` as an opening quote, so a dangling citation in a comment after `"#);` or `""";` was hidden and the tree passed.

## 2. Decision

### 2.1 What carries is a form, never the toggle

Carrying the per-line toggle across lines would fix the interior and break everything else: one apostrophe in `// don't`, a character literal or a Rust lifetime would open a string that hides every later qualified citation in the file, which [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) forbids. So the toggle stays as it is, and what carries is a fixed table of forms, each tied to file extensions and each with an exact opener and an exact closer ([§FS-languages.multi-line-string.forms](../../functional-spec/FS-languages.md#multi-line-stringforms-the-recognized-forms)). A form changes only the lines that begin inside it; the opening line and every other line read as before.

The table is a bounded lexical distinction, like the assigned-data rule of [§DF-python-assigned-triple-quoted-data](DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings), and not a lexer: [§FS-non-goals.3](../../functional-spec/FS-non-goals.md#3-code-ast-parsing) rules out parsing the host language. A JavaScript template literal tracks its `${…}` interpolations with a lexical stack, because without it a template nested in an interpolation would close the outer one at its first backtick.

### 2.2 Two bounds, so a misread opener costs nothing

An opener counts only in code, and a form counts only once its closer is found later in the file ([§FS-languages.multi-line-string.bound](../../functional-spec/FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file)). The second is the rule a `/* … */` block already follows. Together they mean a wrong guess about an opener can only leave a file as it reads today; it can never start a span that hides a citation. Holding that took three readings closer to the language than the per-line toggle: in Rust a character literal is one unit and a lifetime is no quote, so `&'static str = "it's"` opens nothing; a `/* … */` is skipped to its `*/` wherever on its line it opens; and in JavaScript and TypeScript a regular-expression literal is skipped, so `` /`/g `` opens no template. A string opened after a lifetime on the same line is therefore carried like any other. The character literal is Rust's own reading, but the regular expression is a guess: a `/` may be a division or the `</` of a JSX tag. Where a guessed literal holds a quote or a comment opener, the rest of its line is read both past it and through it. Ending the search on that line instead is not enough: an opener in the rest of the line goes unread, and where its closer also reads as an opener, as a `` ` `` or a `"` does, the later line it closes on opens a form that hides the code up to the next one. So the search reads every line to its end. Where it cannot, because the two readings of a guess open different things past the line or a `/*` never closes, it ends, and the rest of the file reads as it does today.

### 2.3 The forms left out are declared, and fail toward reporting

Ordinary strings that span lines where `'` doubles as an apostrophe, Ruby heredocs, Lua long strings and backslash continuations are not carried ([§FS-languages.multi-line-string.unrecognized](../../functional-spec/FS-languages.md#multi-line-stringunrecognized-the-forms-that-are-not-carried)). On their interior lines a qualified citation can still raise a false `unknown reference`. That is the blind spot [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) requires be written down, and it errs toward reporting rather than toward silence.

### 2.4 One reading for every caller

`check`, `refs`, `cover`, `unused`, every `grund fmt` pass, the on-type transform and completion all ask one question — is this column inside a string literal — and all of them get the line's entry state from the same place. A second answer for one of them is how `check` would demand an edit that `fmt` refuses to make.

### 2.5 The closing-line finding is a correction under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)

One verdict moves toward failure: a dangling qualified citation after a multi-line literal's closer on the same line is now reported. The five conditions of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) hold:

1. **Prior prohibition:** the old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which forbids a silent skip, and [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), which forbids a blind spot no section names; the hidden tail was both, and both requirements already applied when the line-by-line reading shipped.
2. **Accepted proof.** This record. The deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit: a warning first would keep printing `success` over a citation that points at nothing for a whole release, which is the false negative the prohibition names. The mechanical migration of [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either: the fix is to correct or remove a citation, and only the author knows which.
3. **Named release.** The release note below names the change.
4. **Actionable findings.** The new finding is the existing `unknown reference`, at the citation's own line, and its action is the one it always had: correct the citation or remove it.
5. **No new licence.** The prohibition is not invented here, and the change is not a policy tightening: no rule gets stricter, a citation the spec already said was read is now read. This route is not a licence for any other verdict change.

The verdicts that move toward success need no licence of their own, since a false `unknown reference` on runtime text is the false alarm [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids. They are listed in the release note all the same.

## 3. Alternatives considered

**Carry the per-line toggle across lines.** Rejected on [§DF-multi-line-string-literals.2.1](DF-multi-line-string-literals.md#21-what-carries-is-a-form-never-the-toggle): it hides citations after any apostrophe.

**Write the gap down as a blind spot and change nothing.** Rejected: the false error is fixable for the forms that cause it in practice, and [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) weighs a false alarm like a false negative.

**End the search on a doubtful line and resume on the next.** Rejected on [§DF-multi-line-string-literals.2.2](DF-multi-line-string-literals.md#22-two-bounds-so-a-misread-opener-costs-nothing): the opener left unread in the rest of the line turns its closer into an opener.

**Read the lines up to that unread opener's closer as today, then resume.** Rejected: it resumes in step only if the unread opener was real. Where the guess was wrong, the closer it stops at is a real opener, and every form after it is read out of step.

**A lexer per language.** Rejected on [§FS-non-goals.3](../../functional-spec/FS-non-goals.md#3-code-ast-parsing).

## 4. Consequences

A qualified citation, and under `strict = false` a bare ID, on the interior line of a recognized form is no longer a citation, so `refs`, `cover` and `unused` stop counting it, as they already did on one line. A declaration whose only use was such a bare ID becomes unused. There is no new configuration key and nothing to migrate.

## release-note: Release note

- [§DF-multi-line-string-literals](DF-multi-line-string-literals.md#df-multi-line-string-literals-a-string-literal-that-spans-lines-is-one-literal-for-a-fixed-set-of-forms), [§FS-languages.multi-line-string](../../functional-spec/FS-languages.md#multi-line-string-a-string-literal-that-spans-lines), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **a string literal that spans lines is one literal, for the forms the spec lists.**
  Rust raw and ordinary strings, Java, Kotlin, Scala, Swift and Dart triple-quoted strings, C# raw and verbatim strings, C++ raw strings, Go raw strings, JavaScript and TypeScript template literals, Python triple-quoted strings that are neither docstrings nor assigned data, shell heredocs and PHP heredocs and nowdocs.
  **Fewer findings:** a qualified citation on an interior line of one of these, and under `strict = false` a bare ID there, is no longer reported as an `unknown reference`, and `grund fmt --check` no longer asks to rewrite a trigger, a shorthand or a bare ID there.
  **The verdict changes** in both directions, and every finding that remains or appears keeps its location and its action.
  **A new finding:** a dangling citation after the closer on a literal's closing line, such as `"#); // <§>api/FS-gone`, was hidden and is now reported at its line. Correct or remove the citation.
  **A use that disappears:** under `strict = false`, a bare ID inside such a literal that was a declaration's only use no longer counts, so `unused` may name that declaration.
  Forms that are not carried are listed in [§FS-languages.multi-line-string.unrecognized](../../functional-spec/FS-languages.md#multi-line-stringunrecognized-the-forms-that-are-not-carried) and read as before, and so does the rest of a file after a line [§FS-languages.multi-line-string.bound](../../functional-spec/FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file) cannot read to its end, such as one with a `/*` that never closes.
