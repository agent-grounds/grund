# PLAN-001-photo-upload: Implementation Plan: Photo upload

**Branch**: `001-photo-upload` | **Date**: 2026-10-03 | **Spec**: [§SPEC-001-photo-upload](spec.md#spec-001-photo-upload-feature-specification-photo-upload)

**Input**: Feature specification from `/specs/001-photo-upload/spec.md`

## summary: Summary

Store an uploaded JPEG or PNG under a new ID after stripping its GPS tags, and refuse anything
else with the limit it broke. One module, `pixelbin.upload`, does the work; the CLI calls it.

## technical-context: Technical Context

**Language/Version**: Python 3.12

**Primary Dependencies**: Pillow

**Storage**: files under the library directory

**Testing**: pytest

**Target Platform**: Linux server

**Project Type**: library with a CLI

**Performance Goals**: a 20 MB upload in under 2 seconds

**Constraints**: no network access at run time

**Scale/Scope**: one owner, tens of thousands of photos

## constitution-check: Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] [§ART-library-first](../../.specify/memory/constitution.md#art-library-first-i-library-first): upload and GPS stripping live in `pixelbin.upload`, importable and
  tested without the CLI.
- [x] [§ART-cli-interface](../../.specify/memory/constitution.md#art-cli-interface-ii-cli-interface): `pixelbin upload PATH` prints the new ID on stdout; a refusal goes to
  stderr with exit 1.
- [x] [§ART-test-first](../../.specify/memory/constitution.md#art-test-first-iii-test-first-non-negotiable): the story tests in `tests/test_upload.py` are written, and fail, before
  `upload.py`.
- [x] [§ART-privacy-by-default](../../.specify/memory/constitution.md#art-privacy-by-default-iv-privacy-by-default): photos stay in the library directory, and
  [§SPEC-001-photo-upload.requirements.functional.fr-003](spec.md#requirementsfunctionalfr-003-fr-003) strips GPS tags before storage.

## project-structure: Project Structure

### project-structure.documentation: Documentation (this feature)

```text
specs/001-photo-upload/
├── plan.md              # This file (/speckit.plan command output)
├── spec.md
└── tasks.md             # /speckit.tasks command output
```

### project-structure.source-code: Source Code (repository root)

```text
src/pixelbin/
└── upload.py

tests/
└── test_upload.py
```

**Structure Decision**: Single project. Upload is one module; the CLI is a thin caller.

## complexity-tracking: Complexity Tracking

No Constitution Check violations to justify.
