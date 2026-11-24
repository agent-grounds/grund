# DF-section-after-nested-declaration: a section heading after a nested declaration is the enclosing declaration's coordinate

**Date:** 2026-11-14

## status: Accepted

## 1. Context

In Markdown a declaration heading deeper than an enclosing one opens a nested body. That
body ends at the next heading at its own level, and the enclosing body goes on
([§FS-declarations.body](../../functional-spec/FS-declarations.md#body-where-a-declarations-body-ends)). The scan offered each section heading only to the declaration it
had read last. After `## AR-matrix` under `# AR-ci`, it offered a later `## 1. Pull-request
workflows` to `AR-matrix`. `AR-matrix` refused it, because the heading is not deeper than
its own heading, so the heading went into no section map. Every other reader put the heading
inside `AR-ci`: `show --full` printed it there, [§FS-declarations.checks.unmarked-heading](../../functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)
told the author to number it as `AR-ci.1`, and [§FS-declarations.checks.section-outside-declaration](../../functional-spec/FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)
found it inside a body and said nothing. After the author numbered it, `check` was silent,
but `grund AR-ci.1` said `section not found` and a citation of it failed with
`missing section` ([issue #1359](https://github.com/agent-grounds/grund/issues/1359)).

## 2. Decision

The heading is a coordinate of the innermost declaration whose body holds it
([§FS-declarations.section.map.nested](../../functional-spec/FS-declarations.md#sectionmapnested-after-a-nested-declaration-the-enclosing-body-owns-the-heading)). This holds for numbered, enabled named and relative
headings alike, at any depth of nesting. That is what [§FS-declarations.section.map](../../functional-spec/FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span) already
said, so the fix brings the section map into line with it. Making `check` refuse the
heading, or stop suggesting `AR-ci.1`, would refuse input the specification accepts.

## 3. Compatibility

Three verdicts move, all in a body that holds a nested declaration, and only for a numbered,
named or relative heading after that nested body has ended.

- A citation of such a heading is reported as `missing section` today and checks clean
  after the fix. A relative heading under such a heading is a
  [§FS-declarations.checks.relative-section-parent](../../functional-spec/FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) error today and a section after the fix.
  These verdicts get looser, so [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) owes them nothing.
- The suggestion in an unmarked-heading error can name a different coordinate: the next
  sibling that is unused once the heading after the nested declaration counts. That is the
  rule [§FS-declarations.checks.unmarked-heading](../../functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) already states ("it never fills a hole or
  reuses a coordinate"), applied to a heading it used to miss. The finding, its code and its
  phrasing do not change.
- A tree that passes today can fail. The heading is now a claimant in its body, so it earns
  every finding a section there earns. Two headings that claim one path across the nested
  declaration are a [§FS-declarations.checks.duplicate-section](../../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path), and a depth that does not
  match the path is a [§FS-declarations.checks.section-heading-level](../../functional-spec/FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) finding.

The last change uses [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), the correction route.

The prior verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), in the two ways the next two sentences give.
That prohibition already applied when the old verdict shipped, because [§FS-declarations.section.map](../../functional-spec/FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span) already made the map the body span.
Where two headings in the body claim one path, `check` and `show` silently chose the first, and [§FS-declarations.checks.duplicate-section](../../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path) calls that choice the guess [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids by name.
And a heading inside the body was missing from the map, so a citation of a coordinate the body declares was reported broken, which [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids.
The new findings come from rules that already applied to every section in the body, so the change makes up no new rule.

[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a warning window would have to do one of two things.
It could leave the heading out of the map, which keeps both faults above. Or it could put the heading in the map and hide the findings it earns, which gives `check` and `show` two section maps, which [§REQ-one-answer.sections.map](../../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map) forbids.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because no command can decide which of two claimants to renumber.
[§FS-declarations.checks.duplicate-section](../../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path) has no ranking, and no command numbers a heading ([§FS-declarations.checks.unmarked-heading](../../functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).
Each new finding is located at the heading and names its repair.
This is not a licence for any other change to how sections are recognised.

## release-note: Release note

- In grund 0.23.0, a numbered, named or relative section heading that follows a nested
  declaration in Markdown, inside the enclosing declaration's body, is a section of the
  enclosing declaration; in grund 0.22 it resolved nowhere. A citation of it that failed
  with `missing section` now checks clean, and `show` and `--toc` print it. Some `check`
  verdicts change from `0` to `1`: the heading now earns the findings any section in that
  body earns, `duplicate-section` where it repeats a path claimed before the nested
  declaration and `section-heading-level` where its depth does not match its path. Every finding names its location and an action: renumber or rename the heading, or fix its depth. An unmarked heading after it is suggested the next unused
  coordinate. This correction uses [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) because the prior verdict
  violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)
  ([§DF-section-after-nested-declaration](DF-section-after-nested-declaration.md#df-section-after-nested-declaration-a-section-heading-after-a-nested-declaration-is-the-enclosing-declarations-coordinate)).
