# PLAN-002-album-share: Implementation Plan: Album share

**Branch**: `002-album-share` | **Date**: 2026-10-10 | **Spec**: [§SPEC-002-album-share](spec.md#spec-002-album-share-feature-specification-album-share)

**Input**: Feature specification from `/specs/002-album-share/spec.md`

## summary: Summary

A share link is a random token stored with its album and its creation time. Serving a link
checks its age and whether it was revoked, then lists that album's photos and no others.

## technical-context: Technical Context

**Language/Version**: Python 3.12

**Primary Dependencies**: none beyond the standard library

**Storage**: one JSON file of links beside the library

**Testing**: pytest

**Target Platform**: Linux server

**Project Type**: library with a CLI

**Performance Goals**: N/A

**Constraints**: a link's lifetime is checked on every request, never cached

**Scale/Scope**: hundreds of live links per library

## constitution-check: Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] [§ART-library-first](../../.specify/memory/constitution.md#art-library-first-i-library-first): links live in `pixelbin.share`, importable without the CLI.
- [x] [§ART-cli-interface](../../.specify/memory/constitution.md#art-cli-interface-ii-cli-interface): `pixelbin share ALBUM` prints the link; `pixelbin revoke LINK`
  prints nothing and exits 0.
- [x] [§ART-test-first](../../.specify/memory/constitution.md#art-test-first-iii-test-first-non-negotiable): `tests/test_share.py` is written, and fails, before `share.py`.
- [x] [§ART-privacy-by-default](../../.specify/memory/constitution.md#art-privacy-by-default-iv-privacy-by-default): a link serves its own album only
  ([§SPEC-002-album-share.requirements.functional.fr-004](spec.md#requirementsfunctionalfr-004-fr-004)), and only while it is live.

## project-structure: Project Structure

### project-structure.documentation: Documentation (this feature)

```text
specs/002-album-share/
├── plan.md
├── spec.md
└── tasks.md
```

### project-structure.source-code: Source Code (repository root)

```text
src/pixelbin/
└── share.py

tests/
└── test_share.py
```

**Structure Decision**: Single project, beside `pixelbin.upload`.
