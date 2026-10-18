# DF-citation-site-queries: a coordinate glob selects citation sites, and `--from` reads what a point cites

**Date:** 2026-10-18

## status: Accepted

## 1. Context and decision

The first slice of coordinate globs let a reviewer count a checklist and read every body of it
in one scan, but asking who cites its points still meant one `refs` per ID, and asking what one
point leans on meant `cover --format json` over the whole tree and a `jq` program that rebuilt
the point's ancestry from `enclosing_section`. [§DF-coordinate-globs.3](DF-coordinate-globs.md#3-the-slice-this-change-ships) left those three
commands to an issue of their own, #810, with two questions open: where the scan path goes
once `refs` takes no ID, and whether `cover` may filter at all.

This change answers both and ships the three. The glob that selects catalog coordinates on
`list` and `show` selects citation sites on `refs` and `cover`, by the coordinate each site
cites ([§FS-coordinate-globs.matching.citations](../../functional-spec/FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites)): `refs --glob` lists who cites a set
([§FS-refs.globs](../../functional-spec/FS-refs.md#globs---glob-lists-who-cites-a-set)), `cover --glob` keeps only those sites in its per-file view
([§FS-cover.globs](../../functional-spec/FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)), and `refs --from` lists what one point's own body cites
([§FS-refs.from](../../functional-spec/FS-refs.md#from---from-lists-what-one-point-cites)). The bindings take the same `globs`, and `refs` takes the source as
`from` ([§FS-coordinate-globs.bindings](../../functional-spec/FS-coordinate-globs.md#bindings-node-and-python)).

## 2. The rulings it carries

- **The path stays positional in every mode.** Under `--glob` or `--from` every positional is
  the path ([§FS-refs.1.1](../../functional-spec/FS-refs.md#11-three-modes-one-path)). Rejected: a `--path` flag that only the new modes accept, which
  would give one command two spellings of one argument. The one confusion the rule invites, an
  ID typed where the path goes, gets a hint ([§FS-refs.1.3](../../functional-spec/FS-refs.md#13-an-id-typed-as-the-path-gets-a-hint)).
- **A glob matches what a site cites, declared or not.** Rejected: selecting catalog
  coordinates and listing their citations, which drops the dangling sites, exactly the ones a
  rename breaks ([§FS-refs.requirements.unresolved](../../functional-spec/FS-refs.md#requirementsunresolved-a-citation-is-listed-whether-or-not-it-resolves--realized)). The qualifier names the project the
  citation targets, never the one it sits in ([§FS-coordinate-globs.scope.targets](../../functional-spec/FS-coordinate-globs.md#scopetargets-on-a-citation-the-qualifier-names-the-project-it-targets)).
- **A pattern's bare ID is not ID mode's.** `refs X` is `--glob 'X{,.**}'`; `--glob 'X'` is
  only the citations of `X` with no section. Both are pinned beside the `--descendants`
  equivalence ([§FS-refs.globs.descendants](../../functional-spec/FS-refs.md#globsdescendants-one-id-asked-both-ways-gives-one-answer)).
- **A point owns its lead, its ancestors and itself.** What a point relies on includes what
  the chapters above it say on its behalf; a sibling or a descendant is another point. The lead
  is every site the scan gives the declaration under no section, a rejected or duplicate
  heading's included ([§FS-refs.from.owned](../../functional-spec/FS-refs.md#fromowned-a-point-owns-its-lead-its-ancestors-and-itself)).
- **`--from` counts in a sentence of its own.** `cited at 8 sites across 1 file` would read as
  if the source were the one cited, and `not cited` as if nobody cited it, so `--from` reads
  `cites at …` and `cites nothing`, and keeps the JSON object ([§FS-refs.from.records](../../functional-spec/FS-refs.md#fromrecords-what-a-record-and-a-total-say)).
- **`cover` drops no file.** [§AR-resolver.3.2](../../architecture/AR-resolver.md#32-grund-cover-drops-no-file) said `cover` filters nothing, so that a
  missing row could never be mistaken for a file with nothing to say. It is amended rather than
  broken: without `--glob` every record holds every citation, and with it the patterns select
  among the nested citations while every scanned file keeps its record.
- **Python spells the source `from_`.** It maps mechanically from Node's `from`, it is PEP 8's
  spelling for a name that clashes with a keyword, and it freezes with the rest.
- **Every new refusal is pinned** ([§FS-refs.1.2](../../functional-spec/FS-refs.md#12-refusals), [§FS-cover.6.4](../../functional-spec/FS-cover.md#64-errors)), with a pasteable glob
  where one exists, and each spelling freezes at 1.0, as the first slice's did.

Still unbuilt, as [§DF-coordinate-globs.3](DF-coordinate-globs.md#3-the-slice-this-change-ships) left them: the relation flags, chains of them, globs
over rule subjects, `cover --at`, and a `--kind` on `refs`.

## 3. When it lands

This change lands before the 1.0 candidate is cut or after v1.0.0, never in between: `1.0.0`
cuts with no new surface ([§DISC-core-concerns.9.5](../../discussions/proposals/2026-09-30-core-concerns.md#95-100)), so these flags, spellings and signatures
either freeze with the rest of the surface or wait for the first minor after it.

## 4. Compatibility

No `check` verdict moves and the managed block keeps its version, so this change owes no
release note ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). Every invocation that worked before prints the same
bytes: ID mode reads its command line as before, `cover` without `--glob` lists every
citation, and every existing binding call keeps its result. [§DF-coordinate-globs.3](DF-coordinate-globs.md#3-the-slice-this-change-ships) stays as
the record of the first slice.
